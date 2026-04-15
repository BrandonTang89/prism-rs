# prismulti

`prismulti` is a Rust implementation of a subset of the
[PRISM](https://www.prismmodelchecker.org/) model checker.

## Current status

Working today:
- PRISM DTMC model parsing and symbolic construction
- symbolic probabilistic checking for:
  - `P=? [X phi]`
  - `P=? [phi1 U<=k phi2]`
  - `P=? [phi1 U phi2]`

Not implemented yet:
- reward model checking (`R=? [...]`)
- broader PRISM language coverage (MDPs, TSGs, CSGs, etc)

## Differences from Prism
Apart from some differences in supported features (see other [docs](docs/)), there are some differences in the internal design.

Here, we explicitly differentiate between BDDs and ADDs in the codebase with the BddNode and AddNode types. These wrap CUDD BDD and ADD nodes respectively. In the Prism codebase, only JDDNode is used, which internally wrap CUDD ADD nodes. BDDs are then just represented as 0-1 ADDs. This lack of type strictness is more prone to errors and likely less efficient since ADDs in CUDD do not implement complementary edges and therefore make negation more expensive.

PRISM maintains its own version of CUDD within the source tree. Here, we use a patch file to make modifications to the CUDD source code, which allows us to easily pull in updates from the original CUDD repository while maintaining our changes (assuming that the files we patch do not change too much).

## Build and test
### Stable Rust
With the stable version of rust, you can build, run and test the project in the usual way with cargo:

```bash
cargo build --release
cargo run -- [options]
cargo test
```

The CUDD library is patched after it is downloaded and then built as part of the Rust build process. 

### Nix
We also support building with Nix for easier packaging and deployment in the future.

```bash
nix build
./result/bin/prismulti [options]
```

For Nix builds, we download the CUDD library from nixpkgs and then apply our patch to it.

For development, you can use `nix develop` to get a shell with all the relevant tools installed.

## Benchmarking 
 Benchmark controls (criterion):
 - `PRISM_BENCH_TARGET_SECS`: override Criterion measurement time in seconds.
 - `PRISM_BENCH_WARMUP_SECS`: override Criterion warmup time in seconds.

## Performance tuning (Sylvan)
 
 The symbolic backend can be tuned via environment variables:
 
 - `PRISM_SYLVAN_WORKERS` (default `0`): Lace worker threads. 
 - `PRISM_SYLVAN_GRANULARITY`: override Sylvan task granularity (optional).
 - `PRISM_SYLVAN_MEMORY_CAP`: memory cap passed to `sylvan_set_limits` (bytes).
 - `PRISM_SYLVAN_TABLE_RATIO`: table/cache ratio passed to `sylvan_set_limits`.
 - `PRISM_SYLVAN_INITIAL_RATIO`: initial table ratio passed to `sylvan_set_limits`.
 - `PRISM_TRACK_REFS`: enable/disable Rust-side ref-tracking bookkeeping (`true`/`false`).
  
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

For deeper DTMC semantics and symbolic checking notes, see
`docs/dtmc_details.md`.
