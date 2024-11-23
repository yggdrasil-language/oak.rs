#![doc = include_str!("readme.md")]
/// GLSL token type definitions.
pub mod token_type;

use crate::{language::GlslLanguage, lexer::token_type::GlslTokenType};
use oak_core::{Lexer, LexerCache, LexerState, OakError, TextEdit, lexer::LexOutput, source::Source};

pub(crate) type State<'a, S> = LexerState<'a, S, GlslLanguage>;

/// A lexer for the GLSL language.
#[derive(Clone)]
pub struct GlslLexer<'config> {
    _config: &'config GlslLanguage,
}

impl<'config> GlslLexer<'config> {
    /// Creates a new GLSL lexer.
    pub fn new(config: &'config GlslLanguage) -> Self {
        Self { _config: config }
    }

    fn run<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        while state.not_at_end() {
            if self.skip_whitespace(state) {
                continue;
            }
            if self.lex_newline(state) {
                continue;
            }
            if self.lex_comment(state) {
                continue;
            }
            if self.lex_directive(state) {
                continue;
            }
            if self.lex_string(state) {
                continue;
            }
            if self.lex_number(state) {
                continue;
            }
            if self.lex_identifier_or_keyword(state) {
                continue;
            }
            if self.lex_operator_or_delimiter(state) {
                continue;
            }

            let start_pos = state.get_position();
            if let Some(ch) = state.peek() {
                state.advance(ch.len_utf8());
                state.add_token(GlslTokenType::Error, start_pos, state.get_position());
            }
        }

        Ok(())
    }

    /// Skips whitespace.
    fn skip_whitespace<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let start_pos = state.get_position();

        while let Some(ch) = state.peek() {
            if ch == ' ' || ch == '\t' {
                state.advance(ch.len_utf8());
            }
            else {
                break;
            }
        }

        if state.get_position() > start_pos {
            state.add_token(GlslTokenType::Whitespace, start_pos, state.get_position());
            true
        }
        else {
            false
        }
    }

    /// Lexes a newline.
    fn lex_newline<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let start_pos = state.get_position();

        if let Some('\n') = state.peek() {
            state.advance(1);
            state.add_token(GlslTokenType::Newline, start_pos, state.get_position());
            true
        }
        else if let Some('\r') = state.peek() {
            state.advance(1);
            if let Some('\n') = state.peek() {
                state.advance(1);
            }
            state.add_token(GlslTokenType::Newline, start_pos, state.get_position());
            true
        }
        else {
            false
        }
    }

    /// Lexes a line or block comment.
    fn lex_comment<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let start_pos = state.get_position();

        if let Some('/') = state.peek() {
            if let Some('/') = state.peek_next_n(1) {
                state.advance(2);
                while let Some(ch) = state.peek() {
                    if ch == '\n' || ch == '\r' {
                        break;
                    }
                    state.advance(ch.len_utf8());
                }
                state.add_token(GlslTokenType::Comment, start_pos, state.get_position());
                return true;
            }
        }

        if let Some('/') = state.peek() {
            if let Some('*') = state.peek_next_n(1) {
                state.advance(2);
                while state.not_at_end() {
                    if let Some('*') = state.peek() {
                        if let Some('/') = state.peek_next_n(1) {
                            state.advance(2);
                            break;
                        }
                    }
                    if let Some(ch) = state.peek() {
                        state.advance(ch.len_utf8());
                    }
                }
                state.add_token(GlslTokenType::Comment, start_pos, state.get_position());
                return true;
            }
        }

        false
    }

    /// Lexes a preprocessor directive line such as `#version 450 core`.
    fn lex_directive<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let start_pos = state.get_position();

        if let Some('#') = state.peek() {
            state.advance(1);
            while let Some(ch) = state.peek() {
                if ch == '\n' || ch == '\r' {
                    break;
                }
                state.advance(ch.len_utf8());
            }
            state.add_token(GlslTokenType::Directive, start_pos, state.get_position());
            return true;
        }

        false
    }

    /// Lexes a string literal.
    fn lex_string<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let start_pos = state.get_position();

        if let Some('"') = state.peek() {
            state.advance(1);
            let mut escaped = false;

            while let Some(ch) = state.peek() {
                if escaped {
                    escaped = false;
                }
                else if ch == '\\' {
                    escaped = true;
                }
                else if ch == '"' {
                    state.advance(1);
                    break;
                }
                else if ch == '\n' || ch == '\r' {
                    break;
                }
                state.advance(ch.len_utf8());
            }

            state.add_token(GlslTokenType::StringLiteral, start_pos, state.get_position());
            return true;
        }

        false
    }

    /// Lexes a number literal.
    fn lex_number<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let start_pos = state.get_position();

        if let Some(ch) = state.peek() {
            if ch.is_ascii_digit() || (ch == '.' && state.peek_next_n(1).is_some_and(|next| next.is_ascii_digit())) {
                let mut is_float = ch == '.';

                if ch == '0' && matches!(state.peek_next_n(1), Some('x') | Some('X')) {
                    state.advance(2);
                    while let Some(ch) = state.peek() {
                        if ch.is_ascii_hexdigit() {
                            state.advance(1);
                        }
                        else {
                            break;
                        }
                    }
                }
                else {
                    while let Some(ch) = state.peek() {
                        if ch.is_ascii_digit() {
                            state.advance(1);
                        }
                        else {
                            break;
                        }
                    }

                    if let Some('.') = state.peek() {
                        state.advance(1);
                        is_float = true;
                        while let Some(ch) = state.peek() {
                            if ch.is_ascii_digit() {
                                state.advance(1);
                            }
                            else {
                                break;
                            }
                        }
                    }

                    if matches!(state.peek(), Some('e') | Some('E')) {
                        let saved_pos = state.get_position();
                        state.advance(1);
                        if matches!(state.peek(), Some('+') | Some('-')) {
                            state.advance(1);
                        }
                        let exp_start = state.get_position();
                        while let Some(ch) = state.peek() {
                            if ch.is_ascii_digit() {
                                state.advance(1);
                            }
                            else {
                                break;
                            }
                        }
                        if state.get_position() == exp_start {
                            state.set_position(saved_pos);
                        }
                        else {
                            is_float = true;
                        }
                    }
                }

                if let Some(suffix) = state.peek() {
                    if matches!(suffix, 'f' | 'F' | 'h' | 'H' | 'l' | 'L' | 'u' | 'U') {
                        if matches!(suffix, 'f' | 'F' | 'h' | 'H' | 'l' | 'L') {
                            is_float = true;
                        }
                        state.advance(1);
                    }
                }

                let kind = if is_float { GlslTokenType::FloatLiteral } else { GlslTokenType::IntegerLiteral };
                state.add_token(kind, start_pos, state.get_position());
                return true;
            }
        }

        false
    }

    /// Lexes an identifier or keyword.
    fn lex_identifier_or_keyword<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let start_pos = state.get_position();

        if let Some(ch) = state.peek() {
            if ch.is_ascii_alphabetic() || ch == '_' {
                state.advance(1);

                while let Some(ch) = state.peek() {
                    if ch.is_ascii_alphanumeric() || ch == '_' {
                        state.advance(1);
                    }
                    else {
                        break;
                    }
                }

                let text = state.get_text_in(oak_core::Range { start: start_pos, end: state.get_position() });
                let kind = match text.as_ref() {
                    "void" => GlslTokenType::Void,
                    "bool" => GlslTokenType::Bool,
                    "int" => GlslTokenType::Int,
                    "uint" => GlslTokenType::Uint,
                    "float" => GlslTokenType::Float,
                    "double" => GlslTokenType::Double,
                    "vec2" => GlslTokenType::Vec2,
                    "vec3" => GlslTokenType::Vec3,
                    "vec4" => GlslTokenType::Vec4,
                    "ivec2" => GlslTokenType::IVec2,
                    "ivec3" => GlslTokenType::IVec3,
                    "ivec4" => GlslTokenType::IVec4,
                    "uvec2" => GlslTokenType::UVec2,
                    "uvec3" => GlslTokenType::UVec3,
                    "uvec4" => GlslTokenType::UVec4,
                    "bvec2" => GlslTokenType::BVec2,
                    "bvec3" => GlslTokenType::BVec3,
                    "bvec4" => GlslTokenType::BVec4,
                    "dvec2" => GlslTokenType::DVec2,
                    "dvec3" => GlslTokenType::DVec3,
                    "dvec4" => GlslTokenType::DVec4,
                    "mat2" => GlslTokenType::Mat2,
                    "mat3" => GlslTokenType::Mat3,
                    "mat4" => GlslTokenType::Mat4,
                    "mat2x2" => GlslTokenType::Mat2x2,
                    "mat2x3" => GlslTokenType::Mat2x3,
                    "mat2x4" => GlslTokenType::Mat2x4,
                    "mat3x2" => GlslTokenType::Mat3x2,
                    "mat3x3" => GlslTokenType::Mat3x3,
                    "mat3x4" => GlslTokenType::Mat3x4,
                    "mat4x2" => GlslTokenType::Mat4x2,
                    "mat4x3" => GlslTokenType::Mat4x3,
                    "mat4x4" => GlslTokenType::Mat4x4,
                    "sampler2D" => GlslTokenType::Sampler2D,
                    "sampler3D" => GlslTokenType::Sampler3D,
                    "samplerCube" => GlslTokenType::SamplerCube,
                    "sampler2DArray" => GlslTokenType::Sampler2DArray,
                    "samplerCubeArray" => GlslTokenType::SamplerCubeArray,
                    "sampler2DShadow" => GlslTokenType::Sampler2DShadow,
                    "samplerCubeShadow" => GlslTokenType::SamplerCubeShadow,
                    "sampler2DArrayShadow" => GlslTokenType::Sampler2DArrayShadow,
                    "isampler2D" => GlslTokenType::ISampler2D,
                    "usampler2D" => GlslTokenType::USampler2D,
                    "image2D" => GlslTokenType::Image2D,
                    "atomic_uint" => GlslTokenType::AtomicUint,
                    "const" => GlslTokenType::Const,
                    "uniform" => GlslTokenType::Uniform,
                    "in" => GlslTokenType::In,
                    "out" => GlslTokenType::Out,
                    "inout" => GlslTokenType::Inout,
                    "buffer" => GlslTokenType::Buffer,
                    "shared" => GlslTokenType::Shared,
                    "layout" => GlslTokenType::Layout,
                    "precision" => GlslTokenType::Precision,
                    "highp" => GlslTokenType::Highp,
                    "mediump" => GlslTokenType::Mediump,
                    "lowp" => GlslTokenType::Lowp,
                    "flat" => GlslTokenType::Flat,
                    "smooth" => GlslTokenType::Smooth,
                    "noperspective" => GlslTokenType::Noperspective,
                    "centroid" => GlslTokenType::Centroid,
                    "sample" => GlslTokenType::Sample,
                    "invariant" => GlslTokenType::Invariant,
                    "patch" => GlslTokenType::Patch,
                    "readonly" => GlslTokenType::Readonly,
                    "writeonly" => GlslTokenType::Writeonly,
                    "coherent" => GlslTokenType::Coherent,
                    "volatile" => GlslTokenType::Volatile,
                    "restrict" => GlslTokenType::Restrict,
                    "if" => GlslTokenType::If,
                    "else" => GlslTokenType::Else,
                    "for" => GlslTokenType::For,
                    "while" => GlslTokenType::While,
                    "do" => GlslTokenType::Do,
                    "switch" => GlslTokenType::Switch,
                    "case" => GlslTokenType::Case,
                    "default" => GlslTokenType::Default,
                    "break" => GlslTokenType::Break,
                    "continue" => GlslTokenType::Continue,
                    "return" => GlslTokenType::Return,
                    "discard" => GlslTokenType::Discard,
                    "struct" => GlslTokenType::Struct,
                    "true" | "false" => GlslTokenType::BooleanLiteral,
                    _ => GlslTokenType::Identifier,
                };

                state.add_token(kind, start_pos, state.get_position());
                return true;
            }
        }

        false
    }

    /// Lexes an operator or a delimiter.
    fn lex_operator_or_delimiter<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let start_pos = state.get_position();

        if let Some(ch) = state.peek() {
            let kind = match ch {
                '+' => {
                    state.advance(1);
                    if let Some('=') = state.peek() {
                        state.advance(1);
                        GlslTokenType::PlusAssign
                    }
                    else if let Some('+') = state.peek() {
                        state.advance(1);
                        GlslTokenType::Increment
                    }
                    else {
                        GlslTokenType::Plus
                    }
                }
                '-' => {
                    state.advance(1);
                    if let Some('=') = state.peek() {
                        state.advance(1);
                        GlslTokenType::MinusAssign
                    }
                    else if let Some('-') = state.peek() {
                        state.advance(1);
                        GlslTokenType::Decrement
                    }
                    else if let Some('>') = state.peek() {
                        state.advance(1);
                        GlslTokenType::Arrow
                    }
                    else {
                        GlslTokenType::Minus
                    }
                }
                '*' => {
                    state.advance(1);
                    if let Some('=') = state.peek() {
                        state.advance(1);
                        GlslTokenType::MultiplyAssign
                    }
                    else {
                        GlslTokenType::Multiply
                    }
                }
                '/' => {
                    state.advance(1);
                    if let Some('=') = state.peek() {
                        state.advance(1);
                        GlslTokenType::DivideAssign
                    }
                    else {
                        GlslTokenType::Divide
                    }
                }
                '%' => {
                    state.advance(1);
                    if let Some('=') = state.peek() {
                        state.advance(1);
                        GlslTokenType::ModuloAssign
                    }
                    else {
                        GlslTokenType::Modulo
                    }
                }
                '=' => {
                    state.advance(1);
                    if let Some('=') = state.peek() {
                        state.advance(1);
                        GlslTokenType::Equal
                    }
                    else {
                        GlslTokenType::Assign
                    }
                }
                '!' => {
                    state.advance(1);
                    if let Some('=') = state.peek() {
                        state.advance(1);
                        GlslTokenType::NotEqual
                    }
                    else {
                        GlslTokenType::LogicalNot
                    }
                }
                '<' => {
                    state.advance(1);
                    if let Some('=') = state.peek() {
                        state.advance(1);
                        GlslTokenType::LessEqual
                    }
                    else if let Some('<') = state.peek() {
                        state.advance(1);
                        if let Some('=') = state.peek() {
                            state.advance(1);
                            GlslTokenType::LeftShiftAssign
                        }
                        else {
                            GlslTokenType::LeftShift
                        }
                    }
                    else {
                        GlslTokenType::Less
                    }
                }
                '>' => {
                    state.advance(1);
                    if let Some('=') = state.peek() {
                        state.advance(1);
                        GlslTokenType::GreaterEqual
                    }
                    else if let Some('>') = state.peek() {
                        state.advance(1);
                        if let Some('=') = state.peek() {
                            state.advance(1);
                            GlslTokenType::RightShiftAssign
                        }
                        else {
                            GlslTokenType::RightShift
                        }
                    }
                    else {
                        GlslTokenType::Greater
                    }
                }
                '&' => {
                    state.advance(1);
                    if let Some('&') = state.peek() {
                        state.advance(1);
                        GlslTokenType::LogicalAnd
                    }
                    else if let Some('=') = state.peek() {
                        state.advance(1);
                        GlslTokenType::BitwiseAndAssign
                    }
                    else {
                        GlslTokenType::BitwiseAnd
                    }
                }
                '|' => {
                    state.advance(1);
                    if let Some('|') = state.peek() {
                        state.advance(1);
                        GlslTokenType::LogicalOr
                    }
                    else if let Some('=') = state.peek() {
                        state.advance(1);
                        GlslTokenType::BitwiseOrAssign
                    }
                    else {
                        GlslTokenType::BitwiseOr
                    }
                }
                '^' => {
                    state.advance(1);
                    if let Some('=') = state.peek() {
                        state.advance(1);
                        GlslTokenType::BitwiseXorAssign
                    }
                    else {
                        GlslTokenType::BitwiseXor
                    }
                }
                '~' => {
                    state.advance(1);
                    GlslTokenType::BitwiseNot
                }
                '?' => {
                    state.advance(1);
                    GlslTokenType::Conditional
                }
                ':' => {
                    state.advance(1);
                    GlslTokenType::Colon
                }
                ';' => {
                    state.advance(1);
                    GlslTokenType::Semicolon
                }
                ',' => {
                    state.advance(1);
                    GlslTokenType::Comma
                }
                '(' => {
                    state.advance(1);
                    GlslTokenType::LeftParen
                }
                ')' => {
                    state.advance(1);
                    GlslTokenType::RightParen
                }
                '[' => {
                    state.advance(1);
                    GlslTokenType::LeftBracket
                }
                ']' => {
                    state.advance(1);
                    GlslTokenType::RightBracket
                }
                '{' => {
                    state.advance(1);
                    GlslTokenType::LeftBrace
                }
                '}' => {
                    state.advance(1);
                    GlslTokenType::RightBrace
                }
                '.' => {
                    state.advance(1);
                    GlslTokenType::Dot
                }
                '#' => {
                    state.advance(1);
                    GlslTokenType::Hash
                }
                _ => return false,
            };

            state.add_token(kind, start_pos, state.get_position());
            return true;
        }

        false
    }
}

impl<'config> Lexer<GlslLanguage> for GlslLexer<'config> {
    fn lex<'a, S: Source + ?Sized>(&self, source: &'a S, _edits: &[TextEdit], cache: &'a mut impl LexerCache<GlslLanguage>) -> LexOutput<GlslLanguage> {
        let mut state = LexerState::new(source);
        let result = self.run(&mut state);
        if result.is_ok() {
            state.add_eof();
        }
        state.finish_with_cache(result, cache)
    }
}
