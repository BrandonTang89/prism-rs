## Use of (MT)BDD Restrict
In (symbolic) model checking, we often use (MT)BDDs to represent vectors and matrices where the indices correspond to states in the model. However, it is typical for the encoding of our binary variables to be sparse, meaning that there are assignments of variables that don't correspond to any relevant (i.e. valid and reachable) state. 

For example, if we have a variable `x` that can take values from 0 to 4, we need at least 3 bits to encode it, but the combinations of those 3 bits that correspond to values 5-7 are invalid. 

The typical way to deal with such things is to get to set the (MT)BDDs values for the irrelevant assignments to zero. This reduces the complexity in reasoning about the (MT)BDD and prevent bugs.

However, in order to achieve maximum performance, we would like to relax this invariant within hot loops. This can be done with `add_restrict(f, c)` which minimises the size of `f` while preserving the values of `f` on the satisfying assignments of `c`. This reduction in size increases the efficiency of subsequent operations on the (MT)BDD, but it also means that the (MT)BDD may have non-zero values on irrelevant assignments.

Most hot loops in the codebase are of the form:

```text
res = add_zero()
loop {
    res' = ReplaceVars(S |-> S', res)
    res = MatrixMultiply(A, res', S');

    // check some termination condition on res
}
```

Where 
- $\text{res} :S \rightarrow \mathbb{R}$
- $\text{A} : S \times S' \rightarrow \mathbb{R}$

Typically $S$ is the current state space and $S'$ is the next state space.

Suppose that $S_C$ and $S'_C$ are the relevant subsets of $S$ and $S'$ respectively. Then we consider restricting $A$ to $S_C$ and $S_C'$ independently or both together.

- Assuming that `res'` is zero outside of $S_C'$, then restricting $A$ to $S_C'$ can be done without any need for later fix-up since the additional non-zero values will be zeroed out in the matrix multiplication. This is a safe and effective optimization that can be done in all cases.
- Assuming that `res'` is zero outside of $S_C'$, then restricting $A$ to $S_C$ can be done but introduces non-zero values outside of $S_C$ in `res` that will need to be fixed up later. This can be done by multiplying `res` by `S_C`
- Both of these can be done together as well, which will require the same fix-up as the second case.

This transforms the loop into:

```text
A_restricted = restrict(restrict(A, S_C), S'_C)
res = add_zero()
loop {
    res' = ReplaceVars(S |-> S', res)
    res = MatrixMultiply(A_restricted, res', S');
    res = add_times(S_C, res)
}
```


Since we only need to do the restriction once at the start of the loop and can reuse the restricted matrix `A_restricted`, it is often the case that the reduction in size from the restriction will more than make up for the cost of the restriction and the fix-up, especially if the loop iterates many times.