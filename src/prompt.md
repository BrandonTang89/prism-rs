I think this is on the right track, but it seems like its slightly messy to have 2 separate structs, one for properties and one for the model.

I think that analyze should actually fold the properties into the model, this way, we have a single list of constants that applies across the entire model + properties, making everything cleaner for when we do the symbolic model checking later on.

I did not really read the new code in a lot of detail, but do ensure that we don't duplicate code that does things like constant folding and type checking between that used for the model analysis and the property analysis. We should be able to reuse the same code for both, and just have a single list of constants that applies to both.

Also yes, do implement pretty printing of model properties to print to the console.