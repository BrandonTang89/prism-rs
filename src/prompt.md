Okay lets try to clean up the parser


"(" <lhs:Ident> "=" <rhs:Expr> ")" => Box::new(Expr::BinOp {
        lhs: Box::new(Expr::Ident(lhs)),
        op: BinOp::Eq,
        rhs,
    }),
    "(" <lhs:Ident> "!=" <rhs:Expr> ")" => Box::new(Expr::BinOp {
        lhs: Box::new(Expr::Ident(lhs)),
        op: BinOp::Neq,
        rhs,
    }),
    "(" <lhs:Ident> "<" <rhs:Expr> ")" => Box::new(Expr::BinOp {
        lhs: Box::new(Expr::Ident(lhs)),
        op: BinOp::Lt,
        rhs,
    }),
    "(" <lhs:Ident> "<=" <rhs:Expr> ")" => Box::new(Expr::BinOp {
        lhs: Box::new(Expr::Ident(lhs)),
        op: BinOp::Leq,
        rhs,
    }),
    "(" <lhs:Ident> ">" <rhs:Expr> ")" => Box::new(Expr::BinOp {
        lhs: Box::new(Expr::Ident(lhs)),
        op: BinOp::Gt,
        rhs,
    }),
    "(" <lhs:Ident> ">=" <rhs:Expr> ")" => Box::new(Expr::BinOp {
        lhs: Box::new(Expr::Ident(lhs)),
        op: BinOp::Geq,
        rhs,
    }),

I thought this wouldn't be necessary since it should be covered under 
"(" <Expr> ")", and 

#[precedence(level="5")] #[assoc(side="left")]
<lhs:Expr> "<" <rhs:Expr> => Box::new(Expr::BinOp { lhs, op: BinOp::Lt, rhs }),
<lhs:Expr> "<=" <rhs:Expr> => Box::new(Expr::BinOp { lhs, op: BinOp::Leq, rhs }),
<lhs:Expr> ">" <rhs:Expr> => Box::new(Expr::BinOp { lhs, op: BinOp::Gt, rhs }),
<lhs:Expr> ">=" <rhs:Expr> => Box::new(Expr::BinOp { lhs, op: BinOp::Geq, rhs }),

#[precedence(level="6")] #[assoc(side="left")]
<lhs:Expr> "=" <rhs:Expr> => Box::new(Expr::BinOp { lhs, op: BinOp::Eq, rhs }),
<lhs:Expr> "!=" <rhs:Expr> => Box::new(Expr::BinOp { lhs, op: BinOp::Neq, rhs }),

Can you figure out how to remove this or explain why it must be there.

Secondly, the binary operands & | + - * should probably be able to take a list of them
without needing explicit parentheses, e.g. "a & b & c" should be parsed as a single expression with 3 operands, rather than needing to be "(a & b) & c" or "a & (b & c)". This would make the syntax more natural and easier to read. This would also remove the need for the 

very specific fix of

    "(" <lhs:Ident> "|" <mid:Ident> "|" <rhs:Ident> ")" => Box::new(Expr::BinOp {
        lhs: Box::new(Expr::Ident(lhs)),
        op: BinOp::Or,
        rhs: Box::new(Expr::BinOp {
            lhs: Box::new(Expr::Ident(mid)),
            op: BinOp::Or,
            rhs: Box::new(Expr::Ident(rhs)),
        }),
    }),
    "(" <lhs:Ident> "||" <mid:Ident> "||" <rhs:Ident> ")" => Box::new(Expr::BinOp {
        lhs: Box::new(Expr::Ident(lhs)),
        op: BinOp::Or,
        rhs: Box::new(Expr::BinOp {
            lhs: Box::new(Expr::Ident(mid)),
            op: BinOp::Or,
            rhs: Box::new(Expr::Ident(rhs)),
        }),
    }),

Help me to make these changes and ensure that the tests still pass