//! Graph algorithms over a loaded snapshot: a breadth-first search for shortest retention
//! paths, and dominators (Lengauer-Tarjan) with retained sizes.
//!
//! Both follow the edges that keep an object alive: weak edges are skipped, and shortcut
//! edges count only from the root (the rule the Chrome DevTools use). Both are iterative,
//! so a long linked list does not overflow the stack.

use std::time::Instant;

use super::{NodeError, snapshot::Snapshot};

const NONE: u32 = u32::MAX;

/// Shortest retention paths from the root.
#[derive(Debug)]
pub struct Bfs {
    /// The edge that first reached each node, `u32::MAX` for the root and for nodes
    /// that are not reachable.
    parent_edge: Vec<u32>,
    /// The node that edge starts at.
    parent: Vec<u32>,
    reached: Vec<bool>,
}

impl Bfs {
    /// True when the node can be reached from the root.
    pub fn reachable(&self, n: usize) -> bool {
        self.reached[n]
    }

    /// The edge by which `n` is retained on a shortest path, and its source node.
    pub fn retainer(&self, n: usize) -> Option<(usize, usize)> {
        (self.parent_edge[n] != NONE)
            .then(|| (self.parent[n] as usize, self.parent_edge[n] as usize))
    }

    /// The path from the root to `n` as `(source node, edge)` pairs, root first.
    pub fn path_to(&self, n: usize) -> Vec<(usize, usize)> {
        let mut out = Vec::new();
        let mut at = n;
        // A tree path is shorter than the node count; the bound guards a damaged table.
        for _ in 0..self.reached.len() {
            let Some((from, e)) = self.retainer(at) else {
                break;
            };
            out.push((from, e));
            at = from;
        }
        out.reverse();
        out
    }
}

/// Dominators and the sizes they give.
#[derive(Debug)]
pub struct Dominators {
    /// Immediate dominator of each node; the root dominates itself; `u32::MAX` for an
    /// unreachable node.
    idom: Vec<u32>,
    /// Bytes retained: the node's own size plus everything it dominates.
    retained: Vec<u64>,
    /// Objects retained, the node included.
    dominated: Vec<u64>,
    /// Reachable nodes in DFS order (the root first).
    order: Vec<u32>,
}

impl Dominators {
    /// The immediate dominator of `n`, `None` for the root and unreachable nodes.
    pub fn idom(&self, n: usize) -> Option<usize> {
        let d = self.idom[n];
        (d != NONE && d as usize != n).then_some(d as usize)
    }

    /// True when `n` is reachable from the root.
    pub fn reachable(&self, n: usize) -> bool {
        self.idom[n] != NONE
    }

    /// Bytes retained by `n`.
    pub fn retained(&self, n: usize) -> u64 {
        self.retained[n]
    }

    /// Objects retained by `n`, itself included.
    pub fn dominated(&self, n: usize) -> u64 {
        self.dominated[n]
    }

    /// Total bytes of the reachable heap.
    pub fn total(&self) -> u64 {
        self.retained.first().copied().unwrap_or(0)
    }

    /// Number of reachable nodes.
    pub fn reachable_count(&self) -> usize {
        self.order.len()
    }

    /// The reachable nodes in DFS order; a dominator comes before what it dominates.
    pub fn order(&self) -> &[u32] {
        &self.order
    }

    /// The dominator tree as a children list (`first[n]..first[n + 1]` into `kids`).
    pub fn children(&self) -> (Vec<u32>, Vec<u32>) {
        let n = self.idom.len();
        let mut first = vec![0u32; n + 2];
        for v in 0..n {
            if let Some(d) = self.idom(v) {
                first[d + 1] += 1;
            }
        }
        for i in 0..=n {
            first[i + 1] += first[i];
        }
        let mut fill = first.clone();
        let mut kids = vec![0u32; first[n + 1] as usize];
        for v in 0..n {
            if let Some(d) = self.idom(v) {
                kids[fill[d] as usize] = v as u32;
                fill[d] += 1;
            }
        }
        (first, kids)
    }
}

impl Snapshot {
    /// Breadth-first search from node 0.
    pub fn bfs(&self) -> Bfs {
        let n = self.node_count();
        let mut parent_edge = vec![NONE; n];
        let mut parent = vec![NONE; n];
        let mut reached = vec![false; n];
        let mut queue: Vec<u32> = Vec::with_capacity(n);
        if n > 0 {
            reached[0] = true;
            queue.push(0);
        }
        let mut head = 0;
        while head < queue.len() {
            let u = queue[head] as usize;
            head += 1;
            for e in self.edges_of(u) {
                if !self.essential(u, self.edge_type(e)) {
                    continue;
                }
                let v = self.edge_target(e);
                if !reached[v] {
                    reached[v] = true;
                    parent_edge[v] = e as u32;
                    parent[v] = u as u32;
                    queue.push(v as u32);
                }
            }
        }
        Bfs {
            parent_edge,
            parent,
            reached,
        }
    }

    /// Dominators by Lengauer-Tarjan, then retained sizes. Stops with `TooLarge` when
    /// `deadline` passes.
    pub fn dominators(&self, deadline: Option<Instant>) -> Result<Dominators, NodeError> {
        let n = self.node_count();
        // ---- iterative DFS from the root: dfn is 1-based, 0 means unvisited
        let mut dfn = vec![0u32; n];
        let mut vertex: Vec<u32> = Vec::with_capacity(n + 1);
        vertex.push(0);
        let mut parent: Vec<u32> = vec![0];
        let mut stack: Vec<(u32, u32)> = Vec::new();
        if n > 0 {
            dfn[0] = 1;
            vertex.push(0);
            parent.push(0);
            stack.push((0, self.edges_of(0).start as u32));
        }
        while let Some(top) = stack.last_mut() {
            let u = top.0 as usize;
            let end = self.edges_of(u).end as u32;
            if top.1 >= end {
                stack.pop();
                continue;
            }
            let e = top.1 as usize;
            top.1 += 1;
            if !self.essential(u, self.edge_type(e)) {
                continue;
            }
            let v = self.edge_target(e);
            if dfn[v] == 0 {
                let d = vertex.len() as u32;
                dfn[v] = d;
                vertex.push(v as u32);
                parent.push(dfn[u]);
                stack.push((v as u32, self.edges_of(v).start as u32));
            }
        }
        let m = vertex.len() - 1;
        // ---- predecessors in dfn space, as a compressed list
        let mut pfirst = vec![0u32; m + 2];
        for (i, &u) in vertex.iter().enumerate().skip(1) {
            let u = u as usize;
            for e in self.edges_of(u) {
                if self.essential(u, self.edge_type(e)) {
                    let w = dfn[self.edge_target(e)] as usize;
                    if w != 0 && w != i {
                        pfirst[w + 1] += 1;
                    }
                }
            }
        }
        for i in 0..=m {
            pfirst[i + 1] += pfirst[i];
        }
        let mut fill = pfirst.clone();
        let mut preds = vec![0u32; pfirst[m + 1] as usize];
        for (i, &u) in vertex.iter().enumerate().skip(1) {
            let u = u as usize;
            for e in self.edges_of(u) {
                if self.essential(u, self.edge_type(e)) {
                    let w = dfn[self.edge_target(e)] as usize;
                    if w != 0 && w != i {
                        preds[fill[w] as usize] = i as u32;
                        fill[w] += 1;
                    }
                }
            }
        }
        drop(fill);
        // ---- Lengauer-Tarjan in dfn space
        let mut semi: Vec<u32> = (0..=m as u32).collect();
        let mut ancestor = vec![0u32; m + 1];
        let mut label: Vec<u32> = (0..=m as u32).collect();
        let mut dom = vec![0u32; m + 1];
        let mut bucket_head = vec![0u32; m + 1];
        let mut bucket_next = vec![0u32; m + 1];
        let mut chain: Vec<u32> = Vec::new();
        // Finds the vertex with the smallest semidominator on the path to the forest root.
        fn eval(
            v: u32,
            semi: &[u32],
            ancestor: &mut [u32],
            label: &mut [u32],
            chain: &mut Vec<u32>,
        ) -> u32 {
            if ancestor[v as usize] == 0 {
                return v;
            }
            chain.clear();
            let mut x = v;
            while ancestor[ancestor[x as usize] as usize] != 0 {
                chain.push(x);
                x = ancestor[x as usize];
            }
            for &x in chain.iter().rev() {
                let a = ancestor[x as usize];
                if semi[label[a as usize] as usize] < semi[label[x as usize] as usize] {
                    label[x as usize] = label[a as usize];
                }
                ancestor[x as usize] = ancestor[a as usize];
            }
            label[v as usize]
        }
        for w in (2..=m).rev() {
            if w % 65_536 == 0
                && let Some(d) = deadline
                && Instant::now() > d
            {
                return Err(NodeError::TooLarge(
                    "the dominator computation ran out of time".to_owned(),
                ));
            }
            for k in pfirst[w]..pfirst[w + 1] {
                let v = preds[k as usize];
                let u = eval(v, &semi, &mut ancestor, &mut label, &mut chain);
                if semi[u as usize] < semi[w] {
                    semi[w] = semi[u as usize];
                }
            }
            let s = semi[w] as usize;
            bucket_next[w] = bucket_head[s];
            bucket_head[s] = w as u32;
            let p = parent[w] as usize;
            ancestor[w] = p as u32;
            let mut b = bucket_head[p];
            bucket_head[p] = 0;
            while b != 0 {
                let next = bucket_next[b as usize];
                let u = eval(b, &semi, &mut ancestor, &mut label, &mut chain);
                dom[b as usize] = if semi[u as usize] < semi[b as usize] {
                    u
                } else {
                    p as u32
                };
                b = next;
            }
        }
        for w in 2..=m {
            if dom[w] != semi[w] {
                dom[w] = dom[dom[w] as usize];
            }
        }
        if m >= 1 {
            dom[1] = 1;
        }
        // ---- sizes, children first
        let mut ret: Vec<u64> = vec![0; m + 1];
        let mut cnt: Vec<u64> = vec![0; m + 1];
        for i in 1..=m {
            ret[i] = self.self_size(vertex[i] as usize);
            cnt[i] = 1;
        }
        for i in (2..=m).rev() {
            let d = dom[i] as usize;
            ret[d] = ret[d].saturating_add(ret[i]);
            cnt[d] += cnt[i];
        }
        let mut idom = vec![NONE; n];
        let mut retained = vec![0u64; n];
        let mut dominated = vec![0u64; n];
        for i in 1..=m {
            let v = vertex[i] as usize;
            idom[v] = vertex[dom[i] as usize];
            retained[v] = ret[i];
            dominated[v] = cnt[i];
        }
        Ok(Dominators {
            idom,
            retained,
            dominated,
            order: vertex[1..].to_vec(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::super::snapshot::testing::*;
    use super::*;

    fn load(nodes: &[N<'_>]) -> Snapshot {
        let t = build(nodes);
        Snapshot::read(t.as_bytes(), t.len() as u64).unwrap()
    }

    #[test]
    fn a_diamond_is_dominated_by_its_top() {
        // 0 -> 1 -> {2, 3} -> 4 ; 4 is dominated by 1, not by 2 or 3.
        let s = load(&[
            ("synthetic", "", 0, vec![("element", "0", 1)]),
            (
                "object",
                "A",
                10,
                vec![("property", "l", 2), ("property", "r", 3)],
            ),
            ("object", "L", 20, vec![("property", "x", 4)]),
            ("object", "R", 30, vec![("property", "x", 4)]),
            ("object", "X", 40, vec![]),
            ("object", "Weak", 1000, vec![]),
        ]);
        let d = s.dominators(None).unwrap();
        assert_eq!(d.idom(4), Some(1));
        assert_eq!(d.idom(2), Some(1));
        assert_eq!(d.retained(1), 100);
        assert_eq!(d.retained(2), 20);
        assert_eq!(d.dominated(1), 4);
        // Node 5 has no edge to it.
        assert!(!d.reachable(5));
        assert_eq!(d.total(), 100);
    }

    #[test]
    fn weak_edges_do_not_retain_and_shortcuts_count_only_from_the_root() {
        let s = load(&[
            ("synthetic", "", 0, vec![("shortcut", "g", 1)]),
            (
                "object",
                "Kept",
                10,
                vec![("weak", "w", 2), ("shortcut", "s", 3)],
            ),
            ("object", "OnlyWeak", 50, vec![]),
            ("object", "OnlyShortcut", 60, vec![]),
        ]);
        let d = s.dominators(None).unwrap();
        assert!(d.reachable(1));
        assert!(!d.reachable(2));
        assert!(!d.reachable(3));
        let b = s.bfs();
        assert!(b.reachable(1) && !b.reachable(2));
    }

    #[test]
    fn a_long_chain_does_not_overflow_and_paths_come_back() {
        let mut nodes: Vec<N<'_>> = vec![("synthetic", "", 0, vec![("element", "0", 1)])];
        let n = 20_000;
        for i in 1..=n {
            let edges = if i < n {
                vec![("property", "next", i + 1)]
            } else {
                vec![]
            };
            nodes.push(("object", "Link", 8, edges));
        }
        let s = load(&nodes);
        let d = s.dominators(None).unwrap();
        assert_eq!(d.retained(1), 8 * n as u64);
        assert_eq!(d.idom(n), Some(n - 1));
        let b = s.bfs();
        let path = b.path_to(n);
        assert_eq!(path.len(), n);
        assert_eq!(s.edge_name(path[1].1), Some("next"));
    }

    #[test]
    fn cycles_and_cross_edges_keep_the_dominator_property() {
        // 0 -> 1 -> 2 -> 3 -> 1 (cycle), 2 -> 4, 3 -> 4
        let s = load(&[
            ("synthetic", "", 0, vec![("element", "0", 1)]),
            ("object", "A", 1, vec![("property", "b", 2)]),
            (
                "object",
                "B",
                2,
                vec![("property", "c", 3), ("property", "e", 4)],
            ),
            (
                "object",
                "C",
                4,
                vec![("property", "a", 1), ("property", "e", 4)],
            ),
            ("object", "E", 8, vec![]),
        ]);
        let d = s.dominators(None).unwrap();
        assert_eq!(d.idom(4), Some(2));
        assert_eq!(d.idom(3), Some(2));
        assert_eq!(d.retained(2), 14);
    }
}
