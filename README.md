# Prism-rs

`prism-rs` is a Rust implementation of a subset of the
[PRISM](https://www.prismmodelchecker.org/) model checker focused on symbolic
DTMC analysis with CUDD-backed decision diagrams.

## Current status

Working today:
- PRISM DTMC model parsing (modules, constants, commands, renamed modules)
- PRISM property parsing for probability/reward query syntax
- constant folding and semantic analysis (including bounded-until bound checks)
- symbolic DTMC construction with ADD transition relation
- reachability analysis and dead-end self-loop completion
- symbolic probabilistic checking for:
  - `P=? [X phi]`
  - `P=? [phi1 U<=k phi2]`
- property filtering by index from the CLI (`--props 1,2,3`)

Not implemented yet:
- unbounded until model checking (`P=? [phi1 U phi2]`)
- reward model checking (`R=? [...]`)
- broader PRISM language coverage beyond current parser subset

## Build and test

Run the full test suite:

```bash
cargo test -- --nocapture
```

Run symbolic checking integration tests only:

```bash
cargo test --test dtmc_sym_check_tests -- --nocapture
```

## Using the binary

General form:

```bash
cargo run -- --model-type dtmc --model <path-to-model.prism> [options]
```

Options:
- `--model-type dtmc` model type selector (currently only DTMC)
- `--model <path>` model file
- `--const NAME=VALUE,...` constant overrides
- `--prop-file <path>` property file to load
- `--props 1,2,3` evaluate only selected property indices (1-based, in file order)
- `-v, --verbose` enable debug-level logging

### Example

```bash
cargo run -- --model-type dtmc --model tests/dtmc/knuth_die.prism --prop-file tests/dtmc/knuth_die.prop --props 2,3 --const x=4
```

This parses the model and properties, constructs the symbolic DTMC, and checks
properties 2 and 3 from the property file.

## Development notes

- CUDD calls are centralized in `src/ref_manager.rs`.
- Parser generation is handled via `build.rs` (LALRPOP).
- `SymbolicDTMC` owns DD roots and enforces release on drop.
