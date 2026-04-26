I'm starting to parse and analyze MDP files, examples of which can be fond in tests/mdp/.

Fortunately the structure of MDP files is very similar to DTMC files. I have done some refactoring to ensure that the parsing and analysis logic is mostly shared between the two, with the use of "BasicAst" and "BasicModelInfo" (since when we consider more complex models like stochastic games, we will have additional fields to the AST and ModelInfo).

Help me to modify the main file, parser.rs, parser.lalrpop, and analyze.rs (and any other necessary files) to add support for MDPs similarly to DTMCs. Test the implementation by checking that after the changes, we can call the binary to parse and analyze the 2 simple test cases in tests/mdp.

Don't just bolt on additional things for MDPs but try to refactor the code to share as much logic as possible between DTMCs and MDPs, since they are quite similar. 