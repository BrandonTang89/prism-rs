build_init_bdd is called multiple times but really should just be called once and stored in the SymbolicDTMC. 

Furthermore, 

    /// Number of reachable states computed by BFS during construction.
    pub reachable_states: u64,

is just the number of minterms of the reachable BDD, so we can remove this from the SymbolicDTMC struct and just compute when needed

Apart from that, help me to write the docs in dtmc_details, where you should insert pseudocode and descriptions for the next and bounded until algorithms that are already implemented.

