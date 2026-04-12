# AGENTS

## Fast Start
- Run all checks with `cargo test -- --nocapture`.
- Run one integration test file with `cargo test --test dtmc_sym_constr_tests -- --nocapture`.
- Run one test case with `cargo test --test dtmc_sym_constr_tests dtmc_simple_constr -- --nocapture`.
- Run the CLI on a model with constants: `cargo run -- --model tests/dtmc/brp.prism --model-type dtmc --const N=1,MAX=2`.

## Build / Tooling Facts
- Parser generation is automatic via `build.rs` (`lalrpop::process_root()`), so grammar edits in `src/parser/parser.lalrpop` are picked up by normal Cargo builds.
- Release profile keeps debug symbols (`[profile.release] debug = true`, `strip = "none"`) for profiling.

## Architecture (what matters when editing)
- Main flow: `src/main.rs` -> `parser::parse_dtmc` -> `analyze::analyze_dtmc` -> `constr_symbolic::build_symbolic_dtmc` -> `reachability::compute_reachable_and_filter`.
- Symbolic manager wrapper is `src/ref_manager.rs`; this is the single place that should call CUDD APIs directly.
- `SymbolicDTMC` owns manager roots and is responsible for deref on drop (`src/symbolic_dtmc.rs`).

## CUDD Type Discipline (critical)
- `BddNode` wraps nodes used with `Cudd_bdd*` operations.
- `AddNode` wraps nodes used with `Cudd_add*` operations.
- Convert explicitly when crossing APIs:
  - ADD -> BDD: `add_to_bdd` / `add_to_bdd_pattern`
  - BDD -> ADD: `bdd_to_add`
- `Cudd_addIte` expects an ADD condition; in this repo `add_ite` accepts `BddNode` and converts internally to ADD before calling CUDD.

## Ref / Leak Checks
- Leak check path is CUDD-based (`Cudd_CheckZeroRef`) through `RefManager::nonzero_ref_count()`.
- `RefManager::debug_check()` wraps `Cudd_DebugCheck`; drop-time debug check is gated by `ENABLE_CUDD_DEBUGCHECK_ON_DROP`.

## Test Expectations You Should Not Accidentally Break
- `tests/dtmc_sym_constr_tests.rs` asserts transition node count, terminal count, minterms, reachable states, and zero nonzero refs via `release_report()`.
- `tests/parser_consts_tests.rs` asserts const parsing supports interspersed `const` declarations and optional initializers.
