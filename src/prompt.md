You are on the right track.

I think this can be optimised.

fn extract_inputs_from_init_bdd(dtmc: &mut SymbolicDTMC, init: BddNode) -> Vec<i32> {
    let mut inputs = vec![0_i32; dtmc.mgr.var_count()];
    let num_curr = dtmc.curr_var_indices.len() as u32;

    for module in &dtmc.ast.modules {
        for var_decl in &module.local_vars {
            for &bit in &dtmc.var_curr_nodes[&var_decl.name] {
                dtmc.mgr.ref_node(init.0);
                dtmc.mgr.ref_node(bit);
                let intersects_true = dtmc.mgr.bdd_and(init, BddNode(bit));
                let minterms = dtmc.mgr.bdd_count_minterms(intersects_true, num_curr);
                dtmc.mgr.deref_node(intersects_true.0);

                let var_index = dtmc.mgr.read_var_index(bit) as usize;
                inputs[var_index] = if minterms > 0 { 1 } else { 0 };
            }
        }
    }

    inputs
}

I think this can be generalised to "extract_leftmost_path_from_bdd" (put good doc comments because I think its difficult to give a good name)
To do so, we use a loop where at each iteration, we check if going to the else child results in a non-zero BDD, if so, we go to the else child, otherwise we go to the then child. We keep track of the variable indices we encounter along the way, and whether we took the then or else branch for each variable. Once we reach a terminal node, we can reconstruct an input valuation from the path we took. Note that for the rest of the variables that we did not encouter along the path, we can assign them any value (e.g. 0) since they do not affect the outcome of the BDD.

This can then just be put as a utility function in the ref_manager.rs file rather than specialized for the init state extraction.

Once you do that, ensure all tests continue to pass, and then update the project's readme on the status of things that are working and todo within the project, as well as any instructions on how to use the binary as an end-user.