I'm in the middle of moving over to using pointer protection for all the BDDs and ADDs

obseerve and example function below:

f(a: AddNode, b: BddNode) -> AddNode {
    // we can assume that a and b are protected by the caller
    let mut guard = LocalRootsGuard::new();

    let mut c = add_zero(); // sadly all our temporaries need to be mutable. For hygiene, we declare all of the that are local to a function once and then protect them all.
    guard.protect(c); // this should be called on every new temporary variable that is created in the function.  (Ideally these 2 lines should come together as macro maybe)
    
    let mut d = bdd_zero();
    guard.protect(d); // (Ideally these 2 lines should come together as macro maybe)
    
    c = self.add_op(a, a); // this is automatically protected because it is stored in c
    c = self.add_op(c, a); // this is automatically protected because it is stored in c

    return c; // the caller will store this within their own local variable, which should be protected by the caller's guard
} // once guard goes out of scope, all the variables that it protects are unprotected

protected_slot should be used within structs that contain BDDs or ADDs, such as SymbolicDTMC. 





Docs for sylvan is https://trolando.github.io/sylvan/