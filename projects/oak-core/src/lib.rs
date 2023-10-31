#![doc = include_str!("readme.md")]
#![allow(incomplete_features)]
#![allow(internal_features)] // Allow using internal features like `core_intrinsics`
#![feature(allocator_api)] // Required for custom arena allocation and manual memory management
#![feature(core_intrinsics)] // Used for `likely`/`unlikely` hints in performance-critical paths
#![feature(checked_type_aliases)] // Allows more flexible type aliases for complex generic AST structures
#![feature(new_range_api)] // Uses the modernized `core::range` API for precise source tracking
#![feature(portable_simd)] // Enables SIMD acceleration for high-performance lexing and parsing
#![feature(slice_ptr_get)] // Provides ergonomic access to raw pointers within slices
#![feature(trusted_len)] // Optimizes iterator performance by trusting length hints in core structures
#![warn(missing_docs)]
#![doc(html_logo_url = "https://raw.githubusercontent.com/ygg-lang/oaks/refs/heads/dev/documents/logo.svg")]
#![doc(html_favicon_url = "https://raw.githubusercontent.com/ygg-lang/oaks/refs/heads/dev/documents/logo.svg")]
// #![feature(nonnull_slice_from_raw_parts)] // Essential for creating raw slices in `SyntaxArena` without null checks

/// Incremental tree builder and cache management.
pub mod builder;
/// Unified `diagnostic` adapters for Oak parser output.
#[cfg(feature = "diagnostic")]
pub mod diagnostic;
/// Error handling and diagnostic reporting for the parsing system.
pub mod errors;
/// Language definition trait for coordinating language-specific components.
pub mod language;
/// Lexical analysis and tokenization functionality.
pub mod lexer;
/// Memory management utilities (Arena, Bump).
pub mod memory;
/// Parsing functionality for converting tokens to kind trees.
pub mod parser;
#[cfg(feature = "serde")]
pub mod serde_arc_str;
#[cfg(feature = "serde")]
pub mod serde_range;
/// Source text management and location tracking.
pub mod source;
/// Tree structures for representing kind trees (green and red trees).
pub mod tree;
/// Tree traversal and transformation utilities.
pub mod visitor;

/// Unified AST processing interface.
pub mod ast;

/// Helper utilities for common operations.
pub mod helpers;
/// Syntax-agnostic structured query primitives for document trees.
pub mod query;

pub use core::range::Range;

pub use crate::{
    ast::{AstError, AstProcessor, DefaultAstProcessor, RedTreeExt, VisitResult, Visitor},
    builder::{Builder, BuilderCache},
    errors::{OakDiagnostics, OakError, OakErrorKind},
    language::{ElementRole, ElementType, Language, LanguageCategory, TokenRole, TokenType, UniversalElementRole, UniversalTokenRole},
    lexer::{LexOutput, Lexer, LexerCache, LexerState, NoLexerCache, Token, TokenStream, Tokens},
    memory::arena::SyntaxArena,
    parser::{Associativity, CachingParseSession, ContentCache, OperatorInfo, ParseCache, ParseOutput, ParseSession, Parser, ParserState, Pratt, PrattParser, binary, parse, parse_one_pass, postfix, state::TreeSink, unary},
    source::{Source, SourceText, TextEdit},
    tree::{GreenNode, GreenTree, RedLeaf, RedNode, RedTree},
};

pub use triomphe::Arc;
