Let's implement the model renaming feature to allow us to easily reused

Look at herman3.prism, particularly at 

// add further processes through renaming
module process2 = process1 [ x1=x2, x3=x1 ] endmodule
module process3 = process1 [ x1=x3, x3=x2 ] endmodule

This is the syntax we want to support. We will initially fill in
DTMCAst::renamed_modules with the renamed modules, then, within
analyze.rs, we will expand the rename modules into actual modules
by copying the original module and applying the renaming substitutions
onto the local variables and update statements.

This probably involves cloning the base module, then traversing over the
clone and updating the relevant fields. For deal with subsituting the
expressions, we probably want a generic function that takes some
renaming map and applies it to an expression.

Once this is done, ensure everything can compile and that the herman3.prism
test in tests/parser_consts_tests.rs passes.