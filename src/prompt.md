I think that the way the ref_manager.rs works now while it passes the tests,
is not very efficient or nice.

Since sylvan-sys exposes BDD and MTBDD references already, we can directly use those to be BddNode and AddNode respectively, without going through a common Node.

