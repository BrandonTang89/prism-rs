use std::collections::HashMap;

use tracing::error;

use crate::analyze::DTMCModelInfo;
use crate::ast::DTMCAst;
use crate::ref_manager::{AddNode, BddNode, Node, RefManager};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefLeakReport {
    pub nonzero_ref_count: usize,
}

/// Symbolic DTMC representation used by construction and analysis passes.
///
/// The DD manager and all DD roots are owned here so the structure can cleanly
/// release references on drop.
pub struct SymbolicDTMC {
    /// Decision diagram manager with reference-tracking wrappers.
    pub mgr: RefManager,

    /// Owned model AST.
    pub ast: DTMCAst,

    /// Owned model analysis information.
    pub info: DTMCModelInfo,

    /// Variable name -> current-state DD bit nodes (LSB..MSB).
    pub var_curr_nodes: HashMap<String, Vec<Node>>,
    /// Variable name -> next-state DD bit nodes (LSB..MSB).
    pub var_next_nodes: HashMap<String, Vec<Node>>,

    /// Current-state variable indices aligned with `next_var_indices`.
    pub curr_var_indices: Vec<u16>,
    /// Next-state variable indices aligned with `curr_var_indices`.
    pub next_var_indices: Vec<u16>,

    /// DD node -> human-friendly name used in DOT output.
    pub dd_var_names: HashMap<Node, String>,

    /// 0-1 ADD cube over all next-state variables.
    pub next_var_cube: BddNode,
    /// 0-1 ADD cube over all current-state variables.
    pub curr_var_cube: BddNode,

    /// ADD transition relation P(s,s').
    pub transitions: AddNode,
    /// 0-1 ADD support of filtered transitions.
    pub transitions_01_add: BddNode,

    /// Initial state over current-state variables as a 0-1 BDD.
    pub init: BddNode,

    /// Reachable states over current-state variables as a 0-1 BDD.
    pub reachable: BddNode,

    /// Cached BDD for `(curr == next)` over all state bits.
    pub curr_next_identity: Option<BddNode>,

    released: bool,
}

impl SymbolicDTMC {
    /// Create an empty symbolic DTMC and allocate base roots.
    pub fn new(ast: DTMCAst, info: DTMCModelInfo) -> Self {
        let mut mgr = RefManager::new();
        let transitions = mgr.add_zero();
        let transitions_01_add = mgr.bdd_zero();
        let next_var_cube = mgr.bdd_one();
        let curr_var_cube = mgr.bdd_one();
        let init = mgr.bdd_zero();
        let reachable = mgr.bdd_zero();

        Self {
            mgr,
            ast,
            info,
            var_curr_nodes: HashMap::new(),
            var_next_nodes: HashMap::new(),
            curr_var_indices: Vec::new(),
            next_var_indices: Vec::new(),
            dd_var_names: HashMap::new(),
            next_var_cube,
            curr_var_cube,
            transitions,
            transitions_01_add,
            init,
            reachable,
            curr_next_identity: None,
            released: false,
        }
    }

    /// Number of state variables in the current/next encoding.
    pub fn state_variable_counts(&self) -> (u32, u32) {
        let curr = self.var_curr_nodes.values().map(|v| v.len() as u32).sum();
        let next = self.var_next_nodes.values().map(|v| v.len() as u32).sum();
        (curr, next)
    }

    /// Total number of variables used
    pub fn total_variable_count(&self) -> u32 {
        self.state_variable_counts().0 + self.state_variable_counts().1
    }

    /// Number of reachable states in the DTMC
    pub fn reachable_state_count(&mut self) -> u64 {
        self.mgr
            .bdd_count_minterms(self.reachable, self.curr_var_indices.len() as u32)
    }

    fn release_refs(&mut self) -> RefLeakReport {
        if self.released {
            return RefLeakReport {
                nonzero_ref_count: 0,
            };
        }

        self.mgr.deref_node(self.transitions.0);
        self.mgr.deref_node(self.transitions_01_add.0);
        self.mgr.deref_node(self.curr_var_cube.0);
        self.mgr.deref_node(self.next_var_cube.0);
        self.mgr.deref_node(self.init.0);
        self.mgr.deref_node(self.reachable.0);
        if let Some(identity) = self.curr_next_identity {
            self.mgr.deref_node(identity.0);
        }

        for nodes in self.var_curr_nodes.values() {
            for &node in nodes {
                self.mgr.deref_node(node);
            }
        }
        for nodes in self.var_next_nodes.values() {
            for &node in nodes {
                self.mgr.deref_node(node);
            }
        }

        self.released = true;
        RefLeakReport {
            nonzero_ref_count: self.mgr.nonzero_ref_count(),
        }
    }

    pub fn release_report(&mut self) -> RefLeakReport {
        self.release_refs()
    }

    /// Human-readable summary of transition relation statistics.
    pub fn describe(&mut self) -> Vec<String> {
        let mut desc = Vec::new();
        desc.push("Variables:\n".into());
        for (var_name, curr_nodes) in &self.var_curr_nodes {
            let next_nodes = &self.var_next_nodes[var_name];
            desc.push(format!(
                "  {}: curr nodes {:?}, next nodes {:?}\n",
                var_name, curr_nodes, next_nodes
            ));
        }

        desc.push(format!("Transitions ADD node ID: {:?}\n", self.transitions));
        desc.push(format!(
            "Transitions 0-1 ADD node ID: {:?}\n",
            self.transitions_01_add
        ));

        let (curr_bits, next_bits) = self.state_variable_counts();
        let stats = self.mgr.add_stats(self.transitions, curr_bits + next_bits);
        desc.push(format!(
            "Num Nodes ADD: {}, Num Terminals: {}, Transitions(minterms): {}\n",
            stats.node_count, stats.terminal_count, stats.minterms
        ));
        desc
    }

    fn build_identity_transition_bdd(&mut self) -> BddNode {
        let mut ident = self.mgr.bdd_one();
        for (&curr_idx, &next_idx) in self
            .curr_var_indices
            .iter()
            .zip(self.next_var_indices.iter())
        {
            let curr = self.mgr.bdd_var(curr_idx);
            let next = self.mgr.bdd_var(next_idx);
            let eq = self.mgr.bdd_equals(curr, next);
            ident = self.mgr.bdd_and(ident, eq);
        }
        ident
    }

    pub fn get_curr_next_identity_bdd(&mut self) -> BddNode {
        if let Some(identity) = self.curr_next_identity {
            self.mgr.ref_node(identity.0);
            return identity;
        }

        let identity = self.build_identity_transition_bdd();
        self.mgr.ref_node(identity.0);
        self.curr_next_identity = Some(identity);
        identity
    }
}

impl Drop for SymbolicDTMC {
    fn drop(&mut self) {
        let report = self.release_refs();
        if report.nonzero_ref_count > 0 {
            error!(
                "RefManager non-zero refs after owned release: {}",
                report.nonzero_ref_count
            );
        }
    }
}
