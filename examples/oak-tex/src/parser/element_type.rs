use oak_core::{ElementType, UniversalElementRole};

/// Represents an element type in a TeX document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum TexElementType {
    /// Root node of the AST.
    Root,
    /// A source file containing TeX content.
    SourceFile,
    /// The document body.
    Document,

    /// A TeX command (e.g., \section).
    Command,
    /// A TeX environment (e.g., \begin{itemize}...\end{itemize}).
    Environment,
    /// The beginning of an environment.
    BeginEnvironment,
    /// The end of an environment.
    EndEnvironment,

    /// Mathematical content.
    MathMode,
    /// Inline mathematical content ($...$).
    InlineMath,
    /// Displayed mathematical content ($$...$$).
    DisplayMath,
    /// Inline math delimited by `\(...\)`.
    ParenInlineMath,
    /// Display math delimited by `\[...\]`.
    BracketDisplayMath,
    /// A grouped set of elements ({...}).
    Group,
    /// A superscript expression (^...).
    Superscript,
    /// A subscript expression (_...).
    Subscript,

    /// A generic command argument.
    Argument,
    /// An optional command argument ([...]).
    OptionalArgument,
    /// A mandatory command argument ({...}).
    MandatoryArgument,

    /// Plain text content.
    Text,
    /// A paragraph of text.
    Paragraph,
    /// A section heading.
    Section,
    /// A subsection heading.
    Subsection,
    /// A subsubsection heading.
    Subsubsection,

    /// A list environment.
    List,
    /// An item within a list.
    Item,
    /// A table environment.
    Table,
    /// A row within a table.
    Row,
    /// A cell within a table row.
    Cell,

    /// A label for cross-referencing.
    Label,
    /// A reference to a label.
    Reference,
    /// A bibliographic citation.
    Citation,

    /// A figure environment.
    Figure,
    /// A caption for a figure or table.
    Caption,

    /// An error node representing a syntax error.
    Error,

    /// \documentclass command.
    DocumentClass,
    /// \usepackage command.
    UsePackage,
    /// \begin command.
    Begin,
    /// \end command.
    End,
    /// \section command.
    Section_,
    /// \subsection command.
    Subsection_,
    /// \subsubsection command.
    Subsubsection_,
    /// \chapter command.
    Chapter,
    /// \part command.
    Part,
    /// \title command.
    Title,
    /// \author command.
    Author,
    /// \date command.
    Date,
    /// \maketitle command.
    MakeTitle,
    /// \tableofcontents command.
    TableOfContents,
    /// \newpage command.
    NewPage,
    /// \clearpage command.
    ClearPage,

    /// \frac command.
    Frac,
    /// \sqrt command.
    Sqrt,
    /// \sum command.
    Sum,
    /// \int command.
    Int,
    /// \lim command.
    Lim,
    /// \alpha command.
    Alpha,
    /// \beta command.
    Beta,
    /// \gamma command.
    Gamma,
    /// \delta command.
    Delta,
    /// \epsilon command.
    Epsilon,
    /// \zeta command.
    Zeta,
    /// \eta command.
    Eta,
    /// \theta command.
    Theta,
    /// \iota command.
    Iota,
    /// \kappa command.
    Kappa,
    /// \lambda command.
    Lambda,
    /// \mu command.
    Mu,
    /// \nu command.
    Nu,
    /// \xi command.
    Xi,
    /// \omicron command.
    Omicron,
    /// \pi command.
    Pi,
    /// \rho command.
    Rho,
    /// \sigma command.
    Sigma,
    /// \tau command.
    Tau,
    /// \upsilon command.
    Upsilon,
    /// \phi command.
    Phi,
    /// \chi command.
    Chi,
    /// \psi command.
    Psi,
    /// \omega command.
    Omega,
    /// \varepsilon command.
    VarEpsilon,
    /// \vartheta command.
    VarTheta,
    /// \varkappa command.
    VarKappa,
    /// \varpi command.
    VarPi,
    /// \varrho command.
    VarRho,
    /// \varsigma command.
    VarSigma,
    /// \varphi command.
    VarPhi,
    /// \Gamma command.
    UpperGamma,
    /// \Delta command.
    UpperDelta,
    /// \Theta command.
    UpperTheta,
    /// \Lambda command.
    UpperLambda,
    /// \Xi command.
    UpperXi,
    /// \Pi command.
    UpperPi,
    /// \Sigma command.
    UpperSigma,
    /// \Upsilon command.
    UpperUpsilon,
    /// \Phi command.
    UpperPhi,
    /// \Psi command.
    UpperPsi,
    /// \Omega command.
    UpperOmega,

    /// \textbf command.
    TextBf,
    /// \textit command.
    TextIt,
    /// \textsc command.
    TextSc,
    /// \texttt command.
    TextTt,
    /// \emph command.
    Emph,
    /// \underline command.
    Underline,
    /// \left ... \right paired delimiter.
    LeftRightPair,
    /// \limits command.
    Limits,
    /// \nolimits command.
    Nolimits,

    /// An identifier.
    Identifier,
    /// A string literal.
    StringLiteral,
    /// A numeric literal.
    Number,

    /// Backslash character (\).
    Backslash,
    /// Left brace character ({).
    LeftBrace,
    /// Right brace character (}).
    RightBrace,
    /// Left bracket character ([).
    LeftBracket,
    /// Right bracket character (]).
    RightBracket,
    /// Left parenthesis character (().
    LeftParen,
    /// Right parenthesis character ()).
    RightParen,
    /// Dollar sign character ($).
    Dollar,
    /// Double dollar sign character ($$).
    DoubleDollar,
    /// Ampersand character (&).
    Ampersand,
    /// Percent character (%).
    Percent,
    /// Hash character (#).
    Hash,
    /// Caret character (^).
    Caret,
    /// Underscore character (_).
    Underscore,
    /// Tilde character (~).
    Tilde,

    /// Equals character (=).
    Equal,
    /// Double equals character (==).
    Equals,
    /// Plus character (+).
    Plus,
    /// Minus character (-).
    Minus,
    /// Asterisk character (*).
    Star,
    /// Slash character (/).
    Slash,
    /// Pipe character (|).
    Pipe,
    /// Less than character (<).
    Less,
    /// Less than or equal to character (<=).
    LessThan,
    /// Greater than character (>).
    Greater,
    /// Greater than or equal to character (>=).
    GreaterThan,
    /// Exclamation mark character (!).
    Exclamation,
    /// Question mark character (?).
    Question,
    /// At sign character (@).
    At,
    /// Colon character (:).
    Colon,
    /// Semicolon character (;).
    Semicolon,
    /// Comma character (,).
    Comma,
    /// Dot character (.).
    Dot,

    /// A comment starting with %.
    Comment,
    /// Whitespace characters.
    Whitespace,
    /// A newline character.
    Newline,

    /// 'begin' keyword.
    BeginKeyword,
    /// 'end' keyword.
    EndKeyword,
    /// 'documentclass' keyword.
    DocumentclassKeyword,
    /// 'usepackage' keyword.
    UsepackageKeyword,
    /// 'section' keyword.
    SectionKeyword,
    /// 'subsection' keyword.
    SubsectionKeyword,
    /// 'subsubsection' keyword.
    SubsubsectionKeyword,
    /// 'chapter' keyword.
    ChapterKeyword,
    /// 'part' keyword.
    PartKeyword,
    /// 'title' keyword.
    TitleKeyword,
    /// 'author' keyword.
    AuthorKeyword,
    /// 'date' keyword.
    DateKeyword,
    /// 'maketitle' keyword.
    MaketitleKeyword,
    /// 'tableofcontents' keyword.
    TableofcontentsKeyword,
    /// 'item' keyword.
    ItemKeyword,
    /// 'label' keyword.
    LabelKeyword,
    /// 'ref' keyword.
    RefKeyword,
    /// 'cite' keyword.
    CiteKeyword,
    /// 'includegraphics' keyword.
    IncludegraphicsKeyword,
    /// \textbf keyword.
    TextbfKeyword,
    /// \textit keyword.
    TextitKeyword,
    /// \emph keyword.
    EmphKeyword,
    /// \left keyword.
    LeftKeyword,
    /// \right keyword.
    RightKeyword,
    /// \middle keyword.
    MiddleKeyword,
    /// \limits keyword.
    LimitsKeyword,
    /// \nolimits keyword.
    NolimitsKeyword,

    /// End of file marker.
    Eof,
}

impl ElementType for TexElementType {
    type Role = UniversalElementRole;

    fn role(&self) -> Self::Role {
        match self {
            _ => UniversalElementRole::None,
        }
    }
}

impl oak_core::language::TokenType for TexElementType {
    type Role = oak_core::UniversalTokenRole;
    const END_OF_STREAM: Self = Self::Eof;

    fn is_ignored(&self) -> bool {
        matches!(self, Self::Whitespace | Self::Newline | Self::Comment)
    }

    fn role(&self) -> Self::Role {
        match self {
            Self::Whitespace | Self::Newline => oak_core::UniversalTokenRole::Whitespace,
            Self::Comment => oak_core::UniversalTokenRole::Comment,
            Self::Eof => oak_core::UniversalTokenRole::Eof,
            Self::Error => oak_core::UniversalTokenRole::Error,
            _ => oak_core::UniversalTokenRole::None,
        }
    }
}

impl From<crate::lexer::token_type::TexTokenType> for TexElementType {
    fn from(token: crate::lexer::token_type::TexTokenType) -> Self {
        use crate::lexer::token_type::TexTokenType as T;
        match token {
            T::Root => Self::Root,
            T::SourceFile => Self::SourceFile,
            T::Document => Self::Document,
            T::Command => Self::Command,
            T::Environment => Self::Environment,
            T::BeginEnvironment => Self::BeginEnvironment,
            T::EndEnvironment => Self::EndEnvironment,
            T::MathMode => Self::MathMode,
            T::InlineMath => Self::InlineMath,
            T::DisplayMath => Self::DisplayMath,
            T::ParenInlineMathDelim => Self::ParenInlineMath,
            T::BracketDisplayMathDelim => Self::BracketDisplayMath,
            T::Group => Self::Group,
            T::Superscript => Self::Superscript,
            T::Subscript => Self::Subscript,
            T::Argument => Self::Argument,
            T::OptionalArgument => Self::OptionalArgument,
            T::MandatoryArgument => Self::MandatoryArgument,
            T::Text => Self::Text,
            T::Paragraph => Self::Paragraph,
            T::Section => Self::Section,
            T::Subsection => Self::Subsection,
            T::Subsubsection => Self::Subsubsection,
            T::List => Self::List,
            T::Item => Self::Item,
            T::Table => Self::Table,
            T::Row => Self::Row,
            T::Cell => Self::Cell,
            T::Label => Self::Label,
            T::Reference => Self::Reference,
            T::Citation => Self::Citation,
            T::Figure => Self::Figure,
            T::Caption => Self::Caption,
            T::Error => Self::Error,
            T::DocumentClass => Self::DocumentClass,
            T::UsePackage => Self::UsePackage,
            T::Begin => Self::Begin,
            T::End => Self::End,
            T::Section_ => Self::Section_,
            T::Subsection_ => Self::Subsection_,
            T::Subsubsection_ => Self::Subsubsection_,
            T::Chapter => Self::Chapter,
            T::Part => Self::Part,
            T::Title => Self::Title,
            T::Author => Self::Author,
            T::Date => Self::Date,
            T::MakeTitle => Self::MakeTitle,
            T::TableOfContents => Self::TableOfContents,
            T::NewPage => Self::NewPage,
            T::ClearPage => Self::ClearPage,
            T::Frac => Self::Frac,
            T::Sqrt => Self::Sqrt,
            T::Sum => Self::Sum,
            T::Int => Self::Int,
            T::Lim => Self::Lim,
            T::Alpha => Self::Alpha,
            T::Beta => Self::Beta,
            T::Gamma => Self::Gamma,
            T::Delta => Self::Delta,
            T::Epsilon => Self::Epsilon,
            T::Zeta => Self::Zeta,
            T::Eta => Self::Eta,
            T::Theta => Self::Theta,
            T::Iota => Self::Iota,
            T::Kappa => Self::Kappa,
            T::Lambda => Self::Lambda,
            T::Mu => Self::Mu,
            T::Nu => Self::Nu,
            T::Xi => Self::Xi,
            T::Omicron => Self::Omicron,
            T::Pi => Self::Pi,
            T::Rho => Self::Rho,
            T::Sigma => Self::Sigma,
            T::Tau => Self::Tau,
            T::Upsilon => Self::Upsilon,
            T::Phi => Self::Phi,
            T::Chi => Self::Chi,
            T::Psi => Self::Psi,
            T::Omega => Self::Omega,
            T::VarEpsilon => Self::VarEpsilon,
            T::VarTheta => Self::VarTheta,
            T::VarKappa => Self::VarKappa,
            T::VarPi => Self::VarPi,
            T::VarRho => Self::VarRho,
            T::VarSigma => Self::VarSigma,
            T::VarPhi => Self::VarPhi,
            T::UpperGamma => Self::UpperGamma,
            T::UpperDelta => Self::UpperDelta,
            T::UpperTheta => Self::UpperTheta,
            T::UpperLambda => Self::UpperLambda,
            T::UpperXi => Self::UpperXi,
            T::UpperPi => Self::UpperPi,
            T::UpperSigma => Self::UpperSigma,
            T::UpperUpsilon => Self::UpperUpsilon,
            T::UpperPhi => Self::UpperPhi,
            T::UpperPsi => Self::UpperPsi,
            T::UpperOmega => Self::UpperOmega,
            T::TextBf => Self::TextBf,
            T::TextIt => Self::TextIt,
            T::TextSc => Self::TextSc,
            T::TextTt => Self::TextTt,
            T::Emph => Self::Emph,
            T::Underline => Self::Underline,
            T::LeftCommand => Self::Command,
            T::RightCommand => Self::Command,
            T::MiddleCommand => Self::Command,
            T::Limits => Self::Limits,
            T::Nolimits => Self::Nolimits,
            T::BraceDelim => Self::Command,
            T::LeftRightPair => Self::LeftRightPair,
            T::Identifier => Self::Identifier,
            T::StringLiteral => Self::StringLiteral,
            T::Number => Self::Number,
            T::Backslash => Self::Backslash,
            T::LeftBrace => Self::LeftBrace,
            T::RightBrace => Self::RightBrace,
            T::LeftBracket => Self::LeftBracket,
            T::RightBracket => Self::RightBracket,
            T::LeftParen => Self::LeftParen,
            T::RightParen => Self::RightParen,
            T::Dollar => Self::Dollar,
            T::DoubleDollar => Self::DoubleDollar,
            T::Ampersand => Self::Ampersand,
            T::Percent => Self::Percent,
            T::Hash => Self::Hash,
            T::Caret => Self::Caret,
            T::Underscore => Self::Underscore,
            T::Tilde => Self::Tilde,
            T::Equal => Self::Equal,
            T::Equals => Self::Equals,
            T::Plus => Self::Plus,
            T::Minus => Self::Minus,
            T::Star => Self::Star,
            T::Slash => Self::Slash,
            T::Pipe => Self::Pipe,
            T::Less => Self::Less,
            T::LessThan => Self::LessThan,
            T::Greater => Self::Greater,
            T::GreaterThan => Self::GreaterThan,
            T::Exclamation => Self::Exclamation,
            T::Question => Self::Question,
            T::At => Self::At,
            T::Colon => Self::Colon,
            T::Semicolon => Self::Semicolon,
            T::Comma => Self::Comma,
            T::Dot => Self::Dot,
            T::Comment => Self::Comment,
            T::Whitespace => Self::Whitespace,
            T::Newline => Self::Newline,
            T::BeginKeyword => Self::BeginKeyword,
            T::EndKeyword => Self::EndKeyword,
            T::DocumentclassKeyword => Self::DocumentclassKeyword,
            T::UsepackageKeyword => Self::UsepackageKeyword,
            T::SectionKeyword => Self::SectionKeyword,
            T::SubsectionKeyword => Self::SubsectionKeyword,
            T::SubsubsectionKeyword => Self::SubsubsectionKeyword,
            T::ChapterKeyword => Self::ChapterKeyword,
            T::PartKeyword => Self::PartKeyword,
            T::TitleKeyword => Self::TitleKeyword,
            T::AuthorKeyword => Self::AuthorKeyword,
            T::DateKeyword => Self::DateKeyword,
            T::MaketitleKeyword => Self::MaketitleKeyword,
            T::TableofcontentsKeyword => Self::TableofcontentsKeyword,
            T::ItemKeyword => Self::ItemKeyword,
            T::LabelKeyword => Self::LabelKeyword,
            T::RefKeyword => Self::RefKeyword,
            T::CiteKeyword => Self::CiteKeyword,
            T::IncludegraphicsKeyword => Self::IncludegraphicsKeyword,
            T::TextbfKeyword => Self::TextbfKeyword,
            T::TextitKeyword => Self::TextitKeyword,
            T::EmphKeyword => Self::EmphKeyword,
            T::LeftKeyword => Self::Command,
            T::RightKeyword => Self::Command,
            T::MiddleKeyword => Self::Command,
            T::LimitsKeyword => Self::Limits,
            T::NolimitsKeyword => Self::Nolimits,
            T::Eof => Self::Eof,
        }
    }
}
