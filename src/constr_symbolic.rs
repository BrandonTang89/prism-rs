use std::collections::{HashMap, HashSet};

use crate::analyze::*;
use crate::ast::*;
use crate::reachability::compute_reachable_and_filter;
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

    /// 0-1 BDD of transitions after reachability filtering
    pub transitions_01_bdd: NodeId,
}

impl<'a> SymbolicDTMC<'a> {
    fn new(ast: &'a DTMCAst, info: &'a DTMCModelInfo) -> Self {
        let mut mgr = RefManager::new();
        let transitions = mgr.add_zero();
        let transitions_01_bdd = mgr.bdd_zero();
        let next_var_cube = mgr.bdd_one();
        let curr_var_cube = mgr.bdd_one();
        SymbolicDTMC {
            var_curr_nodes: std::collections::HashMap::new(),
            var_next_nodes: std::collections::HashMap::new(),
            dd_var_names: std::collections::HashMap::new(),
            mgr,
            transitions,
            transitions_01_bdd,
            next_var_cube,
            curr_var_cube,
            ast,
            info,
        }
    }

    pub fn describe(&mut self) -> String {
        let mut desc = String::new();
        desc.push_str(&format!("Variables:\n"));
        for (var_name, curr_nodes) in &self.var_curr_nodes {
            let next_nodes = &self.var_next_nodes[var_name];
            let var_desc = format!(
                "  {}: curr nodes {:?}, next nodes {:?}\n",
                var_name, curr_nodes, next_nodes
            );
            desc.push_str(&var_desc);
        }
        desc.push_str(&format!(
            "Transitions ADD node ID: {:?}\n",
            self.transitions
        ));
        desc.push_str(&format!(
            "Transitions 0-1 BDD node ID: {:?}\n",
            self.transitions_01_bdd
        ));

        let num_curr_bits: usize = self.var_curr_nodes.values().map(|v| v.len()).sum();
        let num_next_bits: usize = self.var_next_nodes.values().map(|v| v.len()).sum();
        let num_vars = (num_curr_bits + num_next_bits) as u32;
        let stats = self.mgr.add_stats(self.transitions, num_vars);

        desc.push_str(&format!(
            "Num Nodes ADD: {}, Num Terminals: {}, Transitions(minterms): {}\n",
            stats.node_count, stats.terminal_count, stats.minterms
        ));
        desc
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
                "Allocated var '{}' with curr BDD vars: {:?}",
                var_name, dtmc.var_curr_nodes[var_name]
            );
            debug!(
                "Allocated var '{}' with next BDD vars: {:?}",
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
    let add_one = mgr.add_const(1.0);
    let mut assign = symbolic_updates
        .iter()
        .fold(add_one, |acc, &result| mgr.add_times(acc, result));

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
            let eq_bdd = mgr.bdd_equals(curr, next);
            let eq_add = mgr.bdd_to_add(eq_bdd);
            assign = mgr.add_times(assign, eq_add);
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
        .fold(mgr.add_zero(), |acc, &update| mgr.add_plus(acc, update));
    let transition = mgr.add_times(guard, updates_sum);

    SymbolicCommand { transition }
}

fn translate_module(module: &Module, dtmc: &mut SymbolicDTMC) -> SymbolicModule {
    let module_local_vars = module
        .local_vars
        .iter()
        .map(|v| v.name.clone())
        .collect::<Vec<_>>();

    let mut ident = dtmc.mgr.bdd_one();
    for var_name in module.local_vars.iter().map(|v| &v.name) {
        let curr_nodes = dtmc.var_curr_nodes[var_name].clone();
        let next_nodes = dtmc.var_next_nodes[var_name].clone();

        for (curr, next) in curr_nodes.into_iter().zip(next_nodes.into_iter()) {
            dtmc.mgr.ref_node(curr);
            dtmc.mgr.ref_node(next);
            let eq = dtmc.mgr.bdd_equals(curr, next); // curr == next
            ident = dtmc.mgr.bdd_and(ident, eq);
        }
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

    let ident = dtmc.mgr.bdd_to_add(ident);

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

    let mut transitions = dtmc.mgr.add_zero();
    for (act, act_modules) in dtmc.info.modules_of_act.iter() {
        debug!("Action '{}' is part of {:?}", act, act_modules);
        let mut act_trans = dtmc.mgr.add_const(1.0);
        for module_name in dtmc.ast.modules.iter().map(|m| &m.name) {
            if act_modules.contains(module_name) {
                let mut act_mod_trans = dtmc.mgr.add_zero();
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
    compute_reachable_and_filter(&mut dtmc);
    dtmc.mgr
        .dump_add_dot(dtmc.transitions, "tmp.dot", &dtmc.dd_var_names)
        .unwrap();
    dtmc
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use lumindd::{Manager, NodeId};

    use crate::{analyze::analyze_dtmc, parser::parse_dtmc};

    use super::*;

    fn eval_add(mgr: &Manager, node: NodeId, assignment: &HashMap<u16, bool>) -> f64 {
        let reg = node.regular();
        let var = mgr.read_var_index(reg);
        if var == u16::MAX {
            return mgr.add_value(reg).expect("terminal ADD value expected");
        }
        let take_then = assignment.get(&var).copied().unwrap_or(false);
        let child = if take_then {
            mgr.read_then(reg)
        } else {
            mgr.read_else(reg)
        };
        eval_add(mgr, child, assignment)
    }

    fn set_bits(mgr: &Manager, vars: &[NodeId], value: i32, assignment: &mut HashMap<u16, bool>) {
        for (i, node) in vars.iter().enumerate() {
            let idx = mgr.read_var_index(node.regular());
            assignment.insert(idx, (value & (1 << i)) != 0);
        }
    }

    fn trans_prob(dtmc: &SymbolicDTMC, s: i32, sp: i32) -> f64 {
        let mgr = &dtmc.mgr.inner;
        let mut assignment = HashMap::new();
        set_bits(mgr, &dtmc.var_curr_nodes["s"], s, &mut assignment);
        set_bits(mgr, &dtmc.var_next_nodes["s"], sp, &mut assignment);
        eval_add(mgr, dtmc.transitions, &assignment)
    }

    fn trans_prob_knuth(dtmc: &SymbolicDTMC, s: i32, d: i32, sp: i32, dp: i32) -> f64 {
        let mgr = &dtmc.mgr.inner;
        let mut assignment = HashMap::new();
        set_bits(mgr, &dtmc.var_curr_nodes["s"], s, &mut assignment);
        set_bits(mgr, &dtmc.var_curr_nodes["d"], d, &mut assignment);
        set_bits(mgr, &dtmc.var_next_nodes["s"], sp, &mut assignment);
        set_bits(mgr, &dtmc.var_next_nodes["d"], dp, &mut assignment);
        eval_add(mgr, dtmc.transitions, &assignment)
    }

    #[test]
    fn simple_dtmc_rows_are_correct() {
        let model = include_str!("../tests/dtmc/simple_dtmc.prism");
        let mut ast = parse_dtmc(model).expect("parse failed");
        let info = analyze_dtmc(&mut ast).expect("analysis failed");
        let dtmc = build_symbolic_dtmc(&ast, &info);

        let p01 = trans_prob(&dtmc, 0, 1);
        let p02 = trans_prob(&dtmc, 0, 2);
        let p00 = trans_prob(&dtmc, 0, 0);
        let p10 = trans_prob(&dtmc, 1, 0);
        let p12 = trans_prob(&dtmc, 1, 2);
        let p11 = trans_prob(&dtmc, 1, 1);
        let p20 = trans_prob(&dtmc, 2, 0);
        let p21 = trans_prob(&dtmc, 2, 1);
        let p22 = trans_prob(&dtmc, 2, 2);

        assert!((p01 - 0.5).abs() < 1e-9, "p01={p01}");
        assert!((p02 - 0.5).abs() < 1e-9, "p02={p02}");
        assert!((p00 - 0.0).abs() < 1e-9, "p00={p00}");

        assert!((p10 - 0.5).abs() < 1e-9, "p10={p10}");
        assert!((p12 - 0.5).abs() < 1e-9, "p12={p12}");
        assert!((p11 - 0.0).abs() < 1e-9, "p11={p11}");

        assert!((p20 - 0.5).abs() < 1e-9, "p20={p20}");
        assert!((p21 - 0.5).abs() < 1e-9, "p21={p21}");
        assert!((p22 - 0.0).abs() < 1e-9, "p22={p22}");
    }

    #[test]
    fn simple_dtmc_invalid_state_is_not_inf() {
        let model = include_str!("../tests/dtmc/simple_dtmc.prism");
        let mut ast = parse_dtmc(model).expect("parse failed");
        let info = analyze_dtmc(&mut ast).expect("analysis failed");
        let dtmc = build_symbolic_dtmc(&ast, &info);

        let p = trans_prob(&dtmc, 3, 0);
        assert!(
            p.is_finite(),
            "invalid encoded state produced non-finite value: {p}"
        );
    }

    #[test]
    fn knuth_die_transition_count_matches_prism() {
        let model = include_str!("../tests/dtmc/knuth_die.prism");
        let mut ast = parse_dtmc(model).expect("parse failed");
        let info = analyze_dtmc(&mut ast).expect("analysis failed");
        let dtmc = build_symbolic_dtmc(&ast, &info);

        let reachable_states = [
            (0, 0),
            (1, 0),
            (2, 0),
            (3, 0),
            (4, 0),
            (5, 0),
            (6, 0),
            (7, 1),
            (7, 2),
            (7, 3),
            (7, 4),
            (7, 5),
            (7, 6),
        ];

        let mut transitions = 0usize;
        for (s, d) in reachable_states {
            let mut row_sum = 0.0;
            for sp in 0..=7 {
                for dp in 0..=6 {
                    let p = trans_prob_knuth(&dtmc, s, d, sp, dp);
                    assert!(
                        p.is_finite(),
                        "non-finite prob at ({s},{d})->({sp},{dp}): {p}"
                    );
                    if p > 1e-12 {
                        transitions += 1;
                        row_sum += p;
                    }
                }
            }
            assert!(
                (row_sum - 1.0).abs() < 1e-9,
                "row sum for ({s},{d}) is {row_sum}"
            );
        }

        // PRISM reports 20 transitions for this model.
        assert_eq!(transitions, 20, "transition count mismatch: {transitions}");

        // Example unreachable state: (s=1,d=1) should have no outgoing mass after filtering.
        let mut unreachable_row_sum = 0.0;
        for sp in 0..=7 {
            for dp in 0..=6 {
                unreachable_row_sum += trans_prob_knuth(&dtmc, 1, 1, sp, dp);
            }
        }
        assert!(
            unreachable_row_sum.abs() < 1e-12,
            "unreachable row not filtered: {unreachable_row_sum}"
        );
    }
}
