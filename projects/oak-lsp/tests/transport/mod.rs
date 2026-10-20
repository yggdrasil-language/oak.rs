//! 验证 `Driver` 与传输合同：请求被分派并回写，`exit` 与对端关闭都能终止循环。

use core::{convert::Infallible, future::Future};
use oak_lsp::{
    lsp::Lsp,
    protocol::{Notification, Request, ResponseResult, ServerCapabilities},
    transport::{Driver, Inbound, MessageTransport, Outbound, RequestId},
};
use std::sync::{Arc, Mutex};

/// 只回写 `shutdown` 结果的服务替身，用于观察驱动循环的行为。
struct RecordingService;

impl Lsp for RecordingService {
    fn capabilities(&self) -> ServerCapabilities {
        ServerCapabilities::default()
    }

    fn handle_request<'a>(&'a self, request: &'a Request) -> impl Future<Output = ResponseResult> + Send + 'a {
        async move {
            match request {
                Request::Shutdown => ResponseResult::Shutdown,
                _ => ResponseResult::Shutdown,
            }
        }
    }

    fn handle_notification<'a>(&'a self, _notification: &'a Notification) -> impl Future<Output = ()> + Send + 'a {
        async {}
    }
}

/// 按脚本回放入站消息并记录出站消息的传输替身。
struct ScriptedTransport {
    inbound: Vec<Inbound>,
    sent: Arc<Mutex<Vec<Outbound>>>,
}

impl MessageTransport for ScriptedTransport {
    type Error = Infallible;

    fn recv(&mut self) -> impl Future<Output = Result<Option<Inbound>, Infallible>> + Send {
        async move { Ok(if self.inbound.is_empty() { None } else { Some(self.inbound.remove(0)) }) }
    }

    fn send(&mut self, message: Outbound) -> impl Future<Output = Result<(), Infallible>> + Send {
        async move {
            self.sent.lock().expect("outbound log poisoned").push(message);
            Ok(())
        }
    }
}

fn run(inbound: Vec<Inbound>) -> Vec<Outbound> {
    let sent = Arc::new(Mutex::new(Vec::new()));
    let transport = ScriptedTransport { inbound, sent: sent.clone() };
    futures::executor::block_on(Driver::new(RecordingService, transport).run()).expect("driver loop failed");
    Arc::try_unwrap(sent).expect("outbound log still shared").into_inner().expect("outbound log poisoned")
}

#[test]
fn request_is_dispatched_and_answered_with_the_same_id() {
    let sent = run(vec![Inbound::Request { id: RequestId::Number(7), request: Request::Shutdown }]);
    assert_eq!(sent.len(), 1);
    assert!(matches!(
        &sent[0],
        Outbound::Response { id: RequestId::Number(7), result: ResponseResult::Shutdown }
    ));
}

#[test]
fn exit_notification_stops_the_driver_before_peer_close() {
    let sent = run(vec![
        Inbound::Request { id: RequestId::String("first".into()), request: Request::Shutdown },
        Inbound::Notification(Notification::Exit),
        Inbound::Request { id: RequestId::Number(2), request: Request::Shutdown },
    ]);
    assert_eq!(sent.len(), 1, "requests after `exit` must not be answered");
    assert!(matches!(&sent[0], Outbound::Response { id: RequestId::String(id), .. } if id == "first"));
}

#[test]
fn peer_close_ends_the_driver_without_answering() {
    assert!(run(vec![]).is_empty());
}
