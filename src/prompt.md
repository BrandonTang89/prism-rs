Lets work on implementing model checking for the unbounded until property

Given a property of the form `P=? [phi1 U phi2]`:

we compute the probabilities for all the different states symbolically as follows:

s_no = prob0(phi1, phi2); for now we leave this as just (NOT phi1 AND NOT phi2), but we will work on it later
s_yes = prob1(phi1, phi2); for now we leave this as just phi2, but we will work on it later

s_? = reachable AND NOT (s_no OR s_yes) where reachable is the set of states reachable from the initial state

T_? = T times bdd_to_add(s_?) where T is the probabilistic transition ADD 

A = identity_add() - T_?
b = bdd_to_add(s_yes)

return solve_jacobi(A, b, b)


identity_add() is implemented by build_identity_transition_bdd() in reachability.rs
but do upgrade the implementation there to directly use Cudd_bddIthVar together with the vector of cudd variable indices to build the identity BDD more efficiently. You need to expose Cudd_bddIthVar via the ref_manager.rs. To be honest, this probably should also be cached within SymbolicDTMC since we use it multiple times.


To implement solve_jacobi(A, b, init) as follows:

d = Abstract_max(A, next_vars, A * identity_add())
A' = A times const(-1) times not identity_add()
A' = A' / d
b' = b / d

sol = init
done = false
while !done:
    sol' = MVMult(A', rename_vars(sol, next_vars, curr_vars)) + b'
    
    if max_diff(sol, sol') < epsilon:
        done = true
    else:
        sol = sol'

return sol


max_diff can be implemented via Cudd_EqualSupNorm to the same EPS that is used in ref_manager.rs, which needs to be exposed.

Once you are done. Test that P=? [ F s=7 & d=x ] on the knuth die model gives 1/6 as expected for x in {1,2,3,4,5,6}. Put this as a new test in tests/dtmc_sym_check_tests.rs. 

Ensure all the existing tests pass as well. Let me know if there are any issues along the way.
