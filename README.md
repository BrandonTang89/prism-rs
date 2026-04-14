# prism-rs

`prism-rs` is a Rust implementation of a subset of the
[PRISM](https://www.prismmodelchecker.org/) model checker.

## Current status

Working today:
- PRISM DTMC model parsing and symbolic construction
- symbolic probabilistic checking for:
  - `P=? [X phi]`
  - `P=? [phi1 U<=k phi2]`

Not implemented yet:
- unbounded until model checking (`P=? [phi1 U phi2]`)
- reward model checking (`R=? [...]`)
- broader PRISM language coverage (MDPs, TSGs, CSGs, etc)

## Differences from Prism
Apart from some differences in supported features (see other [docs](docs/)), there are some differences in the internal design.

Here, we explicitly differentiate between BDDs and ADDs in the codebase with the BddNode and AddNode types. These wrap CUDD BDD and ADD nodes respectively. In the Prism codebase, only JDDNode is used, which internally wrap CUDD ADD nodes. BDDs are then just represented as 0-1 ADDs. This lack of type strictness is more prone to errors and likely less efficient since ADDs in CUDD do not implement complementary edges and therefore make negation more expensive.

In PRISM, there is heavy use of inheritance, but in Rust we prefer to use composition instead. While we have not implemented other model types yet, we hope to avoid overly messy code that has many if-else branches spread throughout but would prefer mild duplication in the different model types. _This looks to be challenging to achieve in practice, but we will see._

## Build and test
We provide a Nix Development Shell environment with the relevant version of Rust included. To enter it, run:

```bash
nix develop
```

Run the full test suite:

```bash
cargo test -- --nocapture
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