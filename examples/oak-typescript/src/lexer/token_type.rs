use oak_core::{Token, TokenType, UniversalTokenRole};

/// Alias for `Token<TypeScriptTokenType>`.
pub type TypeScriptToken = Token<TypeScriptTokenType>;

/// Token types for TypeScript.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u16)]
pub enum TypeScriptTokenType {
    /// Named imports.
    NamedImports,
    /// A decorator.
    Decorator,
    /// Arrow function `=>`.
    ArrowFunction,
    /// Predefined type.
    PredefinedType,
    /// `abstract` keyword.
    Abstract,
    /// `any` keyword.
    Any,
    /// `as` keyword.
    As,
    /// `asserts` keyword.
    Asserts,
    /// `async` keyword.
    Async,
    /// `await` keyword.
    Await,
    /// `boolean` keyword.
    Boolean,
    /// `break` keyword.
    Break,
    /// `case` keyword.
    Case,
    /// `catch` keyword.
    Catch,
    /// `class` keyword.
    Class,
    /// `const` keyword.
    Const,
    /// `constructor` keyword.
    Constructor,
    /// `continue` keyword.
    Continue,
    /// `debugger` keyword.
    Debugger,
    /// `declare` keyword.
    Declare,
    /// `default` keyword.
    Default,
    /// `delete` keyword.
    Delete,
    /// `do` keyword.
    Do,
    /// `else` keyword.
    Else,
    /// `enum` keyword.
    Enum,
    /// `export` keyword.
    Export,
    /// `extends` keyword.
    Extends,
    /// `false` keyword.
    False,
    /// `finally` keyword.
    Finally,
    /// `for` keyword.
    For,
    /// `from` keyword.
    From,
    /// `function` keyword.
    Function,
    /// `get` keyword.
    Get,
    /// `global` keyword.
    Global,
    /// `if` keyword.
    If,
    /// `implements` keyword.
    Implements,
    /// `import` keyword.
    Import,
    /// `in` keyword.
    In,
    /// `infer` keyword.
    Infer,
    /// `instanceof` keyword.
    Instanceof,
    /// `interface` keyword.
    Interface,
    /// `is` keyword.
    Is,
    /// `keyof` keyword.
    Keyof,
    /// `let` keyword.
    Let,
    /// `namespace` keyword.
    Namespace,
    /// `never` keyword.
    Never,
    /// `new` keyword.
    New,
    /// `null` keyword.
    Null,
    /// `number` keyword.
    Number,
    /// `object` keyword.
    Object,
    /// `of` keyword.
    Of,
    /// `override` keyword.
    Override,
    /// `package` keyword.
    Package,
    /// `private` keyword.
    Private,
    /// `protected` keyword.
    Protected,
    /// `public` keyword.
    Public,
    /// `readonly` keyword.
    Readonly,
    /// `require` keyword.
    Require,
    /// `return` keyword.
    Return,
    /// `set` keyword.
    Set,
    /// `static` keyword.
    Static,
    /// `string` keyword.
    String,
    /// `super` keyword.
    Super,
    /// `switch` keyword.
    Switch,
    /// `symbol` keyword.
    Symbol,
    /// `this` keyword.
    This,
    /// `throw` keyword.
    Throw,
    /// `true` keyword.
    True,
    /// `try` keyword.
    Try,
    /// `type` keyword.
    Type,
    /// `typeof` keyword.
    Typeof,
    /// `undefined` keyword.
    Undefined,
    /// `unique` keyword.
    Unique,
    /// `unknown` keyword.
    Unknown,
    /// `var` keyword.
    Var,
    /// `void` keyword.
    Void,
    /// `while` keyword.
    While,
    /// `with` keyword.
    With,
    /// `yield` keyword.
    Yield,
    /// Plus `+`.
    Plus,
    /// Minus `-`.
    Minus,
    /// Star `*`.
    Star,
    /// Slash `/`.
    Slash,
    /// Percent `%`.
    Percent,
    /// StarStar `**`.
    StarStar,
    /// Question `?`.
    Question,
    /// DotDotDot `...`.
    DotDotDot,
    /// Less `<`.
    Less,
    /// Greater `>`.
    Greater,
    /// LessEqual `<=`.
    LessEqual,
    /// Greater than or equal `>=`.
    GreaterEqual,
    /// Equal `==`.
    EqualEqual,
    /// Not equal `!=`.
    NotEqual,
    /// Strict equal `===`.
    EqualEqualEqual,
    /// Strict not equal `!==`.
    NotEqualEqual,
    /// Logical AND `&&`.
    AmpersandAmpersand,
    /// Logical OR `||`.
    PipePipe,
    /// Logical NOT `!`.
    Exclamation,
    /// Bitwise AND `&`.
    Ampersand,
    /// Bitwise OR `|`.
    Pipe,
    /// Bitwise XOR `^`.
    Caret,
    /// Bitwise NOT `~`.
    Tilde,
    /// Left shift `<<`.
    LeftShift,
    /// Right shift `>>`.
    RightShift,
    /// Unsigned right shift `>>>`.
    UnsignedRightShift,
    /// Assignment `=`.
    Equal,
    /// Addition assignment `+=`.
    PlusEqual,
    /// Subtraction assignment `-=`.
    MinusEqual,
    /// Multiplication assignment `*=`.
    StarEqual,
    /// Division assignment `/=`.
    SlashEqual,
    /// Remainder assignment `%=`.
    PercentEqual,
    /// Exponentiation assignment `**=`.
    StarStarEqual,
    /// Left shift assignment `<<=`.
    LeftShiftEqual,
    /// Right shift assignment `>>=`.
    RightShiftEqual,
    /// Unsigned right shift assignment `>>>=`.
    UnsignedRightShiftEqual,
    /// Bitwise AND assignment `&=`.
    AmpersandEqual,
    /// Bitwise OR assignment `|=`.
    PipeEqual,
    /// Bitwise XOR assignment `^=`.
    CaretEqual,
    /// Logical AND assignment `&&=`.
    AmpersandAmpersandEqual,
    /// Logical OR assignment `||=`.
    PipePipeEqual,
    /// Nullish coalescing assignment `??=`.
    QuestionQuestionEqual,
    /// Increment `++`.
    PlusPlus,
    /// Decrement `--`.
    MinusMinus,
    /// Nullish coalescing `??`.
    QuestionQuestion,
    /// Optional chaining `?.`.
    QuestionDot,
    /// Arrow `=>`.
    Arrow,
    /// Left parenthesis `(`.
    LeftParen,
    /// Right parenthesis `)`.
    RightParen,
    /// Left brace `{`.
    LeftBrace,
    /// Right brace `}`.
    RightBrace,
    /// Left bracket `[`.
    LeftBracket,
    /// Right bracket `]`.
    RightBracket,
    /// Semicolon `;`.
    Semicolon,
    /// Comma `,`.
    Comma,
    /// Dot `.`.
    Dot,
    /// Colon `:`.
    Colon,
    /// At `@`.
    At,
    /// String literal.
    StringLiteral,
    /// Numeric literal.
    NumericLiteral,
    /// BigInt literal.
    BigIntLiteral,
    /// Boolean literal.
    BooleanLiteral,
    /// Template string.
    TemplateString,
    /// Regular expression literal.
    RegexLiteral,
    /// Identifier name.
    IdentifierName,
    /// Line comment `//`.
    LineComment,
    /// Block comment `/* */`.
    BlockComment,
    /// Whitespace.
    Whitespace,
    /// Newline.
    Newline,
    /// End of stream.
    Eof,
    /// Root node.
    Root,
    /// Source file.
    SourceFile,
    /// Module.
    Module,
    /// Variable declaration.
    VariableDeclaration,
    /// Function declaration.
    FunctionDeclaration,
    /// Class declaration.
    ClassDeclaration,
    /// Interface declaration.
    InterfaceDeclaration,
    /// Type alias declaration.
    TypeAliasDeclaration,
    /// Enum declaration.
    EnumDeclaration,
    /// Namespace declaration.
    NamespaceDeclaration,
    /// Class body.
    ClassBody,
    /// Import declaration.
    ImportDeclaration,
    /// Export declaration.
    ExportDeclaration,
    /// Import clause.
    ImportClause,
    /// Import specifier.
    ImportSpecifier,
    /// Parameter.
    Parameter,
    /// Call argument.
    CallArgument,
    /// Property declaration.
    PropertyDeclaration,
    /// Method declaration.
    MethodDeclaration,
    /// Constructor declaration.
    ConstructorDeclaration,
    /// Property assignment.
    PropertyAssignment,
    /// Shorthand property assignment.
    ShorthandPropertyAssignment,
    /// Spread element.
    SpreadElement,
    /// Error token.
    Error,
    /// JSX element.
    JsxElement,
    /// JSX self-closing element.
    JsxSelfClosingElement,
    /// JSX opening element.
    JsxOpeningElement,
    /// JSX closing element.
    JsxClosingElement,
    /// JSX fragment.
    JsxFragment,
    /// JSX opening fragment.
    JsxOpeningFragment,
    /// JSX closing fragment.
    JsxClosingFragment,
    /// JSX attribute.
    JsxAttribute,
    /// JSX attributes.
    JsxAttributes,
    /// JSX expression container.
    JsxExpressionContainer,
    /// JSX spread attribute.
    JsxSpreadAttribute,
    /// JSX text.
    JsxText,
    /// Binary expression.
    BinaryExpression,
    /// Unary expression.
    UnaryExpression,
    /// Conditional expression `a ? b : c`.
    ConditionalExpression,
    /// Call expression `f()`.
    CallExpression,
    /// New expression `new C()`.
    NewExpression,
    /// Member expression `a.b` or `a[b]`.
    MemberExpression,
    /// Array expression `[a, b]`.
    ArrayExpression,
    /// Object expression `{a: b}`.
    ObjectExpression,
    /// Function expression `function() {}`.
    FunctionExpression,
    /// Template expression `` `...` ``.
    TemplateExpression,
    /// Tagged template expression `f` `...` ``.
    TaggedTemplateExpression,
    /// As expression `a as T`.
    AsExpression,
    /// Type assertion expression `<T>a`.
    TypeAssertionExpression,
    /// Non-null expression `a!`.
    NonNullExpression,
    /// Update expression `++a` or `a--`.
    UpdateExpression,
    /// Expression statement.
    ExpressionStatement,
    /// Block statement.
    BlockStatement,
    /// If statement.
    IfStatement,
    /// While statement.
    WhileStatement,
    /// For statement.
    ForStatement,
    /// For-in statement.
    ForInStatement,
    /// For-of statement.
    ForOfStatement,
    /// Do-while statement.
    DoWhileStatement,
    /// Switch statement.
    SwitchStatement,
    /// Case clause.
    CaseClause,
    /// Default clause.
    DefaultClause,
    /// Try statement.
    TryStatement,
    /// Catch clause.
    CatchClause,
    /// Finally clause.
    FinallyClause,
    /// Throw statement.
    ThrowStatement,
    /// Return statement.
    ReturnStatement,
    /// Break statement.
    BreakStatement,
    /// Continue statement.
    ContinueStatement,
    /// Debugger statement.
    DebuggerStatement,
    /// With statement.
    WithStatement,
    /// Binding pattern.
    BindingPattern,
    /// Array binding pattern.
    ArrayBindingPattern,
    /// Object binding pattern.
    ObjectBindingPattern,
    /// Binding element.
    BindingElement,
    /// Type reference.
    TypeReference,
    /// Type literal.
    TypeLiteral,
    /// Function type.
    FunctionType,
    /// Constructor type.
    ConstructorType,
    /// Array type.
    ArrayType,
    /// Tuple type.
    TupleType,
    /// Union type.
    UnionType,
    /// Intersection type.
    IntersectionType,
    /// Conditional type.
    ConditionalType,
    /// Mapped type.
    MappedType,
    /// Indexed access type.
    IndexedAccessType,
    /// Property signature.
    PropertySignature,
    /// Method signature.
    MethodSignature,
    /// Literal type.
    LiteralType,
    /// Type query.
    TypeQuery,
    /// Type predicate.
    TypePredicate,
    /// Type annotation.
    TypeAnnotation,
    /// Type parameter.
    TypeParameter,
    /// Heritage clause.
    HeritageClause,
    /// Enum member.
    EnumMember,
}

impl TypeScriptTokenType {
    /// Returns true for contextual keywords that are valid identifier names in
    /// expression position (`require('./m')`, `map.get(k)`, `obj.type`).
    pub fn is_contextual_keyword(&self) -> bool {
        use TypeScriptTokenType::*;
        matches!(self,
            Abstract | Any | Asserts | Async | Boolean | Constructor | Declare | From | Get | Implements | Keyof | Namespace | Never | Number | Object | Package | Private | Protected | Public | Readonly | Require | Set | String | Symbol | Type | Unique | Unknown
        )
    }

    /// Returns the keyword token type for the given text, if any.
    pub fn from_keyword(text: &str) -> Option<Self> {
        match text {
            "abstract" => Some(Self::Abstract),
            "any" => Some(Self::Any),
            "as" => Some(Self::As),
            "asserts" => Some(Self::Asserts),
            "async" => Some(Self::Async),
            "await" => Some(Self::Await),
            "boolean" => Some(Self::Boolean),
            "break" => Some(Self::Break),
            "case" => Some(Self::Case),
            "catch" => Some(Self::Catch),
            "class" => Some(Self::Class),
            "const" => Some(Self::Const),
            "constructor" => Some(Self::Constructor),
            "continue" => Some(Self::Continue),
            "debugger" => Some(Self::Debugger),
            "declare" => Some(Self::Declare),
            "default" => Some(Self::Default),
            "delete" => Some(Self::Delete),
            "do" => Some(Self::Do),
            "else" => Some(Self::Else),
            "enum" => Some(Self::Enum),
            "export" => Some(Self::Export),
            "extends" => Some(Self::Extends),
            "false" => Some(Self::False),
            "finally" => Some(Self::Finally),
            "for" => Some(Self::For),
            "from" => Some(Self::From),
            "function" => Some(Self::Function),
            "get" => Some(Self::Get),
            "global" => Some(Self::Global),
            "if" => Some(Self::If),
            "implements" => Some(Self::Implements),
            "import" => Some(Self::Import),
            "in" => Some(Self::In),
            "infer" => Some(Self::Infer),
            "instanceof" => Some(Self::Instanceof),
            "interface" => Some(Self::Interface),
            "is" => Some(Self::Is),
            "keyof" => Some(Self::Keyof),
            "let" => Some(Self::Let),
            "namespace" => Some(Self::Namespace),
            "never" => Some(Self::Never),
            "new" => Some(Self::New),
            "null" => Some(Self::Null),
            "number" => Some(Self::Number),
            "object" => Some(Self::Object),
            "of" => Some(Self::Of),
            "override" => Some(Self::Override),
            "package" => Some(Self::Package),
            "private" => Some(Self::Private),
            "protected" => Some(Self::Protected),
            "public" => Some(Self::Public),
            "readonly" => Some(Self::Readonly),
            "require" => Some(Self::Require),
            "return" => Some(Self::Return),
            "set" => Some(Self::Set),
            "static" => Some(Self::Static),
            "string" => Some(Self::String),
            "super" => Some(Self::Super),
            "switch" => Some(Self::Switch),
            "symbol" => Some(Self::Symbol),
            "this" => Some(Self::This),
            "throw" => Some(Self::Throw),
            "true" => Some(Self::True),
            "try" => Some(Self::Try),
            "type" => Some(Self::Type),
            "typeof" => Some(Self::Typeof),
            "undefined" => Some(Self::Undefined),
            "unique" => Some(Self::Unique),
            "unknown" => Some(Self::Unknown),
            "var" => Some(Self::Var),
            "void" => Some(Self::Void),
            "while" => Some(Self::While),
            "with" => Some(Self::With),
            "yield" => Some(Self::Yield),
            _ => None,
        }
    }
}

impl TokenType for TypeScriptTokenType {
    type Role = UniversalTokenRole;
    const END_OF_STREAM: Self = Self::Eof;

    fn is_ignored(&self) -> bool {
        matches!(self, Self::Whitespace | Self::Newline | Self::LineComment | Self::BlockComment)
    }

    fn role(&self) -> Self::Role {
        match self {
            Self::Whitespace => UniversalTokenRole::Whitespace,
            Self::Newline => UniversalTokenRole::Whitespace,
            Self::LineComment => UniversalTokenRole::Comment,
            Self::BlockComment => UniversalTokenRole::Comment,
            Self::Eof => UniversalTokenRole::Eof,
            Self::Error => UniversalTokenRole::Error,
            _ => UniversalTokenRole::None,
        }
    }
}
