//! 传输适配合同：与具体 server 提供者正交。
//!
//! 本模块只定义消息边界与驱动循环，不依赖 tokio、axum、rocket、tower 或任何运行时。
//! stdio、HTTP、WebSocket 等提供者各自实现 [`MessageTransport`]，framing 与 JSON-RPC
//! 编解码留在提供者侧，因此同一个 [`Lsp`] 服务可以挂到任意传输上。

use crate::{
    lsp::Lsp,
    protocol::{Notification, Request, ResponseResult},
};
use core::future::Future;

/// JSON-RPC 请求标识。
///
/// 协议允许数字或字符串标识，驱动循环原样回传，不解释其含义。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RequestId {
    /// 数字标识。
    Number(i64),
    /// 字符串标识。
    String(String),
}

/// 从传输层读到的一条入站消息。
#[derive(Debug, Clone)]
pub enum Inbound {
    /// 需要响应的请求。
    Request {
        /// 对应的 JSON-RPC 标识。
        id: RequestId,
        /// 已建模的请求。
        request: Request,
    },
    /// 不需要响应的通知。
    Notification(Notification),
}

/// 要写给传输层的一条出站消息。
#[derive(Debug, Clone)]
pub enum Outbound {
    /// 请求响应。
    Response {
        /// 对应的 JSON-RPC 标识。
        id: RequestId,
        /// 响应载荷。
        result: ResponseResult,
    },
}

/// 传输适配合同。
///
/// 实现者负责把线上帧解码为 [`Inbound`] 并编码 [`Outbound`]，不拥有语言语义，
/// 也不决定请求分派。
pub trait MessageTransport: Send {
    /// 传输错误。
    type Error: core::fmt::Debug;

    /// 读取下一条消息；返回 `None` 表示对端关闭。
    fn recv(&mut self) -> impl Future<Output = Result<Option<Inbound>, Self::Error>> + Send;

    /// 写出下一条消息。
    fn send(&mut self, message: Outbound) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

/// 驱动循环的失败原因。
#[derive(Debug)]
pub enum DriverError<E: core::fmt::Debug> {
    /// 传输层失败。
    Transport(E),
}

/// 把 [`Lsp`] 服务与 [`MessageTransport`] 连接起来的驱动循环。
pub struct Driver<S: Lsp, T: MessageTransport> {
    /// 语言服务。
    pub service: S,
    /// 传输适配者。
    pub transport: T,
}

impl<S: Lsp, T: MessageTransport> Driver<S, T> {
    /// 创建驱动。
    pub fn new(service: S, transport: T) -> Self {
        Self { service, transport }
    }

    /// 持续读入消息并分派，直到对端关闭或收到 `exit` 通知。
    ///
    /// # Errors
    ///
    /// 传输层读取或写入失败时返回 [`DriverError::Transport`]。
    pub async fn run(mut self) -> Result<(), DriverError<T::Error>> {
        loop {
            let inbound = match self.transport.recv().await {
                Ok(Some(inbound)) => inbound,
                Ok(None) => return Ok(()),
                Err(error) => return Err(DriverError::Transport(error)),
            };
            match inbound {
                Inbound::Request { id, request } => {
                    let result = self.service.handle_request(&request).await;
                    self.transport.send(Outbound::Response { id, result }).await.map_err(DriverError::Transport)?;
                }
                Inbound::Notification(Notification::Exit) => return Ok(()),
                Inbound::Notification(notification) => self.service.handle_notification(&notification).await,
            }
        }
    }
}
