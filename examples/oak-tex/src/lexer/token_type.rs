use oak_core::{Token, TokenType, UniversalTokenRole};

/// A token in a TeX source file.
pub type TexToken = Token<TexTokenType>;

/// TeX token types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum TexTokenType {
    /// Root node of the TeX file.
    Root,
    /// Source file.
    SourceFile,
    /// Document environment.
    Document,

    /// Generic command.
    Command,
    /// Generic environment.
    Environment,
    /// Start of an environment (\begin).
    BeginEnvironment,
    /// End of an environment (\end).
    EndEnvironment,

    /// Math mode content.
    MathMode,
    /// Inline math mode ($...$ or \(...\)).
    InlineMath,
    /// Display math mode ($$...$$ or \[...\]).
    DisplayMath,
    /// \( or \) inline math delimiter.
    ParenInlineMathDelim,
    /// \[ or \] display math delimiter.
    BracketDisplayMathDelim,
    /// \{ or \} brace delimiter command.
    BraceDelim,
    /// Grouped content ({...}).
    Group,
    /// Superscript (^).
    Superscript,
    /// Subscript (_).
    Subscript,

    /// Generic argument.
    Argument,
    /// Optional argument ([...]).
    OptionalArgument,
    /// Mandatory argument ({...}).
    MandatoryArgument,

    /// Text content.
    Text,
    /// Paragraph.
    Paragraph,
    /// Section.
    Section,
    /// Subsection.
    Subsection,
    /// Subsubsection.
    Subsubsection,

    /// List environment.
    List,
    /// Item in a list (\item).
    Item,
    /// Table environment.
    Table,
    /// Row in a table.
    Row,
    /// Cell in a table.
    Cell,

    /// Label definition (\label).
    Label,
    /// Reference to a label (\ref).
    Reference,
    /// Citation (\cite).
    Citation,

    /// Figure environment.
    Figure,
    /// Caption (\caption).
    Caption,

    /// Error token.
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
    /// \alpha Greek letter.
    Alpha,
    /// \beta Greek letter.
    Beta,
    /// \gamma Greek letter.
    Gamma,
    /// \delta Greek letter.
    Delta,
    /// \epsilon Greek letter.
    Epsilon,
    /// \zeta Greek letter.
    Zeta,
    /// \eta Greek letter.
    Eta,
    /// \theta Greek letter.
    Theta,
    /// \iota Greek letter.
    Iota,
    /// \kappa Greek letter.
    Kappa,
    /// \lambda Greek letter.
    Lambda,
    /// \mu Greek letter.
    Mu,
    /// \nu Greek letter.
    Nu,
    /// \xi Greek letter.
    Xi,
    /// \omicron Greek letter.
    Omicron,
    /// \pi Greek letter.
    Pi,
    /// \rho Greek letter.
    Rho,
    /// \sigma Greek letter.
    Sigma,
    /// \tau Greek letter.
    Tau,
    /// \upsilon Greek letter.
    Upsilon,
    /// \phi Greek letter.
    Phi,
    /// \chi Greek letter.
    Chi,
    /// \psi Greek letter.
    Psi,
    /// \omega Greek letter.
    Omega,
    /// \varepsilon Greek letter.
    VarEpsilon,
    /// \vartheta Greek letter.
    VarTheta,
    /// \varkappa Greek letter.
    VarKappa,
    /// \varpi Greek letter.
    VarPi,
    /// \varrho Greek letter.
    VarRho,
    /// \varsigma Greek letter.
    VarSigma,
    /// \varphi Greek letter.
    VarPhi,
    /// \Gamma Greek letter.
    UpperGamma,
    /// \Delta Greek letter.
    UpperDelta,
    /// \Theta Greek letter.
    UpperTheta,
    /// \Lambda Greek letter.
    UpperLambda,
    /// \Xi Greek letter.
    UpperXi,
    /// \Pi Greek letter.
    UpperPi,
    /// \Sigma Greek letter.
    UpperSigma,
    /// \Upsilon Greek letter.
    UpperUpsilon,
    /// \Phi Greek letter.
    UpperPhi,
    /// \Psi Greek letter.
    UpperPsi,
    /// \Omega Greek letter.
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
    /// \left command.
    LeftCommand,
    /// \right command.
    RightCommand,
    /// \middle command.
    MiddleCommand,
    /// \limits command.
    Limits,
    /// \nolimits command.
    Nolimits,
    /// \underline command.
    Underline,
    /// CST node for `\left ... \middle ... \right` paired delimiters.
    LeftRightPair,

    /// Identifier.
    Identifier,
    /// String literal.
    StringLiteral,
    /// Numeric literal.
    Number,

    /// Backslash (\).
    Backslash,
    /// Row break (`\\`) inside an environment body.
    RowBreak,
    /// Left brace ({).
    LeftBrace,
    /// Right brace (}).
    RightBrace,
    /// Left bracket ([).
    LeftBracket,
    /// Right bracket (]).
    RightBracket,
    /// Left parenthesis (().
    LeftParen,
    /// Right parenthesis ()).
    RightParen,
    /// Dollar sign ($).
    Dollar,
    /// Double dollar sign ($$).
    DoubleDollar,
    /// Ampersand (&).
    Ampersand,
    /// Percent sign (%).
    Percent,
    /// Hash sign (#).
    Hash,
    /// Caret sign (^).
    Caret,
    /// Underscore (_).
    Underscore,
    /// Tilde (~).
    Tilde,

    /// Equal sign (=).
    Equal,
    /// Equals sign (alternative).
    Equals,
    /// Plus sign (+).
    Plus,
    /// Minus sign (-).
    Minus,
    /// Star sign (*).
    Star,
    /// Slash sign (/).
    Slash,
    /// Pipe sign (|).
    Pipe,
    /// Less than sign (<).
    Less,
    /// Less than sign (alternative).
    LessThan,
    /// Greater than sign (>).
    Greater,
    /// Greater than sign (alternative).
    GreaterThan,
    /// Exclamation mark (!).
    Exclamation,
    /// Question mark (?).
    Question,
    /// At sign (@).
    At,
    /// Colon (:).
    Colon,
    /// Semicolon (;).
    Semicolon,
    /// Comma (,).
    Comma,
    /// Dot (.).
    Dot,

    /// Comment (% ...).
    Comment,
    /// Whitespace.
    Whitespace,
    /// Newline.
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
    /// 'textbf' keyword.
    TextbfKeyword,
    /// 'textit' keyword.
    TextitKeyword,
    /// 'emph' keyword.
    EmphKeyword,
    /// 'left' keyword.
    LeftKeyword,
    /// 'right' keyword.
    RightKeyword,
    /// 'middle' keyword.
    MiddleKeyword,
    /// 'limits' keyword.
    LimitsKeyword,
    /// 'nolimits' keyword.
    NolimitsKeyword,

    /// End of stream.
    Eof,
}

impl TokenType for TexTokenType {
    type Role = UniversalTokenRole;
    const END_OF_STREAM: Self = Self::Eof;

    fn is_ignored(&self) -> bool {
        matches!(self, Self::Whitespace | Self::Newline | Self::Comment)
    }

    fn role(&self) -> Self::Role {
        match self {
            Self::Whitespace | Self::Newline => UniversalTokenRole::Whitespace,
            Self::Comment => UniversalTokenRole::Comment,
            Self::BeginKeyword
            | Self::EndKeyword
            | Self::DocumentclassKeyword
            | Self::UsepackageKeyword
            | Self::SectionKeyword
            | Self::SubsectionKeyword
            | Self::SubsubsectionKeyword
            | Self::ChapterKeyword
            | Self::PartKeyword
            | Self::TitleKeyword
            | Self::AuthorKeyword
            | Self::DateKeyword
            | Self::MaketitleKeyword
            | Self::TableofcontentsKeyword
            | Self::ItemKeyword
            | Self::LabelKeyword
            | Self::RefKeyword
            | Self::CiteKeyword
            | Self::IncludegraphicsKeyword
            | Self::TextbfKeyword
            | Self::TextitKeyword
            | Self::EmphKeyword
            | Self::LeftKeyword
            | Self::RightKeyword
            | Self::MiddleKeyword
            | Self::LimitsKeyword
            | Self::NolimitsKeyword => UniversalTokenRole::Keyword,
            Self::Number | Self::StringLiteral => UniversalTokenRole::Literal,
            Self::Backslash
            | Self::RowBreak
            | Self::Plus
            | Self::Minus
            | Self::Star
            | Self::Slash
            | Self::Pipe
            | Self::Less
            | Self::LessThan
            | Self::Greater
            | Self::GreaterThan
            | Self::Equal
            | Self::Equals
            | Self::Caret
            | Self::Underscore
            | Self::Tilde
            | Self::Dollar
            | Self::DoubleDollar
            | Self::Ampersand
            | Self::Hash
            | Self::At => UniversalTokenRole::Operator,
            Self::LeftBrace | Self::RightBrace | Self::LeftBracket | Self::RightBracket | Self::LeftParen | Self::RightParen | Self::Comma | Self::Dot | Self::Colon | Self::Semicolon | Self::Exclamation | Self::Question | Self::Percent => {
                UniversalTokenRole::Punctuation
            }
            Self::Identifier | Self::Text => UniversalTokenRole::Name,
            Self::Eof => UniversalTokenRole::Eof,
            Self::Error => UniversalTokenRole::Error,
            _ => UniversalTokenRole::None,
        }
    }
}

impl From<crate::parser::element_type::TexElementType> for TexTokenType {
    fn from(element: crate::parser::element_type::TexElementType) -> Self {
        use crate::parser::element_type::TexElementType as E;
        match element {
            E::Root => Self::Root,
            E::SourceFile => Self::SourceFile,
            E::Document => Self::Document,
            E::Command => Self::Command,
            E::Environment => Self::Environment,
            E::BeginEnvironment => Self::BeginEnvironment,
            E::EndEnvironment => Self::EndEnvironment,
            E::MathMode => Self::MathMode,
            E::InlineMath => Self::InlineMath,
            E::DisplayMath => Self::DisplayMath,
            E::ParenInlineMath => Self::ParenInlineMathDelim,
            E::BracketDisplayMath => Self::BracketDisplayMathDelim,
            E::Group => Self::Group,
            E::Superscript => Self::Superscript,
            E::Subscript => Self::Subscript,
            E::Argument => Self::Argument,
            E::OptionalArgument => Self::OptionalArgument,
            E::MandatoryArgument => Self::MandatoryArgument,
            E::Text => Self::Text,
            E::Paragraph => Self::Paragraph,
            E::Section => Self::Section,
            E::Subsection => Self::Subsection,
            E::Subsubsection => Self::Subsubsection,
            E::List => Self::List,
            E::Item => Self::Item,
            E::Table => Self::Table,
            E::Row => Self::Row,
            E::Cell => Self::Cell,
            E::Label => Self::Label,
            E::Reference => Self::Reference,
            E::Citation => Self::Citation,
            E::Figure => Self::Figure,
            E::Caption => Self::Caption,
            E::Error => Self::Error,
            E::DocumentClass => Self::DocumentClass,
            E::UsePackage => Self::UsePackage,
            E::Begin => Self::Begin,
            E::End => Self::End,
            E::Section_ => Self::Section_,
            E::Subsection_ => Self::Subsection_,
            E::Subsubsection_ => Self::Subsubsection_,
            E::Chapter => Self::Chapter,
            E::Part => Self::Part,
            E::Title => Self::Title,
            E::Author => Self::Author,
            E::Date => Self::Date,
            E::MakeTitle => Self::MakeTitle,
            E::TableOfContents => Self::TableOfContents,
            E::NewPage => Self::NewPage,
            E::ClearPage => Self::ClearPage,
            E::Frac => Self::Frac,
            E::Sqrt => Self::Sqrt,
            E::Sum => Self::Sum,
            E::Int => Self::Int,
            E::Lim => Self::Lim,
            E::Alpha => Self::Alpha,
            E::Beta => Self::Beta,
            E::Gamma => Self::Gamma,
            E::Delta => Self::Delta,
            E::Epsilon => Self::Epsilon,
            E::Zeta => Self::Zeta,
            E::Eta => Self::Eta,
            E::Theta => Self::Theta,
            E::Iota => Self::Iota,
            E::Kappa => Self::Kappa,
            E::Lambda => Self::Lambda,
            E::Mu => Self::Mu,
            E::Nu => Self::Nu,
            E::Xi => Self::Xi,
            E::Omicron => Self::Omicron,
            E::Pi => Self::Pi,
            E::Rho => Self::Rho,
            E::Sigma => Self::Sigma,
            E::Tau => Self::Tau,
            E::Upsilon => Self::Upsilon,
            E::Phi => Self::Phi,
            E::Chi => Self::Chi,
            E::Psi => Self::Psi,
            E::Omega => Self::Omega,
            E::VarEpsilon => Self::VarEpsilon,
            E::VarTheta => Self::VarTheta,
            E::VarKappa => Self::VarKappa,
            E::VarPi => Self::VarPi,
            E::VarRho => Self::VarRho,
            E::VarSigma => Self::VarSigma,
            E::VarPhi => Self::VarPhi,
            E::UpperGamma => Self::UpperGamma,
            E::UpperDelta => Self::UpperDelta,
            E::UpperTheta => Self::UpperTheta,
            E::UpperLambda => Self::UpperLambda,
            E::UpperXi => Self::UpperXi,
            E::UpperPi => Self::UpperPi,
            E::UpperSigma => Self::UpperSigma,
            E::UpperUpsilon => Self::UpperUpsilon,
            E::UpperPhi => Self::UpperPhi,
            E::UpperPsi => Self::UpperPsi,
            E::UpperOmega => Self::UpperOmega,
            E::TextBf => Self::TextBf,
            E::TextIt => Self::TextIt,
            E::TextSc => Self::TextSc,
            E::TextTt => Self::TextTt,
            E::Emph => Self::Emph,
            E::Underline => Self::Underline,
            E::LeftRightPair => Self::LeftRightPair,
            E::Limits => Self::Limits,
            E::Nolimits => Self::Nolimits,
            E::Identifier => Self::Identifier,
            E::StringLiteral => Self::StringLiteral,
            E::Number => Self::Number,
            E::Backslash => Self::Backslash,
            E::LeftBrace => Self::LeftBrace,
            E::RightBrace => Self::RightBrace,
            E::LeftBracket => Self::LeftBracket,
            E::RightBracket => Self::RightBracket,
            E::LeftParen => Self::LeftParen,
            E::RightParen => Self::RightParen,
            E::Dollar => Self::Dollar,
            E::DoubleDollar => Self::DoubleDollar,
            E::Ampersand => Self::Ampersand,
            E::Percent => Self::Percent,
            E::Hash => Self::Hash,
            E::Caret => Self::Caret,
            E::Underscore => Self::Underscore,
            E::Tilde => Self::Tilde,
            E::Equal => Self::Equal,
            E::Equals => Self::Equals,
            E::Plus => Self::Plus,
            E::Minus => Self::Minus,
            E::Star => Self::Star,
            E::Slash => Self::Slash,
            E::Pipe => Self::Pipe,
            E::Less => Self::Less,
            E::LessThan => Self::LessThan,
            E::Greater => Self::Greater,
            E::GreaterThan => Self::GreaterThan,
            E::Exclamation => Self::Exclamation,
            E::Question => Self::Question,
            E::At => Self::At,
            E::Colon => Self::Colon,
            E::Semicolon => Self::Semicolon,
            E::Comma => Self::Comma,
            E::Dot => Self::Dot,
            E::Comment => Self::Comment,
            E::Whitespace => Self::Whitespace,
            E::Newline => Self::Newline,
            E::BeginKeyword => Self::BeginKeyword,
            E::EndKeyword => Self::EndKeyword,
            E::DocumentclassKeyword => Self::DocumentclassKeyword,
            E::UsepackageKeyword => Self::UsepackageKeyword,
            E::SectionKeyword => Self::SectionKeyword,
            E::SubsectionKeyword => Self::SubsectionKeyword,
            E::SubsubsectionKeyword => Self::SubsubsectionKeyword,
            E::ChapterKeyword => Self::ChapterKeyword,
            E::PartKeyword => Self::PartKeyword,
            E::TitleKeyword => Self::TitleKeyword,
            E::AuthorKeyword => Self::AuthorKeyword,
            E::DateKeyword => Self::DateKeyword,
            E::MaketitleKeyword => Self::MaketitleKeyword,
            E::TableofcontentsKeyword => Self::TableofcontentsKeyword,
            E::ItemKeyword => Self::ItemKeyword,
            E::LabelKeyword => Self::LabelKeyword,
            E::RefKeyword => Self::RefKeyword,
            E::CiteKeyword => Self::CiteKeyword,
            E::IncludegraphicsKeyword => Self::IncludegraphicsKeyword,
            E::TextbfKeyword => Self::TextbfKeyword,
            E::TextitKeyword => Self::TextitKeyword,
            E::EmphKeyword => Self::EmphKeyword,
            E::LeftKeyword => Self::LeftKeyword,
            E::RightKeyword => Self::RightKeyword,
            E::MiddleKeyword => Self::MiddleKeyword,
            E::LimitsKeyword => Self::LimitsKeyword,
            E::NolimitsKeyword => Self::NolimitsKeyword,
            E::Eof => Self::Eof,
        }
    }
}
