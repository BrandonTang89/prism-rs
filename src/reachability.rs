use crate::ast::utils::init_value;
use crate::ref_manager::BddNode;
use crate::symbolic_dtmc::SymbolicDTMC;

/// Builds the initial-state BDD over current-state variables.
///
/// Analysis already guarantees folded literal inits and in-range values.
/// The assertions below therefore check internal consistency only.
pub fn build_init_bdd(dtmc: &mut SymbolicDTMC) -> BddNode {
    let mut init = dtmc.mgr.bdd_one();

    for module in &dtmc.ast.modules {
        for var_decl in &module.local_vars {
            let var_name = var_decl.name.clone();
            let (lo, hi) = dtmc.info.var_bounds[&var_name];
            let init_val = init_value(var_decl);
            assert!(init_val >= lo && init_val <= hi);

            let encoded = (init_val - lo) as u32;
            let curr_nodes = dtmc.var_curr_nodes[&var_name].clone();
            for (i, bit) in curr_nodes.into_iter().enumerate() {
                dtmc.mgr.ref_node(bit);
                let lit = if (encoded & (1u32 << i)) != 0 {
                    BddNode(bit)
                } else {
                    dtmc.mgr.bdd_not(BddNode(bit))
                };
                init = dtmc.mgr.bdd_and(init, lit);
            }
        }
    }

    debug_assert_eq!(
        dtmc.mgr
            .bdd_count_minterms(init, dtmc.curr_var_indices.len() as u32),
        1
    );

    init
}

fn build_identity_transition_bdd(dtmc: &mut SymbolicDTMC) -> BddNode {
    let mut ident = dtmc.mgr.bdd_one();
    for module in &dtmc.ast.modules {
        for var_decl in &module.local_vars {
            let var_name = &var_decl.name;
            let curr_nodes = dtmc.var_curr_nodes[var_name].clone();
            let next_nodes = dtmc.var_next_nodes[var_name].clone();
            for (curr, next) in curr_nodes.into_iter().zip(next_nodes.into_iter()) {
                dtmc.mgr.ref_node(curr);
                dtmc.mgr.ref_node(next);
                let eq = dtmc.mgr.bdd_equals(BddNode(curr), BddNode(next));
                ident = dtmc.mgr.bdd_and(ident, eq);
            }
        }
    }
    ident
}

fn add_dead_end_self_loops(dtmc: &mut SymbolicDTMC, reachable: BddNode) {
    dtmc.mgr.ref_node(dtmc.transitions_01_add.0);
    let out_curr = dtmc
        .mgr
        .bdd_or_abstract(dtmc.transitions_01_add, dtmc.next_var_cube);

    dtmc.mgr.ref_node(out_curr.0);
    let not_out_curr = dtmc.mgr.bdd_not(out_curr);

    dtmc.mgr.ref_node(reachable.0);
    let dead_end_curr = dtmc.mgr.bdd_and(reachable, not_out_curr);
    dtmc.mgr.deref_node(out_curr.0);

    dtmc.mgr.ref_node(dead_end_curr.0);
    let dead_end_add_for_count = dtmc.mgr.bdd_to_add(dead_end_curr);
    dtmc.mgr.ref_node(dtmc.curr_var_cube.0);
    let curr_cube_add = dtmc.mgr.bdd_to_add(dtmc.curr_var_cube);
    let dead_end_count_add = dtmc
        .mgr
        .add_sum_abstract(dead_end_add_for_count, curr_cube_add);
    dtmc.mgr.deref_node(curr_cube_add.0);
    let dead_end_count = dtmc
        .mgr
        .add_value(dead_end_count_add.0)
        .unwrap_or(0.0)
        .round() as u64;
    dtmc.mgr.deref_node(dead_end_count_add.0);

    if dead_end_count > 0 {
        let curr_next_eq = build_identity_transition_bdd(dtmc);
        dtmc.mgr.ref_node(dead_end_curr.0);
        let self_loops = dtmc.mgr.bdd_and(dead_end_curr, curr_next_eq);

        let old_rel = dtmc.transitions_01_add;
        dtmc.mgr.ref_node(old_rel.0);
        dtmc.mgr.ref_node(self_loops.0);
        dtmc.transitions_01_add = dtmc.mgr.bdd_or(old_rel, self_loops);
        dtmc.mgr.deref_node(old_rel.0);

        dtmc.mgr.ref_node(self_loops.0);
        let self_loops_add = dtmc.mgr.bdd_to_add(self_loops);
        let old_add = dtmc.transitions;
        dtmc.mgr.ref_node(old_add.0);
        dtmc.transitions = dtmc.mgr.add_plus(old_add, self_loops_add);
        dtmc.mgr.deref_node(old_add.0);
        dtmc.mgr.deref_node(self_loops.0);
    }

    println!("Added self-loops to {} dead-end states", dead_end_count);
    dtmc.mgr.deref_node(dead_end_curr.0);
}

pub fn compute_reachable_and_filter(dtmc: &mut SymbolicDTMC) {
    let init = build_init_bdd(dtmc);
    dtmc.mgr.deref_node(dtmc.init.0);
    dtmc.mgr.ref_node(init.0);
    dtmc.init = init;

    dtmc.mgr.ref_node(init.0);
    let mut reachable = init;

    dtmc.mgr.ref_node(dtmc.transitions.0);
    let trans_rel = dtmc.mgr.add_to_bdd(dtmc.transitions);

    let mut iterations = 0usize;

    loop {
        iterations += 1;
        let old = reachable;

        dtmc.mgr.ref_node(old.0);
        dtmc.mgr.ref_node(trans_rel.0);
        let image_next = dtmc
            .mgr
            .bdd_and_abstract(old, trans_rel, dtmc.curr_var_cube);
        let image_curr =
            dtmc.mgr
                .bdd_swap_variables(image_next, &dtmc.next_var_indices, &dtmc.curr_var_indices);
        let new_reachable = dtmc.mgr.bdd_or(old, image_curr);

        reachable = new_reachable;
        if new_reachable == old {
            break;
        }
    }

    dtmc.mgr.deref_node(dtmc.reachable.0);
    dtmc.mgr.ref_node(reachable.0);
    dtmc.reachable = reachable;

    let reachable_states = dtmc.reachable_state_count();

    println!(
        "Reachability (BFS): {} iterations, reachable states: {}",
        iterations, reachable_states
    );

    dtmc.mgr.ref_node(reachable.0);
    let reachable_add = dtmc.mgr.bdd_to_add(reachable);
    let old_transitions = dtmc.transitions;
    dtmc.mgr.ref_node(old_transitions.0);
    dtmc.transitions = dtmc.mgr.add_times(old_transitions, reachable_add);
    dtmc.mgr.deref_node(old_transitions.0);

    dtmc.mgr.ref_node(dtmc.transitions.0);
    let filtered_01 = dtmc.mgr.add_to_bdd(dtmc.transitions);
    let old = dtmc.transitions_01_add;
    dtmc.transitions_01_add = filtered_01;
    dtmc.mgr.deref_node(old.0);

    add_dead_end_self_loops(dtmc, reachable);

    dtmc.mgr.deref_node(reachable.0);
    dtmc.mgr.deref_node(init.0);
    dtmc.mgr.deref_node(trans_rel.0);
}
