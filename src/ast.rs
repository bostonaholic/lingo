/// AST node types for the Lingo programming language.

#[derive(Debug, Clone)]
pub struct Span {
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone)]
pub struct Program {
    pub items: Vec<Item>,
}

#[derive(Debug, Clone)]
pub enum Item {
    FnDecl(FnDecl),
    ExprStmt(Expr),
}

#[derive(Debug, Clone)]
pub struct FnDecl {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<String>,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub type_ann: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub expr: Option<Box<Expr>>,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Let(LetStmt),
    Expr(Expr),
    For(ForStmt),
    While(WhileStmt),
    Return(Option<Expr>),
    Break,
}

#[derive(Debug, Clone)]
pub struct LetStmt {
    pub pattern: Pattern,
    pub mutable: bool,
    pub type_ann: Option<String>,
    pub value: Expr,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ForStmt {
    pub binding: Pattern,
    pub iterable: Expr,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct WhileStmt {
    pub condition: Expr,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Literal(Literal),
    Ident(String, Span),
    Binary(Box<Expr>, BinOp, Box<Expr>, Span),
    Unary(UnaryOp, Box<Expr>, Span),
    Call(Box<Expr>, Vec<Expr>, Span),
    Pipeline(Box<Expr>, Box<Expr>, Span),
    Index(Box<Expr>, Box<Expr>, Span),
    Field(Box<Expr>, String, Span),
    Range(Box<Expr>, Box<Expr>, bool, Span), // start, end, inclusive
    Tuple(Vec<Expr>, Span),
    List(Vec<Expr>, Span),
    Lambda(Vec<Param>, Box<Expr>, Span),
    StringInterp(Vec<StringPart>, Span),
    If(Box<Expr>, Block, Option<Box<Expr>>, Span), // cond, then, else
    Match(Box<Expr>, Vec<MatchArm>, Span),
    Block(Block, Span),
    Assign(Box<Expr>, Box<Expr>, Span),
    CompoundAssign(Box<Expr>, BinOp, Box<Expr>, Span),
}

#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Expr>,
    pub body: Expr,
}

#[derive(Debug, Clone)]
pub enum StringPart {
    Lit(String),
    Expr(Expr),
}

#[derive(Debug, Clone)]
pub enum Literal {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    And,
    Or,
    Concat, // ++
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Neg,
    Not,
}

#[derive(Debug, Clone)]
pub enum Pattern {
    Wildcard,
    Ident(String),
    Literal(Literal),
    Tuple(Vec<Pattern>),
    Constructor(String, Vec<Pattern>),
    List(Vec<Pattern>),
    Or(Vec<Pattern>),
}
