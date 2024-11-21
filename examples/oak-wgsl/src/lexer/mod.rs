#![doc = include_str!("readme.md")]
/// WGSL token type definitions.
pub mod token_type;

use crate::{language::WgslLanguage, lexer::token_type::WgslTokenType};
use oak_core::{Lexer, LexerCache, LexerState, OakError, TextEdit, lexer::LexOutput, source::Source};

pub(crate) type State<'a, S> = LexerState<'a, S, WgslLanguage>;

/// A lexer for the WGSL language.
#[derive(Clone)]
pub struct WgslLexer<'config> {
    config: &'config WgslLanguage,
}

impl<'config> WgslLexer<'config> {
    /// Creates a new WGSL lexer.
    pub fn new(config: &'config WgslLanguage) -> Self {
        Self { config }
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
            state.add_token(WgslTokenType::Whitespace, start_pos, state.get_position());
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
            state.add_token(WgslTokenType::Newline, start_pos, state.get_position());
            true
        }
        else if let Some('\r') = state.peek() {
            state.advance(1);
            if let Some('\n') = state.peek() {
                state.advance(1);
            }
            state.add_token(WgslTokenType::Newline, start_pos, state.get_position());
            true
        }
        else {
            false
        }
    }

    /// Lexes a comment.
    fn lex_comment<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let start_pos = state.get_position();

        // Line comment: //
        if let Some('/') = state.peek() {
            if state.peek_next_n(1) == Some('/') {
                state.advance(2);

                while let Some(ch) = state.peek() {
                    if ch == '\n' || ch == '\r' {
                        break;
                    }
                    state.advance(ch.len_utf8());
                }

                state.add_token(WgslTokenType::Comment, start_pos, state.get_position());
                return true;
            }
        }

        // Block comment: /* */
        if let Some('/') = state.peek() {
            if state.peek_next_n(1) == Some('*') {
                state.advance(2);
                let mut depth = 1;

                while depth > 0 && state.not_at_end() {
                    if let Some('/') = state.peek() {
                        if state.peek_next_n(1) == Some('*') {
                            state.advance(2);
                            depth += 1;
                            continue;
                        }
                    }

                    if let Some('*') = state.peek() {
                        state.advance(1);
                        if state.peek_next_n(1) == Some('/') {
                            state.advance(1);
                            depth -= 1;
                        }
                    }
                    else if let Some(ch) = state.peek() {
                        state.advance(ch.len_utf8());
                    }
                    else {
                        break;
                    }
                }

                state.add_token(WgslTokenType::Comment, start_pos, state.get_position());
                return true;
            }
        }

        false
    }

    /// Lexes a string literal.
    fn lex_string<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let start_pos = state.get_position();

        if let Some('"') = state.peek() {
            state.advance(1); // Skip starting quote

            while let Some(ch) = state.peek() {
                if ch == '"' {
                    state.advance(1); // Skip ending quote
                    state.add_token(WgslTokenType::StringLiteral, start_pos, state.get_position());
                    return true;
                }
                else if ch == '\\' {
                    state.advance(1); // Skip escape character
                    if let Some(escaped) = state.peek() {
                        state.advance(escaped.len_utf8());
                    }
                }
                else if ch == '\n' || ch == '\r' {
                    break;
                }
                else {
                    state.advance(ch.len_utf8());
                }
            }

            state.add_token(WgslTokenType::Error, start_pos, state.get_position());
            return true;
        }

        false
    }

    /// Lexes a number literal.
    fn lex_number<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let start_pos = state.get_position();

        if let Some(ch) = state.peek() {
            if ch.is_ascii_digit() {
                // Hexadecimal
                if ch == '0' && state.peek_next_n(1) == Some('x') {
                    state.advance(2); // Skip "0x"

                    let mut has_digits = false;
                    while let Some(ch) = state.peek() {
                        if ch.is_ascii_hexdigit() {
                            state.advance(1);
                            has_digits = true;
                        }
                        else {
                            break;
                        }
                    }

                    if has_digits {
                        // Check suffix
                        if let Some('u') = state.peek() {
                            state.advance(1);
                        }
                        else if let Some('i') = state.peek() {
                            state.advance(1);
                        }

                        state.add_token(WgslTokenType::IntegerLiteral, start_pos, state.get_position());
                        return true;
                    }
                }

                // Decimal number
                while let Some(ch) = state.peek() {
                    if ch.is_ascii_digit() {
                        state.advance(1);
                    }
                    else {
                        break;
                    }
                }

                let mut is_float = false;

                // Decimal point
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

                // Exponent
                if let Some('e') = state.peek() {
                    state.advance(1);
                    is_float = true;
                    if let Some('+') | Some('-') = state.peek() {
                        state.advance(1);
                    }
                    while let Some(ch) = state.peek() {
                        if ch.is_ascii_digit() {
                            state.advance(1);
                        }
                        else {
                            break;
                        }
                    }
                }

                // Suffix
                if let Some('f') = state.peek() {
                    state.advance(1);
                    is_float = true;
                }
                else if let Some('u') = state.peek() {
                    state.advance(1);
                }
                else if let Some('i') = state.peek() {
                    state.advance(1);
                }

                if is_float {
                    state.add_token(WgslTokenType::FloatLiteral, start_pos, state.get_position());
                }
                else {
                    state.add_token(WgslTokenType::IntegerLiteral, start_pos, state.get_position());
                }
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
                    "bool" => WgslTokenType::BoolKw,
                    "i32" => WgslTokenType::I32Kw,
                    "u32" => WgslTokenType::U32Kw,
                    "f32" => WgslTokenType::F32Kw,
                    "f16" => WgslTokenType::F16Kw,
                    "vec2" => WgslTokenType::Vec2Kw,
                    "vec3" => WgslTokenType::Vec3Kw,
                    "vec4" => WgslTokenType::Vec4Kw,
                    "mat2x2" => WgslTokenType::Mat2x2Kw,
                    "mat2x3" => WgslTokenType::Mat2x3Kw,
                    "mat2x4" => WgslTokenType::Mat2x4Kw,
                    "mat3x2" => WgslTokenType::Mat3x2Kw,
                    "mat3x3" => WgslTokenType::Mat3x3Kw,
                    "mat3x4" => WgslTokenType::Mat3x4Kw,
                    "mat4x2" => WgslTokenType::Mat4x2Kw,
                    "mat4x3" => WgslTokenType::Mat4x3Kw,
                    "mat4x4" => WgslTokenType::Mat4x4Kw,
                    "array" => WgslTokenType::ArrayKw,
                    "ptr" => WgslTokenType::PtrKw,
                    "atomic" => WgslTokenType::AtomicKw,
                    "sampler" => WgslTokenType::SamplerKw,
                    "sampler_comparison" => WgslTokenType::SamplerComparisonKw,
                    "texture_1d" => WgslTokenType::Texture1dKw,
                    "texture_2d" => WgslTokenType::Texture2dKw,
                    "texture_2d_array" => WgslTokenType::Texture2dArrayKw,
                    "texture_3d" => WgslTokenType::Texture3dKw,
                    "texture_cube" => WgslTokenType::TextureCubeKw,
                    "texture_cube_array" => WgslTokenType::TextureCubeArrayKw,
                    "texture_multisampled_2d" => WgslTokenType::TextureMultisampled2dKw,
                    "texture_storage_1d" => WgslTokenType::TextureStorage1dKw,
                    "texture_storage_2d" => WgslTokenType::TextureStorage2dKw,
                    "texture_storage_2d_array" => WgslTokenType::TextureStorage2dArrayKw,
                    "texture_storage_3d" => WgslTokenType::TextureStorage3dKw,
                    "texture_depth_2d" => WgslTokenType::TextureDepth2dKw,
                    "texture_depth_cube" => WgslTokenType::TextureDepthCubeKw,
                    "texture_depth_multisampled_2d" => WgslTokenType::TextureDepthMultisampled2dKw,
                    "struct" => WgslTokenType::StructKw,
                    "fn" => WgslTokenType::FnKw,
                    "let" => WgslTokenType::LetKw,
                    "var" => WgslTokenType::VarKw,
                    "const" => WgslTokenType::ConstKw,
                    "override" => WgslTokenType::OverrideKw,
                    "alias" => WgslTokenType::AliasKw,
                    "type" => WgslTokenType::TypeKw,
                    "return" => WgslTokenType::ReturnKw,
                    "if" => WgslTokenType::IfKw,
                    "else" => WgslTokenType::ElseKw,
                    "switch" => WgslTokenType::SwitchKw,
                    "case" => WgslTokenType::CaseKw,
                    "default" => WgslTokenType::DefaultKw,
                    "loop" => WgslTokenType::LoopKw,
                    "while" => WgslTokenType::WhileKw,
                    "for" => WgslTokenType::ForKw,
                    "break" => WgslTokenType::BreakKw,
                    "continue" => WgslTokenType::ContinueKw,
                    "discard" => WgslTokenType::DiscardKw,
                    "enable" => WgslTokenType::EnableKw,
                    "requires" => WgslTokenType::RequiresKw,
                    "function" => WgslTokenType::FunctionKw,
                    "private" => WgslTokenType::PrivateKw,
                    "workgroup" => WgslTokenType::WorkgroupKw,
                    "uniform" => WgslTokenType::UniformKw,
                    "storage" => WgslTokenType::StorageKw,
                    "read" => WgslTokenType::ReadKw,
                    "write" => WgslTokenType::WriteKw,
                    "read_write" => WgslTokenType::ReadWriteKw,
                    "vertex" => WgslTokenType::VertexKw,
                    "fragment" => WgslTokenType::FragmentKw,
                    "compute" => WgslTokenType::ComputeKw,
                    "binding" => WgslTokenType::BindingKw,
                    "group" => WgslTokenType::GroupKw,
                    "location" => WgslTokenType::LocationKw,
                    "builtin" => WgslTokenType::BuiltinKw,
                    "interpolate" => WgslTokenType::InterpolateKw,
                    "invariant" => WgslTokenType::InvariantKw,
                    "size" => WgslTokenType::SizeKw,
                    "align" => WgslTokenType::AlignKw,
                    "workgroup_size" => WgslTokenType::WorkgroupSizeKw,
                    "true" => WgslTokenType::BoolLiteral,
                    "false" => WgslTokenType::BoolLiteral,
                    _ => WgslTokenType::Identifier,
                };

                state.add_token(kind, start_pos, state.get_position());
                return true;
            }
        }

        false
    }

    /// Lexes a punctuation mark.
    fn lex_punctuation<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let start_pos = state.get_position();

        if let Some(ch) = state.peek() {
            let kind = match ch {
                '(' => WgslTokenType::LeftParen,
                ')' => WgslTokenType::RightParen,
                '{' => WgslTokenType::LeftBrace,
                '}' => WgslTokenType::RightBrace,
                '[' => WgslTokenType::LeftBracket,
                ']' => WgslTokenType::RightBracket,
                ';' => WgslTokenType::Semicolon,
                ',' => WgslTokenType::Comma,
                '.' => WgslTokenType::Dot,
                ':' => WgslTokenType::Colon,
                '?' => WgslTokenType::Question,
                '@' => WgslTokenType::At,
                '#' => WgslTokenType::Hash,
                '$' => WgslTokenType::Dollar,
                _ => return false,
            };

            state.advance(ch.len_utf8());
            state.add_token(kind, start_pos, state.get_position());
            return true;
        }

        false
    }

    /// Lexes an operator.
    fn lex_operator<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let start_pos = state.get_position();

        // Generic closers must stay separable, so `>>` is emitted as two `Gt`
        // tokens. The compound `>>=` stays a single assignment operator.
        if state.peek() == Some('>') && state.peek_next_n(1) == Some('>') {
            if state.peek_next_n(2) == Some('=') {
                state.advance(3);
                state.add_token(WgslTokenType::RightShiftAssign, start_pos, state.get_position());
            }
            else {
                state.advance(2);
                state.add_token(WgslTokenType::Gt, start_pos, start_pos + 1);
                state.add_token(WgslTokenType::Gt, start_pos + 1, start_pos + 2);
            }
            return true;
        }

        if let Some(ch) = state.peek() {
            let kind = match ch {
                '=' => {
                    state.advance(1);
                    if let Some('=') = state.peek() {
                        state.advance(1);
                        WgslTokenType::EqEq
                    }
                    else {
                        WgslTokenType::Assign
                    }
                }
                '!' => {
                    state.advance(1);
                    if let Some('=') = state.peek() {
                        state.advance(1);
                        WgslTokenType::BangEq
                    }
                    else {
                        WgslTokenType::Bang
                    }
                }
                '<' => {
                    state.advance(1);
                    if let Some('=') = state.peek() {
                        state.advance(1);
                        WgslTokenType::Le
                    }
                    else if let Some('<') = state.peek() {
                        state.advance(1);
                        if let Some('=') = state.peek() {
                            state.advance(1);
                            WgslTokenType::LeftShiftAssign
                        }
                        else {
                            WgslTokenType::LeftShift
                        }
                    }
                    else {
                        WgslTokenType::Lt
                    }
                }
                // `>` is always lexed as a single token so that nested generic
                // argument lists such as `array<array<f32, 4>, 4>` can be closed
                // unambiguously. Shift operators are recognised by the parser
                // from two consecutive `>` tokens.
                '>' => {
                    state.advance(1);
                    if let Some('=') = state.peek() {
                        state.advance(1);
                        WgslTokenType::Ge
                    }
                    else {
                        WgslTokenType::Gt
                    }
                }
                '+' => {
                    state.advance(1);
                    if let Some('=') = state.peek() {
                        state.advance(1);
                        WgslTokenType::PlusAssign
                    }
                    else {
                        WgslTokenType::Plus
                    }
                }
                '-' => {
                    state.advance(1);
                    if let Some('=') = state.peek() {
                        state.advance(1);
                        WgslTokenType::MinusAssign
                    }
                    else if let Some('>') = state.peek() {
                        state.advance(1);
                        WgslTokenType::Arrow
                    }
                    else {
                        WgslTokenType::Minus
                    }
                }
                '*' => {
                    state.advance(1);
                    if let Some('=') = state.peek() {
                        state.advance(1);
                        WgslTokenType::StarAssign
                    }
                    else {
                        WgslTokenType::Star
                    }
                }
                '/' => {
                    state.advance(1);
                    if let Some('=') = state.peek() {
                        state.advance(1);
                        WgslTokenType::SlashAssign
                    }
                    else {
                        WgslTokenType::Slash
                    }
                }
                '%' => {
                    state.advance(1);
                    if let Some('=') = state.peek() {
                        state.advance(1);
                        WgslTokenType::PercentAssign
                    }
                    else {
                        WgslTokenType::Percent
                    }
                }
                '&' => {
                    state.advance(1);
                    if let Some('&') = state.peek() {
                        state.advance(1);
                        WgslTokenType::AmpersandAmpersand
                    }
                    else if let Some('=') = state.peek() {
                        state.advance(1);
                        WgslTokenType::AmpersandAssign
                    }
                    else {
                        WgslTokenType::Ampersand
                    }
                }
                '|' => {
                    state.advance(1);
                    if let Some('|') = state.peek() {
                        state.advance(1);
                        WgslTokenType::PipePipe
                    }
                    else if let Some('=') = state.peek() {
                        state.advance(1);
                        WgslTokenType::PipeAssign
                    }
                    else {
                        WgslTokenType::Pipe
                    }
                }
                '^' => {
                    state.advance(1);
                    if let Some('=') = state.peek() {
                        state.advance(1);
                        WgslTokenType::CaretAssign
                    }
                    else {
                        WgslTokenType::Caret
                    }
                }
                '~' => {
                    state.advance(1);
                    WgslTokenType::Tilde
                }
                _ => return false,
            };

            state.add_token(kind, start_pos, state.get_position());
            return true;
        }

        false
    }

    /// Lexes plain text.
    fn lex_text<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let start_pos = state.get_position();

        if let Some(ch) = state.peek() {
            state.advance(ch.len_utf8());
            state.add_token(WgslTokenType::Text, start_pos, state.get_position());
            true
        }
        else {
            false
        }
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

            if self.lex_string(state) {
                continue;
            }

            if self.lex_number(state) {
                continue;
            }

            if self.lex_identifier_or_keyword(state) {
                continue;
            }

            if self.lex_operator(state) {
                continue;
            }

            if self.lex_punctuation(state) {
                continue;
            }

            if self.lex_text(state) {
                continue;
            }

            let start_pos = state.get_position();
            if let Some(ch) = state.peek() {
                state.advance(ch.len_utf8());
                state.add_token(WgslTokenType::Error, start_pos, state.get_position());
            }
        }

        Ok(())
    }
}

impl<'config> Lexer<WgslLanguage> for WgslLexer<'config> {
    fn lex<'a, S: Source + ?Sized>(&self, source: &'a S, _edits: &[TextEdit], cache: &'a mut impl LexerCache<WgslLanguage>) -> LexOutput<WgslLanguage> {
        let mut state = LexerState::new(source);
        let result = self.run(&mut state);
        if result.is_ok() {
            state.add_eof();
        }
        state.finish_with_cache(result, cache)
    }
}
