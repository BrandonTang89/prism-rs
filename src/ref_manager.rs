use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::{self, Write},
};

use lumindd::{Manager, NodeId};
pub struct RefManager {
    pub inner: Manager,
}

#[derive(Debug, Clone, Copy)]
pub struct AddStats {
    pub node_count: usize,
    pub terminal_count: usize,
    pub minterms: u64,
}

static EPS: f64 = 1e-10;

impl RefManager {
    pub fn new() -> Self {
        Self {
            inner: Manager::new(),
        }
    }

    /// __Refs__: Result \
    pub fn add_const(&mut self, value: f64) -> NodeId {
        let node = self.inner.add_const(value);
        self.inner.ref_node(node);
        node
    }

    /// __Refs__: Result \
    /// __Derefs__: a
    pub fn bdd_not(&mut self, a: NodeId) -> NodeId {
        let result = self.inner.bdd_not(a);
        self.inner.ref_node(result);
        self.inner.deref_node(a);
        result
    }

    /// __Refs__: Result \
    /// __Derefs__: a, b
    pub fn bdd_equals(&mut self, a: NodeId, b: NodeId) -> NodeId {
        let result = self.inner.bdd_xnor(a, b);
        self.inner.ref_node(result);
        self.inner.deref_node(a);
        self.inner.deref_node(b);
        result
    }

    /// __Refs__: Result \
    /// __Derefs__: a, b
    pub fn bdd_nequals(&mut self, a: NodeId, b: NodeId) -> NodeId {
        let result = self.inner.bdd_xor(a, b);
        self.inner.ref_node(result);
        self.inner.deref_node(a);
        self.inner.deref_node(b);
        result
    }

    /// __Refs__: Result \
    /// __Derefs__: a, b
    pub fn bdd_and(&mut self, a: NodeId, b: NodeId) -> NodeId {
        let result = self.inner.bdd_and(a, b);
        self.inner.ref_node(result);
        self.inner.deref_node(a);
        self.inner.deref_node(b);
        result
    }

    /// __Refs__: Result \
    /// __Derefs__: a, b
    pub fn bdd_or(&mut self, a: NodeId, b: NodeId) -> NodeId {
        let result = self.inner.bdd_or(a, b);
        self.inner.ref_node(result);
        self.inner.deref_node(a);
        self.inner.deref_node(b);
        result
    }

    /// __Refs__: Result \
    /// __Derefs__: f, g
    pub fn bdd_and_abstract(&mut self, f: NodeId, g: NodeId, cube: NodeId) -> NodeId {
        let result = self.inner.bdd_and_abstract(f, g, cube);
        self.inner.ref_node(result);
        self.inner.deref_node(f);
        self.inner.deref_node(g);
        result
    }

    /// __Refs__: Result \
    /// __Derefs__: f
    pub fn bdd_swap_variables(&mut self, f: NodeId, x: &[u16], y: &[u16]) -> NodeId {
        let result = self.inner.bdd_swap_variables(f, x, y);
        self.inner.ref_node(result);
        self.inner.deref_node(f);
        result
    }

    /// __Refs__: Result \
    /// __Derefs__: a, b
    pub fn add_times(&mut self, a: NodeId, b: NodeId) -> NodeId {
        let result = self.inner.add_times(a, b);
        self.inner.ref_node(result);
        self.inner.deref_node(a);
        self.inner.deref_node(b);
        result
    }

    /// __Refs__: Result \
    /// __Derefs__: a, b
    pub fn add_divide(&mut self, a: NodeId, b: NodeId) -> NodeId {
        let result = self.inner.add_divide(a, b);
        self.inner.ref_node(result);
        self.inner.deref_node(a);
        self.inner.deref_node(b);
        result
    }

    /// __Refs__: Result \
    /// __Derefs__: a, b
    pub fn add_plus(&mut self, a: NodeId, b: NodeId) -> NodeId {
        let result = self.inner.add_plus(a, b);
        self.inner.ref_node(result);
        self.inner.deref_node(a);
        self.inner.deref_node(b);
        result
    }

    /// __Refs__: Result \
    /// __Derefs__: a, b
    pub fn add_minus(&mut self, a: NodeId, b: NodeId) -> NodeId {
        let result = self.inner.add_minus(a, b);
        self.inner.ref_node(result);
        self.inner.deref_node(a);
        self.inner.deref_node(b);
        result
    }

    /// __Refs__: Result \
    /// __Derefs__: a
    pub fn add_bdd_pattern(&mut self, a: NodeId) -> NodeId {
        let result = self.inner.add_bdd_pattern(a);
        self.inner.ref_node(result);
        self.inner.deref_node(a);
        result
    }

    /// __Refs__: Result \
    /// __Derefs__: a
    fn add_bdd_threshold(&mut self, a: NodeId, threshold: f64) -> NodeId {
        let result = self.inner.add_bdd_threshold(a, threshold);
        self.inner.ref_node(result);
        self.inner.deref_node(a);
        result
    }

    /// __Refs__: Result \
    /// __Derefs__: a
    pub fn add_to_bdd(&mut self, a: NodeId) -> NodeId {
        let result = self.add_bdd_threshold(a, EPS);
        self.inner.ref_node(result);
        self.inner.deref_node(a);
        result
    }

    /// __Refs__: Result \
    /// __Derefs__: a
    pub fn bdd_to_add(&mut self, a: NodeId) -> NodeId {
        let result = self.inner.bdd_to_add(a);
        self.inner.ref_node(result);
        self.inner.deref_node(a);
        result
    }

    /// Convert two ADDs to a BDD that is 1 iff they are equal.
    /// __Refs__: Result \
    /// __Derefs__: a, b
    pub fn add_equals(&mut self, a: NodeId, b: NodeId) -> NodeId {
        self.ref_node(a);
        self.ref_node(b);
        let gt = self.add_greater_than(a, b);
        let lt = self.add_less_than(a, b);
        let neq = self.bdd_or(gt, lt);
        self.bdd_not(neq)
    }

    /// Convert two ADDs to a BDD that is 1 iff they are not equal.
    /// __Refs__: Result \
    /// __Derefs__: a, b
    pub fn add_nequals(&mut self, a: NodeId, b: NodeId) -> NodeId {
        self.ref_node(a);
        self.ref_node(b);
        let gt = self.add_greater_than(a, b);
        let lt = self.add_less_than(a, b);
        self.bdd_or(gt, lt)
    }

    /// If cond > 0 then then_branch else else_branch \
    /// __Refs__: Result \
    /// __Derefs__: cond, then_branch, else_branch
    pub fn add_ite(&mut self, cond: NodeId, then_branch: NodeId, else_branch: NodeId) -> NodeId {
        let result = self.inner.add_ite(cond, then_branch, else_branch);
        self.inner.ref_node(result);
        self.inner.deref_node(cond);
        self.inner.deref_node(then_branch);
        self.inner.deref_node(else_branch);
        result
    }

    /// Convert two ADDs to a BDD that is 1 iff a > b.
    /// __Refs__: Result \
    /// __Derefs__: a, b
    pub fn add_greater_than(&mut self, a: NodeId, b: NodeId) -> NodeId {
        let diff = self.add_minus(a, b);
        self.add_bdd_threshold(diff, EPS)
    }

    /// Convert two ADDs to a BDD that is 1 iff a < b. \
    /// __Refs__: Result \
    /// __Derefs__: a, b
    pub fn add_less_than(&mut self, a: NodeId, b: NodeId) -> NodeId {
        let diff = self.add_minus(b, a);
        self.add_bdd_threshold(diff, EPS)
    }

    /// Convert two ADDs to a BDD that is 1 iff a >= b. \
    /// __Refs__: Result \
    /// __Derefs__: a, b
    pub fn add_greater_or_equal(&mut self, a: NodeId, b: NodeId) -> NodeId {
        let lt_bdd = self.add_less_than(a, b);
        self.bdd_not(lt_bdd)
    }

    /// Convert two ADDs to a BDD that is 1 iff a <= b. \
    /// __Refs__: Result \
    /// __Derefs__: a, b
    pub fn add_less_or_equal(&mut self, a: NodeId, b: NodeId) -> NodeId {
        let gt_bdd = self.add_greater_than(a, b);
        self.bdd_not(gt_bdd)
    }

    /// __Refs__: Result \
    /// __Derefs__: F
    pub fn add_exist_abstract(&mut self, f: NodeId, cube: NodeId) -> NodeId {
        let result = self.inner.add_exist_abstract(f, cube);
        self.inner.ref_node(result);
        self.inner.deref_node(f);
        result
    }

    /// __Refs__: Result
    pub fn new_var(&mut self) -> NodeId {
        let node = self.inner.bdd_new_var();
        self.inner.ref_node(node);
        node
    }

    /// __Refs__: Result (0/1-valued ADD variable)
    pub fn add_var(&mut self, var_index: u16) -> NodeId {
        let node = self.inner.add_ith_var(var_index);
        self.inner.ref_node(node);
        node
    }

    /// __Refs__: Node \
    /// __Derefs__: None
    pub fn ref_node(&mut self, node: NodeId) -> NodeId {
        self.inner.ref_node(node);
        node
    }

    /// __Refs__: None \
    /// __Derefs__: Node
    pub fn deref_node(&mut self, node: NodeId) -> NodeId {
        self.inner.deref_node(node);
        node
    }

    /// __Refs__: BDD ONE \
    pub fn bdd_one(&mut self) -> NodeId {
        self.inner.ref_node(NodeId::ONE);
        NodeId::ONE
    }

    /// __Refs__: BDD ZERO \
    pub fn bdd_zero(&mut self) -> NodeId {
        self.inner.ref_node(NodeId::ZERO);
        NodeId::ZERO
    }

    /// __Refs__: ZERO \
    pub fn add_zero(&mut self) -> NodeId {
        let node = self.inner.add_zero();
        self.inner.ref_node(node);
        node
    }

    /// Unif(m) = m ÷ Abstract(+,next_var_cube,m). \
    /// __Refs__: result \
    /// __Derefs__: m
    pub fn unif(&mut self, m: NodeId, next_var_cube: NodeId) -> NodeId {
        // Keep one reference to `m` as numerator while abstraction consumes one.
        self.ref_node(m);
        let denom = self.add_sum_abstract(m, next_var_cube);

        // Avoid division by zero on out-of-domain encodings:
        // safe_denom = denom + ITE(denom > EPS, 0, 1).
        self.ref_node(denom);
        let denom_pos_bdd = self.add_bdd_threshold(denom, EPS);
        let denom_pos_add = self.bdd_to_add(denom_pos_bdd);
        let one = self.add_const(1.0);
        let denom_is_zero_add = self.add_minus(one, denom_pos_add);
        let safe_denom = self.add_plus(denom, denom_is_zero_add);

        self.add_divide(m, safe_denom)
    }

    /// Generic ADD abstraction over all vars in `cube`.
    ///
    /// For each quantified variable x, combines positive/negative cofactors
    /// with `combine` in variable-order sequence.
    ///
    /// `combine` must consume both operands and return a referenced result.
    /// __Refs__: result
    /// __Derefs__: f
    pub fn add_abstract_with<F>(&mut self, f: NodeId, cube: NodeId, mut combine: F) -> NodeId
    where
        F: FnMut(&mut RefManager, NodeId, NodeId) -> NodeId,
    {
        let mut vars: Vec<(u16, u32)> = self
            .inner
            .bdd_support(cube)
            .into_iter()
            .map(|var| (var, self.inner.read_perm(var)))
            .collect();
        vars.sort_by_key(|&(_, level)| level);

        let mut cache: HashMap<(NodeId, usize), NodeId> = HashMap::new();
        let res = self.add_abstract_with_rec(f.regular(), &vars, 0, &mut combine, &mut cache);

        // Drop cache-owned references; keep only the returned root reference.
        for &cached in cache.values() {
            self.deref_node(cached);
        }

        self.deref_node(f);
        res
    }

    fn add_abstract_with_rec<F>(
        &mut self,
        node: NodeId,
        vars: &[(u16, u32)],
        idx: usize,
        combine: &mut F,
        cache: &mut HashMap<(NodeId, usize), NodeId>,
    ) -> NodeId
    where
        F: FnMut(&mut RefManager, NodeId, NodeId) -> NodeId,
    {
        let node = node.regular();
        let key = (node, idx);
        if let Some(&cached) = cache.get(&key) {
            self.ref_node(cached);
            return cached;
        }

        let result = if idx >= vars.len() {
            self.ref_node(node)
        } else {
            let node_var = self.inner.read_var_index(node);

            if node_var == u16::MAX {
                // Terminal: each remaining quantified variable is absent here,
                // so repeatedly combine the function with itself.
                let mut acc = self.ref_node(node);
                for _ in idx..vars.len() {
                    self.ref_node(acc);
                    self.ref_node(acc);
                    let next = combine(self, acc, acc);
                    self.deref_node(acc);
                    acc = next;
                }
                acc
            } else {
                let node_level = self.inner.read_perm(node_var);
                let (_, quant_level) = vars[idx];

                if node_level > quant_level {
                    // Quantified variable is above this node but absent in subtree.
                    self.ref_node(node);
                    self.ref_node(node);
                    let merged = combine(self, node, node);
                    let out = self.add_abstract_with_rec(merged, vars, idx + 1, combine, cache);
                    self.deref_node(merged);
                    out
                } else if node_level == quant_level {
                    // Quantify this variable by combining the two cofactors.
                    let t = self.inner.read_then(node).regular();
                    let e = self.inner.read_else(node).regular();
                    self.ref_node(t);
                    self.ref_node(e);
                    let merged = combine(self, t, e);
                    let out = self.add_abstract_with_rec(merged, vars, idx + 1, combine, cache);
                    self.deref_node(merged);
                    out
                } else {
                    // This node's variable is not quantified yet; recurse once.
                    let t = self.inner.read_then(node).regular();
                    let e = self.inner.read_else(node).regular();
                    let t_abs = self.add_abstract_with_rec(t, vars, idx, combine, cache);
                    let e_abs = self.add_abstract_with_rec(e, vars, idx, combine, cache);
                    let cond = self.add_var(node_var);
                    self.add_ite(cond, t_abs, e_abs)
                }
            }
        };

        // Keep one cache-owned reference plus one for the caller.
        self.ref_node(result);
        cache.insert(key, result);
        result
    }

    /// ADD sum-abstraction over all vars in `cube`.
    /// For each quantified variable x, computes f[x=1] + f[x=0].
    /// __Refs__: result
    /// __Derefs__: f
    pub fn add_sum_abstract(&mut self, f: NodeId, cube: NodeId) -> NodeId {
        self.add_abstract_with(f, cube, |mgr, a, b| mgr.add_plus(a, b))
    }

    fn var_index_label_map(&self, var_names: &HashMap<NodeId, String>) -> HashMap<u16, String> {
        let mut labels = HashMap::new();
        for (&node, name) in var_names {
            let var_index = self.inner.read_var_index(node.regular());
            if var_index != u16::MAX {
                labels.entry(var_index).or_insert_with(|| name.clone());
            }
        }
        labels
    }

    fn var_label(var_index: u16, labels: &HashMap<u16, String>) -> String {
        labels
            .get(&var_index)
            .cloned()
            .unwrap_or_else(|| format!("x{}", var_index))
    }

    pub fn num_nodes(&self, node: NodeId) -> usize {
        let mut visited = HashSet::new();
        self.num_nodes_rec(node.regular(), &mut visited);
        visited.len()
    }

    fn num_nodes_rec(&self, node: NodeId, visited: &mut HashSet<NodeId>) {
        let node = node.regular();
        if !visited.insert(node) {
            return;
        }
        if self.inner.read_var_index(node) == u16::MAX {
            return;
        }
        let t = self.inner.read_then(node);
        let e = self.inner.read_else(node);
        self.num_nodes_rec(t, visited);
        self.num_nodes_rec(e, visited);
    }

    pub fn bdd_count_minterms(&mut self, bdd: NodeId, num_vars: u32) -> u64 {
        self.inner.bdd_count_minterm(bdd, num_vars).round() as u64
    }

    pub fn add_stats(&mut self, root: NodeId, num_vars: u32) -> AddStats {
        let root = root.regular();

        let mut visited = HashSet::new();
        let mut terminal_count = 0usize;
        self.count_nodes_and_terminals(root, &mut visited, &mut terminal_count);

        let bdd = self.add_to_bdd(root);
        let minterms = self.bdd_count_minterms(bdd, num_vars);

        AddStats {
            node_count: visited.len(),
            terminal_count,
            minterms,
        }
    }

    fn count_nodes_and_terminals(
        &self,
        node: NodeId,
        visited: &mut HashSet<NodeId>,
        terminal_count: &mut usize,
    ) {
        let node = node.regular();
        if !visited.insert(node) {
            return;
        }

        if self.inner.read_var_index(node) == u16::MAX {
            *terminal_count += 1;
            return;
        }

        let t = self.inner.read_then(node);
        let e = self.inner.read_else(node);
        self.count_nodes_and_terminals(t, visited, terminal_count);
        self.count_nodes_and_terminals(e, visited, terminal_count);
    }

    /// Dumps a graphviz dot file representing the structure of the ADD rooted at `root`. \
    /// Positive edges (then) are solid, negative edges (else) are dashed. \
    /// __Refs__: None \
    /// __Derefs__: None
    pub fn dump_add_dot(
        &self,
        root: NodeId,
        path: &str,
        var_names: &HashMap<NodeId, String>,
    ) -> io::Result<()> {
        let mut out = File::create(path)?;
        writeln!(out, "digraph ADD {{")?;
        writeln!(out, "  rankdir=TB;")?;
        let mut ids: HashMap<NodeId, usize> = HashMap::new();
        let mut next_id = 0usize;
        let mut visited: HashSet<NodeId> = HashSet::new();
        let labels = self.var_index_label_map(var_names);
        self.dump_add_dot_rec(
            root.regular(),
            &mut out,
            &labels,
            &mut ids,
            &mut next_id,
            &mut visited,
        )?;
        writeln!(out, "}}")?;
        Ok(())
    }
    fn intern_id(ids: &mut HashMap<NodeId, usize>, next_id: &mut usize, n: NodeId) -> usize {
        *ids.entry(n.regular()).or_insert_with(|| {
            let id = *next_id;
            *next_id += 1;
            id
        })
    }

    /// Recurses through the DD structure to output DOT \
    /// Uses ids and next_id to assign a unique integer to each node created to use as the
    /// node identifier in the dot file \
    /// Uses labels to label the nodes in the dot file \
    /// We don't use the labels as identifiers since the labels can contain spaces
    fn dump_add_dot_rec<W: Write>(
        &self,
        n: NodeId,
        out: &mut W,
        labels: &HashMap<u16, String>,
        ids: &mut HashMap<NodeId, usize>,
        next_id: &mut usize,
        visited: &mut HashSet<NodeId>,
    ) -> io::Result<()> {
        let n = n.regular();
        if !visited.insert(n) {
            return Ok(());
        }
        let this = Self::intern_id(ids, next_id, n);
        let var = self.inner.read_var_index(n);
        if var == u16::MAX {
            let v = self.inner.add_value(n).unwrap_or(f64::NAN);
            writeln!(out, "  n{} [shape=box,label=\"{}\"];", this, v)?;
            return Ok(());
        }
        let t = self.inner.read_then(n).regular();
        let e = self.inner.read_else(n).regular();
        let tid = Self::intern_id(ids, next_id, t);
        let eid = Self::intern_id(ids, next_id, e);
        let label = Self::var_label(var, labels);
        writeln!(out, "  n{} [shape=ellipse,label=\"{}\"];", this, label)?;
        writeln!(out, "  n{} -> n{};", this, tid)?;
        writeln!(out, "  n{} -> n{} [style=dashed];", this, eid)?;
        self.dump_add_dot_rec(t, out, labels, ids, next_id, visited)?;
        self.dump_add_dot_rec(e, out, labels, ids, next_id, visited)?;
        Ok(())
    }

    pub fn dump_bdd_dot(
        &self,
        root: NodeId,
        path: &str,
        var_names: &HashMap<NodeId, String>,
    ) -> io::Result<()> {
        let mut out = File::create(path)?;
        writeln!(out, "digraph BDD {{")?;
        writeln!(out, "  rankdir=TB;")?;
        writeln!(out, "  ONE [shape=box,label=\"1\"];")?;
        writeln!(out, "  ZERO [shape=box,label=\"0\"];")?;

        if root.is_one() {
            writeln!(out, "  root [shape=point];")?;
            writeln!(out, "  root -> ONE;")?;
            writeln!(out, "}}")?;
            return Ok(());
        }

        if root.is_zero() {
            writeln!(out, "  root [shape=point];")?;
            writeln!(out, "  root -> ZERO;")?;
            writeln!(out, "}}")?;
            return Ok(());
        }

        let labels = self.var_index_label_map(var_names);
        let mut ids: HashMap<NodeId, usize> = HashMap::new();
        let mut next_id = 0usize;
        let mut visited: HashSet<NodeId> = HashSet::new();

        let root_reg = root.regular();
        let root_id = Self::intern_id(&mut ids, &mut next_id, root_reg);
        writeln!(out, "  root [shape=point];")?;
        if root.is_complemented() {
            writeln!(out, "  root -> n{} [color=red];", root_id)?;
        } else {
            writeln!(out, "  root -> n{};", root_id)?;
        }

        self.dump_bdd_dot_rec(
            root_reg,
            &mut out,
            &labels,
            &mut ids,
            &mut next_id,
            &mut visited,
        )?;
        writeln!(out, "}}")?;
        Ok(())
    }

    fn dump_bdd_dot_rec<W: Write>(
        &self,
        n: NodeId,
        out: &mut W,
        labels: &HashMap<u16, String>,
        ids: &mut HashMap<NodeId, usize>,
        next_id: &mut usize,
        visited: &mut HashSet<NodeId>,
    ) -> io::Result<()> {
        let n = n.regular();
        if n.is_constant() || !visited.insert(n) {
            return Ok(());
        }

        let this = Self::intern_id(ids, next_id, n);
        let var = self.inner.read_var_index(n);
        let label = Self::var_label(var, labels);
        writeln!(out, "  n{} [shape=ellipse,label=\"{}\"] ;", this, label)?;

        let t = self.inner.read_then(n);
        let e = self.inner.read_else(n);

        let t_target = if t.is_one() {
            "ONE".to_string()
        } else if t.is_zero() {
            "ZERO".to_string()
        } else {
            let tid = Self::intern_id(ids, next_id, t.regular());
            format!("n{}", tid)
        };
        let e_target = if e.is_one() {
            "ONE".to_string()
        } else if e.is_zero() {
            "ZERO".to_string()
        } else {
            let eid = Self::intern_id(ids, next_id, e.regular());
            format!("n{}", eid)
        };

        if t.is_complemented() {
            writeln!(out, "  n{} -> {} [label=\"1\",color=red];", this, t_target)?;
        } else {
            writeln!(out, "  n{} -> {} [label=\"1\"] ;", this, t_target)?;
        }

        if e.is_complemented() {
            writeln!(
                out,
                "  n{} -> {} [label=\"0\",style=dashed,color=red];",
                this, e_target
            )?;
        } else {
            writeln!(
                out,
                "  n{} -> {} [label=\"0\",style=dashed,color=blue];",
                this, e_target
            )?;
        }

        if !t.is_constant() {
            self.dump_bdd_dot_rec(t.regular(), out, labels, ids, next_id, visited)?;
        }
        if !e.is_constant() {
            self.dump_bdd_dot_rec(e.regular(), out, labels, ids, next_id, visited)?;
        }
        Ok(())
    }

    /// Given a vector of variables (x0, x_1, ...),
    /// return the ADD that maps (x0, x1, ...) value
    /// assuming x0 is the LSB, x1 is the next bit, etc.\
    /// __Refs__: result \
    /// __Derefs__: None
    pub fn get_encoding(&mut self, nodes: &Vec<NodeId>) -> NodeId {
        let mut result = self.add_const(0.0);
        let add_one = self.add_const(1.0);

        for bm in 0..(1i32 << nodes.len()) {
            self.ref_node(add_one);
            let mut term = add_one;
            for i in 0..nodes.len() {
                let var = nodes[i];
                self.ref_node(var);
                let literal = if (bm & (1 << i)) != 0 {
                    var
                } else {
                    self.bdd_not(var)
                };
                term = self.bdd_and(term, literal);
            }
            let term = self.bdd_to_add(term);
            let value = self.add_const(bm as f64);
            let term = self.add_times(term, value);
            result = self.add_plus(result, term);
        }

        result
    }
}
