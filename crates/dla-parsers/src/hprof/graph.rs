//! The strong reference graph in compressed rows, its dominator tree (Cooper, Harvey and
//! Kennedy) and retained sizes.

use super::ids::NONE;

/// Object nodes `0..n`, then synthetic holder nodes, then the super root.
pub(crate) struct Graph {
    /// Number of object nodes.
    pub n: usize,
    /// Row starts of the object nodes (`n + 1` entries).
    pub eoff: Vec<u32>,
    /// Targets.
    pub edges: Vec<u32>,
    /// Successors of the synthetic nodes `n..n + syn.len()`.
    pub syn: Vec<Vec<u32>>,
    /// Successors of the super root.
    pub root_succ: Vec<u32>,
}

impl Graph {
    /// The super root.
    pub fn root(&self) -> u32 {
        (self.n + self.syn.len()) as u32
    }

    /// Total nodes, super root included.
    pub fn nodes(&self) -> usize {
        self.n + self.syn.len() + 1
    }

    #[inline]
    pub fn succ(&self, v: u32) -> &[u32] {
        let i = v as usize;
        if i < self.n {
            &self.edges[self.eoff[i] as usize..self.eoff[i + 1] as usize]
        } else if i < self.n + self.syn.len() {
            &self.syn[i - self.n]
        } else {
            &self.root_succ
        }
    }
}

/// The dominator tree over the nodes reachable from the super root.
pub(crate) struct Dominators {
    /// Immediate dominator per node ([`NONE`] when unreachable; the root dominates itself).
    pub idom: Vec<u32>,
    /// Reachable nodes in post-order.
    pub post: Vec<u32>,
    /// Iterations run.
    pub passes: u32,
    /// False when the pass cap stopped the iteration.
    pub converged: bool,
}

/// Computes dominators with at most `max_passes` iterations.
pub(crate) fn dominators(g: &Graph, max_passes: u32) -> Dominators {
    let nn = g.nodes();
    let root = g.root();
    let mut post: Vec<u32> = Vec::new();
    let mut pno = vec![NONE; nn];
    let mut seen = vec![false; nn];
    let mut stack: Vec<(u32, u32)> = vec![(root, 0)];
    seen[root as usize] = true;
    while let Some(&mut (v, ref mut i)) = stack.last_mut() {
        let s = g.succ(v);
        if (*i as usize) < s.len() {
            let w = s[*i as usize];
            *i += 1;
            if !seen[w as usize] {
                seen[w as usize] = true;
                stack.push((w, 0));
            }
        } else {
            pno[v as usize] = post.len() as u32;
            post.push(v);
            stack.pop();
        }
    }
    drop(seen);
    drop(stack);
    // Predecessors of the reachable nodes.
    let mut pstart = vec![0u32; nn + 1];
    for &v in &post {
        for &w in g.succ(v) {
            pstart[w as usize + 1] += 1;
        }
    }
    for i in 0..nn {
        pstart[i + 1] += pstart[i];
    }
    let mut fill = pstart.clone();
    let mut preds = vec![0u32; pstart[nn] as usize];
    for &v in &post {
        for &w in g.succ(v) {
            preds[fill[w as usize] as usize] = v;
            fill[w as usize] += 1;
        }
    }
    drop(fill);
    let mut idom = vec![NONE; nn];
    idom[root as usize] = root;
    let mut passes = 0;
    let mut converged = false;
    while passes < max_passes {
        passes += 1;
        let mut changed = false;
        for &v in post.iter().rev() {
            if v == root {
                continue;
            }
            let mut ni = NONE;
            for &p in &preds[pstart[v as usize] as usize..pstart[v as usize + 1] as usize] {
                if idom[p as usize] == NONE {
                    continue;
                }
                if ni == NONE {
                    ni = p;
                } else {
                    let (mut a, mut b) = (p, ni);
                    while a != b {
                        while pno[a as usize] < pno[b as usize] {
                            a = idom[a as usize];
                        }
                        while pno[b as usize] < pno[a as usize] {
                            b = idom[b as usize];
                        }
                    }
                    ni = a;
                }
            }
            if idom[v as usize] != ni {
                idom[v as usize] = ni;
                changed = true;
            }
        }
        if !changed {
            converged = true;
            break;
        }
    }
    Dominators {
        idom,
        post,
        passes,
        converged,
    }
}

/// Retained bytes and dominated object counts per node: each node's own shallow size
/// (zero for synthetic nodes) plus its dominator subtree.
pub(crate) fn retained(g: &Graph, d: &Dominators, shallow: &[u32]) -> (Vec<u64>, Vec<u32>) {
    let nn = g.nodes();
    let mut ret = vec![0u64; nn];
    let mut cnt = vec![0u32; nn];
    for i in 0..g.n {
        ret[i] = u64::from(shallow[i]);
        cnt[i] = 1;
    }
    let root = g.root();
    for &v in &d.post {
        if v == root {
            continue;
        }
        let p = d.idom[v as usize];
        if p == NONE {
            continue;
        }
        ret[p as usize] += ret[v as usize];
        cnt[p as usize] += cnt[v as usize];
    }
    (ret, cnt)
}
