use std::sync::OnceLock;

use oxidd::{mtbdd::MTBDDManagerRef, mtbdd::new_manager, mtbdd::terminal::F64};
use crate::ast::*;

static MANAGER: OnceLock<MTBDDManagerRef<F64>> = OnceLock::new();

fn get_manager() -> &'static MTBDDManagerRef<F64> {
    const INNER_NODE_CAP: usize = 1000;
    const TERMINAL_NODE_CAP: usize = 1000;
    const CACHE_CAP: usize = 1000;
    const NUM_THREADS: u32 = 4;
    MANAGER.get_or_init(|| new_manager(INNER_NODE_CAP, TERMINAL_NODE_CAP, CACHE_CAP, NUM_THREADS))
}

struct SymbolicDTMC {
    transitions: 
}

pub fn build_symbolic_dtmc(model: &DTMCModel) -> SymbolicDTMC {
    let manager = get_manager();



}
