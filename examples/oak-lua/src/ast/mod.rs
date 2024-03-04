#![doc = include_str!("readme.md")]
use core::range::Range;
use oak_core::source::{SourceBuffer, ToSource};
#[cfg(feature = "oak-pretty-print")]
use oak_pretty_print::{AsDocument, Document};

/// Lua root node
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LuaRoot {
    /// Statements in the root.
    pub statements: Vec<LuaStatement>,
    /// Source span of the root.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

impl ToSource for LuaRoot {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        for stmt in &self.statements {
            stmt.to_source(buffer);
            buffer.push("\n")
        }
    }
}

#[cfg(feature = "oak-pretty-print")]
impl AsDocument for LuaRoot {
    fn as_document(&self, _config: &Self::Config) -> Document<'_> {
        Document::join(self.statements.iter().map(|s| s.as_document(&())), Document::Line)
    }
}

/// Lua statement
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LuaStatement {
    /// A local statement.
    Local(LuaLocalStatement),
    /// An assignment statement.
    Assignment(LuaAssignmentStatement),
    /// An expression statement.
    Expression(LuaExpression),
    /// A return statement.
    Return(LuaReturnStatement),
    /// An if statement.
    If(LuaIfStatement),
    /// A while statement.
    While(LuaWhileStatement),
    /// A for statement.
    For(LuaForStatement),
    /// A repeat statement.
    Repeat(LuaRepeatStatement),
    /// A function statement.
    Function(LuaFunctionStatement),
    /// A break statement.
    Break,
    /// A do block.
    Do(Vec<LuaStatement>),
    /// A goto statement.
    Goto(String),
    /// A label statement.
    Label(String),
}

impl ToSource for LuaStatement {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        match self {
            LuaStatement::Local(s) => s.to_source(buffer),
            LuaStatement::Assignment(s) => s.to_source(buffer),
            LuaStatement::Expression(e) => e.to_source(buffer),
            LuaStatement::Return(s) => s.to_source(buffer),
            LuaStatement::If(s) => s.to_source(buffer),
            LuaStatement::While(s) => s.to_source(buffer),
            LuaStatement::For(s) => s.to_source(buffer),
            LuaStatement::Repeat(s) => s.to_source(buffer),
            LuaStatement::Function(s) => s.to_source(buffer),
            LuaStatement::Break => buffer.push("break"),
            LuaStatement::Do(stmts) => {
                buffer.push("do\n");
                for stmt in stmts {
                    stmt.to_source(buffer);
                    buffer.push("\n")
                }
                buffer.push("end")
            }
            LuaStatement::Goto(label) => {
                buffer.push("goto ");
                buffer.push(label)
            }
            LuaStatement::Label(name) => {
                buffer.push("::");
                buffer.push(name);
                buffer.push("::")
            }
        }
    }
}

#[cfg(feature = "oak-pretty-print")]
impl AsDocument for LuaStatement {
    fn as_document(&self, _config: &Self::Config) -> Document<'_> {
        let mut buffer = SourceBuffer::new();
        self.to_source(&mut buffer);
        Document::Text(buffer.finish().into())
    }
}

/// Local variable declaration
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct LuaLocalStatement {
    /// Names of the local variables.
    pub names: Vec<String>,
    /// Values assigned to the local variables.
    pub values: Vec<LuaExpression>,
}

impl ToSource for LuaLocalStatement {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        buffer.push("local ");
        for (i, name) in self.names.iter().enumerate() {
            if i > 0 {
                buffer.push(", ")
            }
            buffer.push(name)
        }
        if !self.values.is_empty() {
            buffer.push(" = ");
            for (i, val) in self.values.iter().enumerate() {
                if i > 0 {
                    buffer.push(", ")
                }
                val.to_source(buffer)
            }
        }
    }
}

/// Assignment statement
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct LuaAssignmentStatement {
    /// Targets of the assignment.
    pub targets: Vec<LuaExpression>,
    /// Values assigned to the targets.
    pub values: Vec<LuaExpression>,
}

impl ToSource for LuaAssignmentStatement {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        for (i, target) in self.targets.iter().enumerate() {
            if i > 0 {
                buffer.push(", ")
            }
            target.to_source(buffer)
        }
        buffer.push(" = ");
        for (i, val) in self.values.iter().enumerate() {
            if i > 0 {
                buffer.push(", ")
            }
            val.to_source(buffer)
        }
    }
}

/// Return statement
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct LuaReturnStatement {
    /// Values returned.
    pub values: Vec<LuaExpression>,
}

impl ToSource for LuaReturnStatement {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        buffer.push("return ");
        for (i, val) in self.values.iter().enumerate() {
            if i > 0 {
                buffer.push(", ")
            }
            val.to_source(buffer)
        }
    }
}

/// If statement
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct LuaIfStatement {
    /// The condition of the `if` block.
    pub condition: LuaExpression,
    /// The block of the `if` part.
    pub then_block: Vec<LuaStatement>,
    /// Else-if blocks.
    pub else_ifs: Vec<(LuaExpression, Vec<LuaStatement>)>,
    /// The block of the `else` part.
    pub else_block: Option<Vec<LuaStatement>>,
}

impl ToSource for LuaIfStatement {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        buffer.push("if ");
        self.condition.to_source(buffer);
        buffer.push(" then\n");
        for stmt in &self.then_block {
            stmt.to_source(buffer);
            buffer.push("\n")
        }
        for (cond, block) in &self.else_ifs {
            buffer.push("elseif ");
            cond.to_source(buffer);
            buffer.push(" then\n");
            for stmt in block {
                stmt.to_source(buffer);
                buffer.push("\n")
            }
        }
        if let Some(block) = &self.else_block {
            buffer.push("else\n");
            for stmt in block {
                stmt.to_source(buffer);
                buffer.push("\n")
            }
        }
        buffer.push("end")
    }
}

/// While statement
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LuaWhileStatement {
    /// The condition of the `while` loop.
    pub condition: LuaExpression,
    /// The block of the `while` loop.
    pub block: Vec<LuaStatement>,
}

impl ToSource for LuaWhileStatement {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        buffer.push("while ");
        self.condition.to_source(buffer);
        buffer.push(" do\n");
        for stmt in &self.block {
            stmt.to_source(buffer);
            buffer.push("\n")
        }
        buffer.push("end")
    }
}

/// For statement
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LuaForStatement {
    /// A numeric for loop: `for var = start, end, step do block end`.
    Numeric {
        /// The loop variable.
        variable: String,
        /// The start value.
        start: LuaExpression,
        /// The end value.
        end: LuaExpression,
        /// The step value.
        step: Option<LuaExpression>,
        /// The loop block.
        block: Vec<LuaStatement>,
    },
    /// A generic for loop: `for vars in iters do block end`.
    Generic {
        /// The loop variables.
        variables: Vec<String>,
        /// The iterators.
        iterators: Vec<LuaExpression>,
        /// The loop block.
        block: Vec<LuaStatement>,
    },
}

impl ToSource for LuaForStatement {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        match self {
            LuaForStatement::Numeric { variable, start, end, step, block } => {
                buffer.push("for ");
                buffer.push(variable);
                buffer.push(" = ");
                start.to_source(buffer);
                buffer.push(", ");
                end.to_source(buffer);
                if let Some(s) = step {
                    buffer.push(", ");
                    s.to_source(buffer)
                }
                buffer.push(" do\n");
                for stmt in block {
                    stmt.to_source(buffer);
                    buffer.push("\n")
                }
                buffer.push("end")
            }
            LuaForStatement::Generic { variables, iterators, block } => {
                buffer.push("for ");
                for (i, var) in variables.iter().enumerate() {
                    if i > 0 {
                        buffer.push(", ")
                    }
                    buffer.push(var)
                }
                buffer.push(" in ");
                for (i, it) in iterators.iter().enumerate() {
                    if i > 0 {
                        buffer.push(", ")
                    }
                    it.to_source(buffer)
                }
                buffer.push(" do\n");
                for stmt in block {
                    stmt.to_source(buffer);
                    buffer.push("\n")
                }
                buffer.push("end")
            }
        }
    }
}

/// Repeat statement
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LuaRepeatStatement {
    /// The block of the `repeat` loop.
    pub block: Vec<LuaStatement>,
    /// The condition of the `repeat` loop.
    pub condition: LuaExpression,
}

impl ToSource for LuaRepeatStatement {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        buffer.push("repeat\n");
        for stmt in &self.block {
            stmt.to_source(buffer);
            buffer.push("\n")
        }
        buffer.push("until ");
        self.condition.to_source(buffer)
    }
}

/// Function definition statement
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LuaFunctionStatement {
    /// The name parts of the function.
    pub name: Vec<String>,
    /// The receiver part (after `:`) if any.
    pub receiver: Option<String>,
    /// The parameters of the function.
    pub parameters: Vec<String>,
    /// Whether the function has a vararg parameter.
    pub is_vararg: bool,
    /// The function body.
    pub block: Vec<LuaStatement>,
}

impl ToSource for LuaFunctionStatement {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        buffer.push("function ");
        for (i, part) in self.name.iter().enumerate() {
            if i > 0 {
                buffer.push(".")
            }
            buffer.push(part)
        }
        if let Some(recv) = &self.receiver {
            buffer.push(":");
            buffer.push(recv)
        }
        buffer.push("(");
        for (i, param) in self.parameters.iter().enumerate() {
            if i > 0 {
                buffer.push(", ")
            }
            buffer.push(param)
        }
        if self.is_vararg {
            if !self.parameters.is_empty() {
                buffer.push(", ")
            }
            buffer.push("...")
        }
        buffer.push(")\n");
        for stmt in &self.block {
            stmt.to_source(buffer);
            buffer.push("\n")
        }
        buffer.push("end")
    }
}

/// Lua expression
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LuaExpression {
    /// An identifier.
    Identifier(String),
    /// A number literal.
    Number(f64),
    /// A string literal.
    String(String),
    /// A boolean literal.
    Boolean(bool),
    /// A nil literal.
    Nil,
    /// A binary expression.
    Binary(Box<LuaBinaryExpression>),
    /// A unary expression.
    Unary(Box<LuaUnaryExpression>),
    /// A call expression.
    Call(Box<LuaCallExpression>),
    /// A table constructor.
    Table(LuaTableConstructor),
    /// A function expression.
    Function(LuaFunctionExpression),
    /// An index expression.
    Index(Box<LuaIndexExpression>),
    /// A member expression.
    Member(Box<LuaMemberExpression>),
    /// A vararg expression.
    Vararg,
}

impl ToSource for LuaExpression {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        match self {
            LuaExpression::Identifier(id) => buffer.push(id),
            LuaExpression::Number(n) => buffer.push(&n.to_string()),
            LuaExpression::String(s) => {
                buffer.push("\"");
                buffer.push(s);
                buffer.push("\"")
            }
            LuaExpression::Boolean(b) => buffer.push(if *b { "true" } else { "false" }),
            LuaExpression::Nil => buffer.push("nil"),
            LuaExpression::Binary(bin) => bin.to_source(buffer),
            LuaExpression::Unary(un) => un.to_source(buffer),
            LuaExpression::Call(call) => call.to_source(buffer),
            LuaExpression::Table(table) => table.to_source(buffer),
            LuaExpression::Function(func) => func.to_source(buffer),
            LuaExpression::Index(idx) => idx.to_source(buffer),
            LuaExpression::Member(mem) => mem.to_source(buffer),
            LuaExpression::Vararg => buffer.push("..."),
        }
    }
}

#[cfg(feature = "oak-pretty-print")]
impl AsDocument for LuaExpression {
    fn as_document(&self, _config: &Self::Config) -> Document<'_> {
        let mut buffer = SourceBuffer::new();
        self.to_source(&mut buffer);
        Document::Text(buffer.finish().into())
    }
}

/// Unary expression
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LuaUnaryExpression {
    /// The operator.
    pub op: String,
    /// The operand.
    pub operand: LuaExpression,
}

impl ToSource for LuaUnaryExpression {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        buffer.push(&self.op);
        self.operand.to_source(buffer)
    }
}

/// Binary expression
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LuaBinaryExpression {
    /// The left-hand side.
    pub left: LuaExpression,
    /// The operator.
    pub op: String,
    /// The right-hand side.
    pub right: LuaExpression,
}

impl ToSource for LuaBinaryExpression {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        self.left.to_source(buffer);
        buffer.push(" ");
        buffer.push(&self.op);
        buffer.push(" ");
        self.right.to_source(buffer)
    }
}

/// Function call expression
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LuaCallExpression {
    /// The function being called.
    pub function: LuaExpression,
    /// The arguments passed to the function.
    pub arguments: Vec<LuaExpression>,
}

impl ToSource for LuaCallExpression {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        self.function.to_source(buffer);
        buffer.push("(");
        for (i, arg) in self.arguments.iter().enumerate() {
            if i > 0 {
                buffer.push(", ")
            }
            arg.to_source(buffer)
        }
        buffer.push(")")
    }
}

/// Table constructor
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LuaTableConstructor {
    /// The fields in the table constructor.
    pub fields: Vec<LuaTableField>,
}

impl ToSource for LuaTableConstructor {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        buffer.push("{");
        for (i, field) in self.fields.iter().enumerate() {
            if i > 0 {
                buffer.push(", ")
            }
            field.to_source(buffer)
        }
        buffer.push("}")
    }
}

/// A field in a Lua table constructor.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LuaTableField {
    /// A keyed field: `[key] = value`.
    Keyed {
        /// The key expression.
        key: LuaExpression,
        /// The value expression.
        value: LuaExpression,
    },
    /// A named field: `name = value`.
    Named {
        /// The name.
        name: String,
        /// The value expression.
        value: LuaExpression,
    },
    /// A list field: `value`.
    List {
        /// The value expression.
        value: LuaExpression,
    },
}

impl ToSource for LuaTableField {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        match self {
            LuaTableField::Keyed { key, value } => {
                buffer.push("[");
                key.to_source(buffer);
                buffer.push("] = ");
                value.to_source(buffer)
            }
            LuaTableField::Named { name, value } => {
                buffer.push(name);
                buffer.push(" = ");
                value.to_source(buffer)
            }
            LuaTableField::List { value } => value.to_source(buffer),
        }
    }
}

/// Anonymous function expression
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LuaFunctionExpression {
    /// The parameters of the function.
    pub parameters: Vec<String>,
    /// Whether the function has a vararg parameter.
    pub is_vararg: bool,
    /// The function body.
    pub block: Vec<LuaStatement>,
}

impl ToSource for LuaFunctionExpression {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        buffer.push("function(");
        for (i, param) in self.parameters.iter().enumerate() {
            if i > 0 {
                buffer.push(", ")
            }
            buffer.push(param)
        }
        if self.is_vararg {
            if !self.parameters.is_empty() {
                buffer.push(", ")
            }
            buffer.push("...")
        }
        buffer.push(")\n");
        for stmt in &self.block {
            stmt.to_source(buffer);
            buffer.push("\n")
        }
        buffer.push("end")
    }
}

/// Index access expression
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LuaIndexExpression {
    /// The table being indexed.
    pub table: LuaExpression,
    /// The index expression.
    pub index: LuaExpression,
}

impl ToSource for LuaIndexExpression {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        self.table.to_source(buffer);
        buffer.push("[");
        self.index.to_source(buffer);
        buffer.push("]")
    }
}

/// Member access expression
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LuaMemberExpression {
    /// The table whose member is being accessed.
    pub table: LuaExpression,
    /// The member name.
    pub member: String,
    /// Whether this is a method call (using `:`).
    pub is_method: bool,
}

impl ToSource for LuaMemberExpression {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        self.table.to_source(buffer);
        if self.is_method {
            buffer.push(":")
        }
        else {
            buffer.push(".")
        }
        buffer.push(&self.member)
    }
}
