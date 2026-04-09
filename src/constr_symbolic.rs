use crate::analyze::*;
use crate::ast::*;
use lumindd::{Manager, NodeId};
use tracing::debug;

pub struct SymbolicDTMC<'a> {
    /// ManagerRef
    pub manager: Manager,

    /// AST
    pub ast: &'a DTMCAst,

    /// Info
    pub info: &'a DTMCModelInfo,

    /// Variable name to DD node IDs, from LSB to MSB
    pub var_curr_nodes: std::collections::HashMap<String, Vec<NodeId>>,
    pub var_next_nodes: std::collections::HashMap<String, Vec<NodeId>>,

    /// ADD representing the transition relation
    pub transitions: NodeId,
}

fn allocate_dd_vars(symbolic_dtmc: &mut SymbolicDTMC) {
    for module in &symbolic_dtmc.ast.modules {
        for var_decl in &module.local_vars {
            let var_name = &var_decl.name;
            let var_type = &var_decl.var_type;
            let num_bits = match var_type {
                VarType::Bool => 1,
                VarType::BoundedInt { lo, hi } => {
                    // For simplicity, we assume lo and hi are integer literals
                    let lo_val = match **lo {
                        Expr::IntLit(val) => val,
                        _ => panic!("Expected integer literal for variable bounds"),
                    };
                    let hi_val = match **hi {
                        Expr::IntLit(val) => val,
                        _ => panic!("Expected integer literal for variable bounds"),
                    };
                    let range_size = hi_val - lo_val + 1;

                    match range_size {
                        0 => panic!("Invalid variable bounds: lo must be <= hi"),
                        1 => panic!("Variable '{}' has only one possible value", var_name), // No bits needed for a single value
                        _ => (range_size - 1).ilog2() + 1,
                    }
                }
            };

            let mgr = &mut symbolic_dtmc.manager;

            // Interleaved ordering
            let nodes: Vec<NodeId> = (0..num_bits * 2).map(|_| mgr.bdd_new_var()).collect();
            let curr_nodes: Vec<NodeId> = nodes.chunks(2).map(|c| c[0]).collect();
            let next_nodes: Vec<NodeId> = nodes.chunks(2).map(|c| c[1]).collect();

            symbolic_dtmc
                .var_curr_nodes
                .insert(var_name.clone(), curr_nodes);
            symbolic_dtmc
                .var_next_nodes
                .insert(var_name.clone(), next_nodes);

            debug!(
                "Allocated variable '{}' with current BDD variables: {:?}",
                var_name, symbolic_dtmc.var_curr_nodes[var_name]
            );
            debug!(
                "Allocated variable '{}' with next BDD variables: {:?}",
                var_name, symbolic_dtmc.var_next_nodes[var_name]
            );
        }
    }
}

struct SymbolicCommand {
    transition: NodeId,
}

struct SymbolicModule {
    ident: NodeId,
    commands: Vec<NodeId>,
}

fn translate_module(module: &Module, symbolic_dtmc: &mut SymbolicDTMC) -> SymbolicModule {
    let mgr = &mut symbolic_dtmc.manager;
    SymbolicModule {
        ident: NodeId::ZERO,
        commands: vec![],
    }
}

pub fn build_symbolic_dtmc<'a>(
    ast: &'a DTMCAst,
    model_info: &'a DTMCModelInfo,
) -> SymbolicDTMC<'a> {
    let mut symbolic_info = SymbolicDTMC {
        var_curr_nodes: std::collections::HashMap::new(),
        var_next_nodes: std::collections::HashMap::new(),
        manager: Manager::new(),
        transitions: NodeId::ZERO,
        ast,
        info: model_info,
    };

    allocate_dd_vars(&mut symbolic_info);

    symbolic_info
}
