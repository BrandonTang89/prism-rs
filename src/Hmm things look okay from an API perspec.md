Okay, I see the issue. Firstly, we are still mixing CUDD 0-1 ADDs and BDDs.
Note that BDDs in CUDD are represented with complement arcs rather than ADDs
which don't use complement arcs.

Now, some of the functions require BDDs, and some require ADDs. 

Everything that is of the form cudd_bdd* requires bdds, everything that is of the form
cudd_add* requires adds. Most of the code looks fine, but for example, ate_ite uses
cudd_addite which actually requires the condition to be an ADD, so we should convert
the condition from a bdd to an add before calling it, i.e via self.add_to_bdd

I previously mentioned that I want everything to be ADDs. But this has changed. Now BddNode will wrap a NodeId that is a BDD, and AddNode will wrap a NodeId that is an ADD. We will have helper functions to convert between the two when necessary.

Rewrite the code to fix any issues with this that you find, and update the docs comments. We shouldn't have anymore Add01 kind of functions, use BDDs whenever they are required, and convert to ADDs when necessary.

Furthermore, 

    #[inline]
    pub fn regular(self) -> Self {
        Self(unsafe { Cudd_Regular(self.0) })
    }

    #[inline]
    pub fn is_complemented(self) -> bool {
        unsafe { Cudd_IsComplement(self.0) != 0 }

These only make sense for BDDs, so we should move them to BddNode.

The docs for CUDD are given in: https://web.mit.edu/sage/export/tmp/y/usr/share/doc/polybori/cudd/cuddAllDet.html

Read this to know which functions expect BDDs and which expect ADDs.