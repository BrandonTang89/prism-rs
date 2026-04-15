Okay, create a criterion based benchmarking suite.

We want 2 types of benchmarks:
- Construction benchmarks: Time taken to parse, analyze and construct the symbolic representation of a model (without a property file)
- Checking benchmarks: Time taken to parse, analyze, construct and check a property on a model

Create benchmarks for BRP and leaders models and the properties there that we can already benchmark. We can vary the parameters they take to get a range of sizes

Do set criterion to generate the HTML report from benchmarks.

Also make the format for info! messages slightly neater, we just need to print "INFO" and not the location of the info call or the timestamp. 

Don't worry about benchmarks in the CI for now.
