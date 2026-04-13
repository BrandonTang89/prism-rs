use crate::ast::{Expr, VarDecl, VarType};

/// Returns the folded integer encoding of a variable's single initial value.
///
/// For bool variables, `false -> 0` and `true -> 1`.
/// This helper assumes analysis has already folded initial expressions to
/// literals compatible with their declared type.
pub fn init_value(var_decl: &VarDecl) -> i32 {
    match (&var_decl.var_type, &*var_decl.init) {
        (VarType::BoundedInt { .. }, Expr::IntLit(v)) => *v,
        (VarType::Bool, Expr::BoolLit(b)) => {
            if *b {
                1
            } else {
                0
            }
        }
        (VarType::Bool, Expr::IntLit(v)) if *v == 0 || *v == 1 => *v,
        _ => panic!(
            "Unsupported folded init expression for variable '{}': {:?}",
            var_decl.name, var_decl.init
        ),
    }
}
