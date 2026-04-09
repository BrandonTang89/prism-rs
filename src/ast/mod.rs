pub struct DTMCModel {
    pub modules: Vec<Module>,
    // constants
    // global vars
    // functions, etc.
}

pub struct Module {
    pub name: String,
    pub local_vars: Vec<VarDecl>,
    pub commands: Vec<Command>,
}

pub struct VarDecl {
    pub name: String,
    pub var_type: VarType,
    pub init: Option<String>,
}

pub enum VarType {
    Int,
    Bool,
}

pub struct Command {
    pub labels: Vec<String>,
    pub guard: Expr,
    pub updates: Vec<ProbUpdate>,
}

pub struct ProbUpdate {
    pub probability: f64,
    pub assignments: Vec<Expr>,
}

pub enum Expr {
    // Literals
    BoolLit(bool),
    IntLit(i32),

    // References
    Ident(String),
    PrimedIdent(String),

    // Operators
    UnaryOp {
        op: UnOp,
        operand: Box<Expr>,
    },
    BinOp {
        lhs: Box<Expr>,
        op: BinOp,
        rhs: Box<Expr>,
    },
    Ternary {
        cond: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Box<Expr>,
    },
}

pub enum UnOp {
    Not,
    Neg,
}

pub enum BinOp {
    And,
    Or,
    Implies,
    Equals,
    NotEquals,
    Less,
    LessEq,
    Greater,
    GreaterEq,
}

/// `module mac2 = mac1 [s1=s2, s2=s1,...] endmodule`
pub struct RenamedModule {
    pub name: String,
    pub base: String,
    pub renames: Vec<(String, String)>,
}
