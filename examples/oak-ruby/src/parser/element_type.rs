use oak_core::{ElementType, UniversalElementRole};
use std::fmt;

/// Element types for Ruby.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum RubyElementType {
    /// An identifier.
    Identifier,
    /// A global variable (e.g., `$stdout`).
    GlobalVariable,
    /// An instance variable (e.g., `@name`).
    InstanceVariable,
    /// A class variable (e.g., `@@name`).
    ClassVariable,
    /// A constant (e.g., `MATH`).
    Constant,
    /// An integer literal.
    IntegerLiteral,
    /// A float literal.
    FloatLiteral,
    /// A string literal.
    StringLiteral,
    /// A general literal.
    Literal,
    /// A symbol (e.g., `:name`).
    Symbol,
    /// A regular expression literal.
    RegexLiteral,

    /// The `if` keyword.
    If,
    /// The `unless` keyword.
    Unless,
    /// The `elsif` keyword.
    Elsif,
    /// The `else` keyword.
    Else,
    /// The `case` keyword.
    Case,
    /// The `when` keyword.
    When,
    /// The `then` keyword.
    Then,
    /// The `for` keyword.
    For,
    /// The `while` keyword.
    While,
    /// The `until` keyword.
    Until,
    /// The `break` keyword.
    Break,
    /// The `next` keyword.
    Next,
    /// The `redo` keyword.
    Redo,
    /// The `retry` keyword.
    Retry,
    /// The `return` keyword.
    Return,
    /// The `yield` keyword.
    Yield,
    /// The `def` keyword.
    Def,
    /// The `class` keyword.
    Class,
    /// The `module` keyword.
    Module,
    /// The `end` keyword.
    End,
    /// The `lambda` keyword.
    Lambda,
    /// The `proc` keyword.
    Proc,
    /// The `begin` keyword.
    Begin,
    /// The `rescue` keyword.
    Rescue,
    /// The `ensure` keyword.
    Ensure,
    /// The `raise` keyword.
    Raise,
    /// The `require` keyword.
    Require,
    /// The `load` keyword.
    Load,
    /// The `include` keyword.
    Include,
    /// The `extend` keyword.
    Extend,
    /// The `prepend` keyword.
    Prepend,
    /// The `and` keyword.
    And,
    /// The `or` keyword.
    Or,
    /// The `not` keyword.
    Not,
    /// The `in` keyword.
    In,
    /// The `true` keyword.
    True,
    /// The `false` keyword.
    False,
    /// The `nil` keyword.
    Nil,
    /// The `super` keyword.
    Super,
    /// The `self` keyword.
    Self_,
    /// The `alias` keyword.
    Alias,
    /// The `undef` keyword.
    Undef,
    /// The `defined?` keyword.
    Defined,
    /// The `do` keyword.
    Do,

    /// Plus operator `+`.
    Plus,
    /// Minus operator `-`.
    Minus,
    /// Multiply operator `*`.
    Multiply,
    /// Divide operator `/`.
    Divide,
    /// Modulo operator `%`.
    Modulo,
    /// Power operator `**`.
    Power,
    /// Equality operator `==`.
    EqualEqual,
    /// Inequality operator `!=`.
    NotEqual,
    /// Less than operator `<`.
    Less,
    /// Greater than operator `>`.
    Greater,
    /// Less than or equal operator `<=`.
    LessEqual,
    /// Greater than or equal operator `>=`.
    GreaterEqual,
    /// Case equality operator `===`.
    EqualEqualEqual,
    /// Spaceship operator `<=>`.
    Spaceship,
    /// Assignment operator `=`.
    Assign,
    /// Plus assignment operator `+=`.
    PlusAssign,
    /// Minus assignment operator `-=`.
    MinusAssign,
    /// Multiply assignment operator `*=`.
    MultiplyAssign,
    /// Divide assignment operator `/=`.
    DivideAssign,
    /// Modulo assignment operator `%=`.
    ModuloAssign,
    /// Power assignment operator `**=`.
    PowerAssign,
    /// Bitwise AND operator `&`.
    BitAnd,
    /// Bitwise OR operator `|`.
    BitOr,
    /// Bitwise XOR operator `^`.
    Xor,
    /// Logical NOT operator `!`.
    LogicalNot,
    /// Bitwise NOT operator `~`.
    Tilde,
    /// Left shift operator `<<`.
    LeftShift,
    /// Right shift operator `>>`.
    RightShift,
    /// AND assignment operator `&=`.
    AndAssign,
    /// OR assignment operator `|=`.
    OrAssign,
    /// XOR assignment operator `^=`.
    XorAssign,
    /// Left shift assignment operator `<<=`.
    LeftShiftAssign,
    /// Right shift assignment operator `>>=`.
    RightShiftAssign,
    /// Logical AND operator `&&`.
    AndAnd,
    /// Logical OR operator `||`.
    OrOr,
    /// OR OR assignment operator `||=`.
    OrOrAssign,
    /// AND AND assignment operator `&&=`.
    AndAndAssign,
    /// Question mark `?`.
    Question,
    /// Range operator `..`.
    DotDot,
    /// Range operator `...`.
    DotDotDot,
    /// Match operator `=~`.
    Match,
    /// Not match operator `!~`.
    NotMatch,

    /// Left parenthesis `(`.
    LeftParen,
    /// Right parenthesis `)`.
    RightParen,
    /// Left bracket `[`.
    LeftBracket,
    /// Right bracket `]`.
    RightBracket,
    /// Left brace `{`.
    LeftBrace,
    /// Right brace `}`.
    RightBrace,
    /// Comma `,`.
    Comma,
    /// Colon `:`.
    Colon,
    /// Semicolon `;`.
    Semicolon,
    /// Dot `.`.
    Dot,
    /// Double colon `::`.
    DoubleColon,
    /// At symbol `@`.
    At,
    /// Dollar sign `$`.
    Dollar,

    /// Whitespace.
    Whitespace,
    /// Newline.
    Newline,
    /// Comment.
    Comment,
    /// End of file.
    Eof,
    /// Invalid token.
    Invalid,
    /// Root element.
    Root,
    /// A binary expression.
    BinaryExpression,
    /// A unary expression.
    UnaryExpression,
    /// A literal expression.
    LiteralExpression,
    /// A parenthesized expression.
    ParenExpression,
    /// A parenthesized expression (alternative).
    ParenthesizedExpression,
    /// A method definition.
    MethodDefinition,
    /// A class definition.
    ClassDefinition,
    /// A module definition.
    ModuleDefinition,
    /// An if statement.
    IfStatement,
    /// A while statement.
    WhileStatement,
    /// An unless statement.
    UnlessStatement,
    /// An until statement.
    UntilStatement,
    /// A for statement.
    ForStatement,
    /// A case statement.
    CaseStatement,
    /// A when clause.
    WhenClause,
    /// A begin statement.
    BeginStatement,
    /// A rescue clause.
    RescueClause,
    /// An ensure clause.
    EnsureClause,
    /// A return statement.
    ReturnStatement,
    /// An if expression.
    IfExpression,
    /// A call expression.
    CallExpression,
    /// A member access expression.
    MemberAccess,
    /// A parameter list.
    ParameterList,
    /// An argument list.
    ArgumentList,
    /// An error element.
    Error,
    /// Equal operator `=`.
    Equal,
    /// 赋值语句 `name = expr`。
    AssignmentStatement,
    /// 数组字面量 `[a, b]`。
    ArrayExpression,
    /// 哈希字面量 `{a => 1}` / `{}`。
    HashExpression,
    /// 块表达式 `{...}` / `do...end`
    BlockExpression,
    /// `break`
    BreakStatement,
}

impl RubyElementType {
    /// Returns true if the element is ignored (whitespace, newline, or comment).
    pub fn is_ignored(&self) -> bool {
        matches!(self, Self::Whitespace | Self::Newline | Self::Comment)
    }

    /// Returns true if the element is a keyword.
    pub fn is_keyword(&self) -> bool {
        matches!(
            self,
            Self::If
                | Self::Unless
                | Self::Elsif
                | Self::Else
                | Self::Case
                | Self::When
                | Self::Then
                | Self::For
                | Self::While
                | Self::Until
                | Self::Break
                | Self::Next
                | Self::Redo
                | Self::Retry
                | Self::Return
                | Self::Yield
                | Self::Def
                | Self::Class
                | Self::Module
                | Self::End
                | Self::Lambda
                | Self::Proc
                | Self::Begin
                | Self::Rescue
                | Self::Ensure
                | Self::Raise
                | Self::Require
                | Self::Load
                | Self::Include
                | Self::Extend
                | Self::Prepend
                | Self::And
                | Self::Or
                | Self::Not
                | Self::In
                | Self::True
                | Self::False
                | Self::Nil
                | Self::Super
                | Self::Self_
                | Self::Alias
                | Self::Undef
                | Self::Defined
                | Self::Do
        )
    }
}

impl fmt::Display for RubyElementType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Identifier => "Identifier",
            Self::GlobalVariable => "GlobalVariable",
            Self::InstanceVariable => "InstanceVariable",
            Self::ClassVariable => "ClassVariable",
            Self::Constant => "Constant",
            Self::IntegerLiteral => "IntegerLiteral",
            Self::FloatLiteral => "FloatLiteral",
            Self::StringLiteral => "StringLiteral",
            Self::Literal => "Literal",
            Self::Symbol => "Symbol",
            Self::RegexLiteral => "RegexLiteral",

            Self::If => "If",
            Self::Unless => "Unless",
            Self::Elsif => "Elsif",
            Self::Else => "Else",
            Self::Case => "Case",
            Self::When => "When",
            Self::Then => "Then",
            Self::For => "For",
            Self::While => "While",
            Self::Until => "Until",
            Self::Break => "Break",
            Self::Next => "Next",
            Self::Redo => "Redo",
            Self::Retry => "Retry",
            Self::Return => "Return",
            Self::Yield => "Yield",
            Self::Def => "Def",
            Self::Class => "Class",
            Self::Module => "Module",
            Self::End => "End",
            Self::Lambda => "Lambda",
            Self::Proc => "Proc",
            Self::Begin => "Begin",
            Self::Rescue => "Rescue",
            Self::Ensure => "Ensure",
            Self::Raise => "Raise",
            Self::Require => "Require",
            Self::Load => "Load",
            Self::Include => "Include",
            Self::Extend => "Extend",
            Self::Prepend => "Prepend",
            Self::And => "And",
            Self::Or => "Or",
            Self::Not => "Not",
            Self::In => "In",
            Self::True => "True",
            Self::False => "False",
            Self::Nil => "Nil",
            Self::Super => "Super",
            Self::Self_ => "Self",
            Self::Alias => "Alias",
            Self::Undef => "Undef",
            Self::Defined => "Defined",
            Self::Do => "Do",

            Self::Plus => "Plus",
            Self::Minus => "Minus",
            Self::Multiply => "Multiply",
            Self::Divide => "Divide",
            Self::Modulo => "Modulo",
            Self::Power => "Power",
            Self::EqualEqual => "EqualEqual",
            Self::NotEqual => "NotEqual",
            Self::Less => "Less",
            Self::Greater => "Greater",
            Self::LessEqual => "LessEqual",
            Self::GreaterEqual => "GreaterEqual",
            Self::EqualEqualEqual => "EqualEqualEqual",
            Self::Spaceship => "Spaceship",
            Self::Assign => "Assign",
            Self::PlusAssign => "PlusAssign",
            Self::MinusAssign => "MinusAssign",
            Self::MultiplyAssign => "MultiplyAssign",
            Self::DivideAssign => "DivideAssign",
            Self::ModuloAssign => "ModuloAssign",
            Self::PowerAssign => "PowerAssign",
            Self::BitAnd => "BitAnd",
            Self::BitOr => "BitOr",
            Self::Xor => "Xor",
            Self::LogicalNot => "LogicalNot",
            Self::Tilde => "Tilde",
            Self::LeftShift => "LeftShift",
            Self::RightShift => "RightShift",
            Self::AndAssign => "AndAssign",
            Self::OrAssign => "OrAssign",
            Self::XorAssign => "XorAssign",
            Self::LeftShiftAssign => "LeftShiftAssign",
            Self::RightShiftAssign => "RightShiftAssign",
            Self::AndAnd => "AndAnd",
            Self::OrOr => "OrOr",
            Self::OrOrAssign => "OrOrAssign",
            Self::AndAndAssign => "AndAndAssign",
            Self::Question => "Question",
            Self::DotDot => "DotDot",
            Self::DotDotDot => "DotDotDot",
            Self::Match => "Match",
            Self::NotMatch => "NotMatch",

            Self::LeftParen => "LeftParen",
            Self::RightParen => "RightParen",
            Self::LeftBracket => "LeftBracket",
            Self::RightBracket => "RightBracket",
            Self::LeftBrace => "LeftBrace",
            Self::RightBrace => "RightBrace",
            Self::Comma => "Comma",
            Self::Colon => "Colon",
            Self::Semicolon => "Semicolon",
            Self::Dot => "Dot",
            Self::DoubleColon => "DoubleColon",
            Self::At => "At",
            Self::Dollar => "Dollar",

            Self::Whitespace => "Whitespace",
            Self::Newline => "Newline",
            Self::Comment => "Comment",
            Self::Eof => "Eof",
            Self::Invalid => "Invalid",
            Self::Root => "Root",
            Self::BinaryExpression => "BinaryExpression",
            Self::UnaryExpression => "UnaryExpression",
            Self::LiteralExpression => "LiteralExpression",
            Self::ParenExpression => "ParenExpression",
            Self::ParenthesizedExpression => "ParenthesizedExpression",
            Self::MethodDefinition => "MethodDefinition",
            Self::ClassDefinition => "ClassDefinition",
            Self::ModuleDefinition => "ModuleDefinition",
            Self::IfStatement => "IfStatement",
            Self::WhileStatement => "WhileStatement",
            Self::UnlessStatement => "UnlessStatement",
            Self::UntilStatement => "UntilStatement",
            Self::ForStatement => "ForStatement",
            Self::CaseStatement => "CaseStatement",
            Self::WhenClause => "WhenClause",
            Self::BeginStatement => "BeginStatement",
            Self::RescueClause => "RescueClause",
            Self::EnsureClause => "EnsureClause",
            Self::ReturnStatement => "ReturnStatement",
            Self::IfExpression => "IfExpression",
            Self::CallExpression => "CallExpression",
            Self::MemberAccess => "MemberAccess",
            Self::ParameterList => "ParameterList",
            Self::ArgumentList => "ArgumentList",
            Self::Error => "Error",
            Self::Equal => "Equal",
            Self::AssignmentStatement => "AssignmentStatement",
            Self::ArrayExpression => "ArrayExpression",
            Self::HashExpression => "HashExpression",
            Self::BlockExpression => "BlockExpression",
            Self::BreakStatement => "BreakStatement",
        };
        write!(f, "{}", name)
    }
}

impl ElementType for RubyElementType {
    type Role = UniversalElementRole;

    fn role(&self) -> Self::Role {
        match self {
            _ => UniversalElementRole::None,
        }
    }
}

impl From<crate::lexer::token_type::RubyTokenType> for RubyElementType {
    fn from(token: crate::lexer::token_type::RubyTokenType) -> Self {
        unsafe { std::mem::transmute(token) }
    }
}
