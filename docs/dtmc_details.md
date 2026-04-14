## Discrete Time Markov Chains (DTMCs)
### Model Structure and Semantics
DTMCs are completely probabilistic, i.e. they have no nondeterminism/choices. Variables are organised into modules. Each module has some local variables and a set of commands. Each command has an action label, a guard (a boolean expression over the module's local variables), and a probabilistic update (a probability distribution over assignments to the module's local variables). 

We say that a command is "enabled" in a state $s$ if its guard is satisfied by $s$.

For each action label $a$, we let the set of modules that have at least one command with action label $a$ be $M_a$. The rest of the modules are then $M_{\neg a}$. At any state, action $a$ is "enabled" if and only if for every module $m \in M_a$, there is at least one "enabled" command in $m$ with action label $a$. It is not allowed for a module to have more than one "enabled" command with the same action label in any (reachable) state. 

For a given state $s$, if $a$ is runs, then
- For each module $m \in M_a$, $m$ is updated based on the unique enabled command in $m$. For local variables that are not mentioned by the command's update, they remain unchanged.
- For each module $m in M_{\neg a}$, $m$ remains unchanged.

At each time-step/state, there can be multiple "enabled" actions. We assume that each of the "enabled" actions is chosen with equal probability, i.e. the next action is chosen *uniformly*.

As syntactic sugar, we allow commands without an action label. Such commands are assumed to have an implicit action label that is unique to the module and shared by all commands in the same module without an explicit action label. 

Compared to normal prism, we do not support global variables and require that the model have a unique initial state. For variables without an `init` value, the lowest value/false is the default initial value.

Lastly, if there are any states with no enabled actions, we call them "dead-end" states and add a self-loop with probability 1 to them. This is to ensure that the transition relation is total, i.e. the sum of probabilities of outgoing transitions from any state is 1.

### Property Semantics
We support the following numerical properties for DTMCs:
- `P=? [X phi]`: The probability that the next state satisfies `phi`. 
- `P=? [phi1 U<=k phi2]`: the probability that `phi2` is satisfied within the next `k` steps, and that `phi1` is satisfied at all preceding steps.
- `P=? [phi1 U phi2]`: the probability that `phi2` is eventually satisfied, and that `phi1` is satisfied at all preceding steps until then.

Apart from lines of properties, we also allow declaration of constants in the property file.

### Parsing and Semantic Analysis Implementation Notes
DTMCs are parsed by [parse_dtmc](../src/parser.rs) and represented by [DTMCAst](../src/ast.rs). We then pass them through semantic analysis to do type checking, constant folding, and desugaring of commands without action labels. This modifies the AST and produces [DTMCModelInfo](../src/analyze.rs) which is later used for model checking.

Specifically regarding desugarinig if module `M` has commands without action labels, we will desugar them into commands with the same unique action label `__M_action__`.

Properties are parsed by [parse_dtmc_props](../src/parser.rs) and represented by [ParsedProps](../src/parser.rs). We merge this together with the model AST within DTMCAst, also merging the declared constants, before passing it to semantic analysis. This allows us to do constant folding and resolution that incorporates constants from both the model and property files.

### Symbolic Representation
We represent the transition relation of a DTMC an algebraic decision diagram (ADD) over current state variables $S$ and next state variables $S'$. All the information for this is stored in [SymbolicDTMC](../src/symbolic_dtmc.rs).

We construct it compositionally from the commands and actions in the model. After which, we perform reachability analysis to filter out unreachable states and add self-loops to dead-end states (see [../src/reachability.rs](../src/reachability.rs) for details).

### Symbolic Model Checking

#### Next
To compute 
