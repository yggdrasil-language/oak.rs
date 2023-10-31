use oak_core::{Token, TokenType, UniversalTokenRole};

/// A token in the XML language.
pub type XmlToken = Token<XmlTokenType>;

impl TokenType for XmlTokenType {
    type Role = UniversalTokenRole;
    const END_OF_STREAM: Self = Self::Error;

    fn is_ignored(&self) -> bool {
        matches!(self, Self::Whitespace | Self::Newline | Self::Comment | Self::ProcessingInstruction)
    }

    fn role(&self) -> Self::Role {
        match self {
            _ => UniversalTokenRole::None,
        }
    }
}

/// XML token types.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum XmlTokenType {
    /// Root element.
    Root,
    /// Whitespace characters.
    Whitespace,
    /// Newline character.
    Newline,
    /// Comment.
    Comment,
    /// Text content.
    Text,
    /// Error token.
    Error,
    /// End of file.
    Eof,

    /// XML declaration.
    XmlDeclaration,
    /// Document type declaration.
    DoctypeDeclaration,
    /// Processing instruction.
    ProcessingInstruction,
    /// CDATA section.
    CData,

    /// Start tag.
    StartTag,
    /// End tag.
    EndTag,
    /// Self-closing tag.
    SelfClosingTag,
    /// Tag name.
    TagName,

    /// Attribute name.
    AttributeName,
    /// Attribute value.
    AttributeValue,

    /// String literal.
    StringLiteral,

    /// `<` operator.
    LeftAngle,
    /// `>` operator.
    RightAngle,
    /// `</` operator.
    LeftAngleSlash,
    /// `/>` operator.
    SlashRightAngle,
    /// `=` operator.
    Equals,
    /// `"` quote.
    Quote,
    /// `'` single quote.
    SingleQuote,
    /// `!` exclamation.
    Exclamation,
    /// `?` question.
    Question,
    /// `&` ampersand.
    Ampersand,
    /// `;` semicolon.
    Semicolon,

    /// Entity reference.
    EntityReference,
    /// Character reference.
    CharacterReference,

    /// Identifier.
    Identifier,

    /// Source file non-terminal.
    SourceFile,
    /// Element non-terminal.
    Element,
    /// Attribute non-terminal.
    Attribute,
    /// Prolog non-terminal.
    Prolog,
}
