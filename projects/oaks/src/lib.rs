#![feature(new_range_api)]
#![warn(missing_docs)]
#![doc = include_str!("readme.md")]
#![doc(html_logo_url = "https://raw.githubusercontent.com/ygg-lang/oaks/refs/heads/dev/documents/logo.svg")]
#![doc(html_favicon_url = "https://raw.githubusercontent.com/ygg-lang/oaks/refs/heads/dev/documents/logo.svg")]

/// Re-export of the core parsing and tree structures.
pub use oak_core::*;
/// Re-export of the folding range provider.
pub use oak_folding::{FoldingProvider, FoldingRange, FoldingRangeKind};
/// Re-export of the hover information provider.
pub use oak_hover::{Hover, HoverProvider};
/// Re-export of the Language Server Protocol types, service and dispatch contract.
pub use oak_lsp::{LanguageService, Location, Lsp, LspRange};
/// Re-export of the definition and reference providers.
pub use oak_navigation::{DefinitionProvider, ReferencesProvider};
/// Re-export of the semantic tokens provider.
pub use oak_semantic_tokens::{SemanticToken, SemanticTokensProvider};
/// Re-export of the document structural view provider.
pub use oak_structural_view::{StructureItem, StructureProvider};
/// Re-export of the symbol information provider.
pub use oak_symbols::{SymbolInformation, SymbolProvider};
/// Re-export of the Virtual File System.
pub use oak_vfs::{FileMetadata, FileType, MemoryVfs, Vfs};

mod languages;

/// Re-export of language-specific implementations.
pub use languages::*;
