use alloc::string::String;
use core::marker::PhantomData;
use oak_core::language::Language;
use std::collections::HashMap;

use crate::document::PrinterConfig;

/// Formatting state alias used by `FormatRule` defaults.
pub type FormatState = DefaultFormatState;

/// Context passed through RedTree `FormatRule` application.
#[derive(Debug, Clone)]
pub struct FormatContext<L: Language, C: Clone, S: Clone = FormatState> {
    /// Language-specific formatter configuration.
    pub config: C,
    /// Mutable formatting state (indent, overrides).
    pub state: S,
    /// Printer configuration for final document rendering.
    pub printer_config: PrinterConfig,
    _language: PhantomData<L>,
}

impl<L: Language, C: Clone, S: Default + Clone> FormatContext<L, C, S> {
    /// Create a context with the given configuration, default state, and printer config.
    pub fn new(config: C, printer_config: PrinterConfig) -> Self {
        Self { config, state: S::default(), printer_config, _language: PhantomData }
    }
}

/// Default format state implementation
///
/// This struct provides a default implementation of the format state.
#[derive(Debug, Clone, Default)]
pub struct DefaultFormatState {
    /// Local configuration overrides
    pub local_config: HashMap<String, serde_json::Value>,
    /// Custom state values
    pub custom_state: HashMap<String, serde_json::Value>,
    /// Current indentation level
    pub indent_level: usize,
    /// Whether to force single line formatting
    pub force_single_line: bool,
    /// Whether to align elements
    pub align_elements: bool,
}
