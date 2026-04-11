use std::collections::{HashMap, HashSet};

use crate::analyze::*;
use crate::ast::*;
use crate::ref_manager::RefManager;
use lumindd::NodeId;
use tracing::debug;

pub struct SymbolicDTMC<'a> {
    /// ManagerRef
    pub mgr: RefManager,

    /// AST
    pub ast: &'a DTMCAst,

    /// Info
    pub info: &'a DTMCModelInfo,

    /// Variable name to DD node IDs, from LSB to MSB
    pub var_curr_nodes: std::collections::HashMap<String, Vec<NodeId>>,
    pub var_next_nodes: std::collections::HashMap<String, Vec<NodeId>>,

    /// DD node ID to human-friendly variable bit label (e.g., s_0, s_1)
    pub dd_var_names: std::collections::HashMap<NodeId, String>,

    /// All primed variables BDD cube
    pub next_var_cube: NodeId,

    /// All current variables BDD cube
    pub curr_var_cube: NodeId,
    
    /// ADD representing the transition relation
    pub transitions: NodeId,
}

impl<'a> SymbolicDTMC<'a> {
    fn new(ast: &'a DTMCAst, info: &'a DTMCModelInfo) -> Self {
        let mut mgr = RefManager::new();
        let transitions = mgr.zero();
        let next_var_cube = mgr.one();
        let curr_var_cube = mgr.one();
        SymbolicDTMC {
            var_curr_nodes: std::collections::HashMap::new(),
            var_next_nodes: std::collections::HashMap::new(),
            dd_var_names: std::collections::HashMap::new(),
            mgr,
            transitions,
            next_var_cube,
            curr_var_cube,
            ast,
            info,
        }
    }
}

fn allocate_dd_vars(dtmc: &mut SymbolicDTMC) {
    for module in &dtmc.ast.modules {
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

            let mgr = &mut dtmc.mgr;

            // Interleaved ordering
            let nodes: Vec<NodeId> = (0..num_bits * 2).map(|_| mgr.new_var()).collect();
            let curr_nodes: Vec<NodeId> = nodes.chunks(2).map(|c| c[0]).collect();
            let next_nodes: Vec<NodeId> = nodes.chunks(2).map(|c| c[1]).collect();

            for (i, &curr) in curr_nodes.iter().enumerate() {
                dtmc.dd_var_names
                    .insert(curr, format!("{}_{}", var_name, i));
            }
            for (i, &next) in next_nodes.iter().enumerate() {
                dtmc.dd_var_names
                    .insert(next, format!("{}'_{}", var_name, i));
            }

            dtmc.curr_var_cube = curr_nodes.iter().fold(dtmc.curr_var_cube, |cube, &node| {
                mgr.ref_node(node);
                mgr.bdd_and(cube, node)
            });
            dtmc.next_var_cube = next_nodes.iter().fold(dtmc.next_var_cube, |cube, &node| {
                mgr.ref_node(node);
                mgr.bdd_and(cube, node)
            });

            dtmc.var_curr_nodes.insert(var_name.clone(), curr_nodes);
            dtmc.var_next_nodes.insert(var_name.clone(), next_nodes);

            debug!(
                "Allocated variable '{}' with current BDD variables: {:?}",
                var_name, dtmc.var_curr_nodes[var_name]
            );
            debug!(
                "Allocated variable '{}' with next BDD variables: {:?}",
                var_name, dtmc.var_next_nodes[var_name]
            );
        }
    }
}

#[derive(Debug)]
struct SymbolicCommand {
    transition: NodeId,
}

#[derive(Debug)]
struct SymbolicModule {
    ident: NodeId,
    commands_by_action: std::collections::HashMap<String, Vec<SymbolicCommand>>,
}

fn get_variable_encoding(dtmc: &mut SymbolicDTMC, var_name: &str, primed: bool) -> NodeId {
    let (lo, _) = dtmc
        .info
        .var_bounds
        .get(var_name)
        .expect(&format!("Variable '{}' not found in model info", var_name));

    let mgr = &mut dtmc.mgr;
    let offset_add = mgr.add_const(*lo as f64);
    let variable_nodes = if primed {
        &dtmc.var_next_nodes[var_name]
    } else {
        &dtmc.var_curr_nodes[var_name]
    };
    let encoding = mgr.get_encoding(&variable_nodes);
    mgr.add_plus(encoding, offset_add)
}

/// Returns a referenced ADD representing the expression
fn translate_expr(expr: &Expr, dtmc: &mut SymbolicDTMC) -> NodeId {
    match expr {
        Expr::IntLit(i) => dtmc.mgr.add_const(*i as f64),
        Expr::FloatLit(f) => dtmc.mgr.add_const(*f),
        Expr::BoolLit(b) => dtmc.mgr.add_const(if *b { 1.0 } else { 0.0 }),
        Expr::Ident(name) => get_variable_encoding(dtmc, name, false),
        Expr::PrimedIdent(name) => get_variable_encoding(dtmc, name, true),
        Expr::UnaryOp { op, operand } => {
            let value = translate_expr(operand, dtmc);
            match op {
                UnOp::Not => {
                    let one = dtmc.mgr.add_const(1.0);
                    dtmc.mgr.add_minus(one, value)
                }
                UnOp::Neg => {
                    let zero = dtmc.mgr.add_const(0.0);
                    dtmc.mgr.add_minus(zero, value)
                }
            }
        }
        Expr::BinOp { lhs, op, rhs } => {
            let left = translate_expr(lhs, dtmc);
            let right = translate_expr(rhs, dtmc);
            match op {
                BinOp::Plus => dtmc.mgr.add_plus(left, right),
                BinOp::Minus => dtmc.mgr.add_minus(left, right),
                BinOp::Mul => dtmc.mgr.add_times(left, right),
                BinOp::Div => dtmc.mgr.add_divide(left, right),
                BinOp::Eq => {
                    let bdd = dtmc.mgr.add_equals(left, right);
                    dtmc.mgr.bdd_to_add(bdd)
                }
                BinOp::Neq => {
                    let bdd = dtmc.mgr.add_nequals(left, right);
                    dtmc.mgr.bdd_to_add(bdd)
                }
                BinOp::Lt => {
                    let bdd = dtmc.mgr.add_less_than(left, right);
                    dtmc.mgr.bdd_to_add(bdd)
                }
                BinOp::Leq => {
                    let bdd = dtmc.mgr.add_less_or_equal(left, right);
                    dtmc.mgr.bdd_to_add(bdd)
                }
                BinOp::Gt => {
                    let bdd = dtmc.mgr.add_greater_than(left, right);
                    dtmc.mgr.bdd_to_add(bdd)
                }
                BinOp::Geq => {
                    let bdd = dtmc.mgr.add_greater_or_equal(left, right);
                    dtmc.mgr.bdd_to_add(bdd)
                }
                BinOp::And => dtmc.mgr.add_times(left, right),
                BinOp::Or => {
                    let bdd_left = dtmc.mgr.add_to_bdd(left);
                    let bdd_right = dtmc.mgr.add_to_bdd(right);
                    let bdd_or = dtmc.mgr.bdd_or(bdd_left, bdd_right);
                    dtmc.mgr.bdd_to_add(bdd_or)
                }
            }
        }
        Expr::Ternary {
            cond,
            then_branch,
            else_branch,
        } => {
            let cond_add = translate_expr(cond, dtmc);
            let then_add = translate_expr(then_branch, dtmc);
            let else_add = translate_expr(else_branch, dtmc);
            dtmc.mgr.add_ite(cond_add, then_add, else_add)
        }
    }
}

fn get_assign_target(expr: &Expr) -> Option<&str> {
    match expr {
        Expr::BinOp {
            lhs, op: BinOp::Eq, ..
        } => match &**lhs {
            Expr::PrimedIdent(name) => Some(name.as_str()),
            _ => None,
        },
        _ => None,
    }
}

fn translate_update(
    update: &ProbUpdate,
    module_local_vars: &[String],
    dtmc: &mut SymbolicDTMC,
) -> NodeId {
    let prob = translate_expr(&update.prob, dtmc);

    let assigned_vars: HashSet<String> = update
        .assignments
        .iter()
        .filter_map(|assignment| get_assign_target(assignment).map(|name| name.to_string()))
        .collect();

    // For variables that are updated, use the translated expression
    let symbolic_updates = update
        .assignments
        .iter()
        .map(|assignment| translate_expr(&*assignment, dtmc))
        .collect::<Vec<_>>();
    let mgr = &mut dtmc.mgr;
    let mut assign = symbolic_updates
        .iter()
        .fold(mgr.one(), |acc, &result| mgr.add_times(acc, result));

    // For variables that are not assigned, ensure they remain unchanged
    for var_name in module_local_vars {
        if assigned_vars.contains(var_name) {
            continue;
        }
        let curr_nodes = dtmc.var_curr_nodes[var_name].clone();
        let next_nodes = dtmc.var_next_nodes[var_name].clone();
        for (curr, next) in curr_nodes.into_iter().zip(next_nodes.into_iter()) {
            mgr.ref_node(curr);
            mgr.ref_node(next);
            let eq = mgr.bdd_equals(curr, next);
            assign = mgr.add_times(assign, eq);
        }
    }

    dtmc.mgr.add_times(prob, assign)
}

fn translate_command(
    cmd: &Command,
    module_local_vars: &[String],
    dtmc: &mut SymbolicDTMC,
) -> SymbolicCommand {
    let guard = translate_expr(&cmd.guard, dtmc);
    let updates = cmd
        .updates
        .iter()
        .map(|update| translate_update(update, module_local_vars, dtmc))
        .collect::<Vec<_>>();

    let mgr = &mut dtmc.mgr;
    let updates_sum = updates
        .iter()
        .fold(mgr.zero(), |acc, &update| mgr.add_plus(acc, update));
    let transition = mgr.add_times(guard, updates_sum);

    SymbolicCommand { transition }
}

fn translate_module(module: &Module, dtmc: &mut SymbolicDTMC) -> SymbolicModule {
    let mgr = &mut dtmc.mgr;
    let module_local_vars = module
        .local_vars
        .iter()
        .map(|v| v.name.clone())
        .collect::<Vec<_>>();

    let mut ident = mgr.one();
    for var_name in module.local_vars.iter().map(|v| &v.name) {
        let curr_nodes = &dtmc.var_curr_nodes[var_name];
        let next_nodes = &dtmc.var_next_nodes[var_name];

        ident = curr_nodes
            .iter()
            .zip(next_nodes.iter())
            .fold(ident, |acc, (&curr, &next)| {
                mgr.ref_node(curr);
                mgr.ref_node(next);
                let eq = mgr.bdd_equals(curr, next); // curr == next
                mgr.bdd_and(acc, eq)
            });
    }

    let mut commands_by_action: std::collections::HashMap<String, Vec<SymbolicCommand>> =
        std::collections::HashMap::new();
    for cmd in &module.commands {
        let symbolic_cmd = translate_command(cmd, &module_local_vars, dtmc);
        assert!(
            cmd.labels.len() == 1,
            "DTMCs should have exactly one label per command after analysis"
        );
        let action = &cmd.labels[0];
        commands_by_action
            .entry(action.clone())
            .or_insert_with(Vec::new)
            .push(symbolic_cmd);
    }

    SymbolicModule {
        ident,
        commands_by_action,
    }
}

fn translate_modules(dtmc: &mut SymbolicDTMC) -> HashMap<String, SymbolicModule> {
    dtmc.ast
        .modules
        .iter()
        .map(|module| (module.name.clone(), translate_module(module, dtmc)))
        .collect()
}

fn translate_dtmc(dtmc: &mut SymbolicDTMC) {
    let symbolic_modules = translate_modules(dtmc);

    let mut transitions = dtmc.mgr.zero();
    for (act, act_modules) in dtmc.info.modules_of_act.iter() {
        debug!("Action '{}' is part of {:?}", act, act_modules);
        let mut act_trans = dtmc.mgr.one();
        for module_name in dtmc.ast.modules.iter().map(|m| &m.name) {
            if act_modules.contains(module_name) {
                let mut act_mod_trans = dtmc.mgr.zero();
                for cmd in &symbolic_modules[module_name].commands_by_action[act] {
                    act_mod_trans = dtmc.mgr.add_plus(act_mod_trans, cmd.transition);
                }
                act_trans = dtmc.mgr.add_times(act_trans, act_mod_trans);
            } else {
                // If the module doesn't have a command with this action, it should stay the same
                let ident = symbolic_modules[module_name].ident;
                act_trans = dtmc.mgr.add_times(act_trans, ident);
            }
        }
        transitions = dtmc.mgr.add_plus(transitions, act_trans);
    }

    transitions = dtmc.mgr.unif(transitions, dtmc.next_var_cube);
    dtmc.transitions = transitions;
}

pub fn build_symbolic_dtmc<'a>(
    ast: &'a DTMCAst,
    model_info: &'a DTMCModelInfo,
) -> SymbolicDTMC<'a> {
    let mut dtmc = SymbolicDTMC::new(ast, model_info);
    allocate_dd_vars(&mut dtmc);
    translate_dtmc(&mut dtmc);
    dtmc.mgr
        .dump_add_dot(dtmc.transitions, "tmp.dot", &dtmc.dd_var_names)
        .unwrap();
    dtmc
}
