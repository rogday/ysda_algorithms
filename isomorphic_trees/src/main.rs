#![allow(dead_code)]

use std::collections::{HashMap, VecDeque};

use either::Either;

#[derive(Hash, Debug)]
struct Tree {
    inner: Vec<Vec<usize>>,
}

#[derive(Copy, Clone)]
struct MappingInfo<'a> {
    tree:      &'a Tree,
    vertex:    usize,
    vertex_id: &'a [usize],
}

impl Tree {
    fn size(&self) -> usize {
        self.inner.len()
    }

    fn get_farthest(&self, mut vertex: usize) -> (usize, Vec<usize>) {
        let n = self.size();
        let mut stack: VecDeque<usize> = vec![vertex].into();
        let mut path = vec![n + 1; n];
        path[vertex] = n;

        while let Some(current) = stack.pop_back() {
            vertex = current;

            for &child in &self.inner[vertex] {
                if path[child] != n + 1 {
                    continue;
                }

                path[child] = vertex;
                stack.push_front(child);
            }
        }

        (vertex, path)
    }

    fn get_roots(&self) -> Either<usize, (usize, usize)> {
        let (u, _) = self.get_farthest(0);
        let (mut u, path) = self.get_farthest(u);

        let mut trace = vec![u];
        while path[u] != self.size() {
            u = path[u];
            trace.push(u);
        }

        let n = trace.len();
        if n % 2 == 1 {
            Either::Left(trace[n / 2])
        } else {
            Either::Right((trace[n / 2 - 1], trace[n / 2]))
        }
    }

    fn dfs_order(&self, vertex: usize) -> Vec<usize> {
        let mut visited = vec![false; self.size()];
        let mut stack = vec![vertex];
        let mut order = vec![];

        while let Some(vertex) = stack.pop() {
            visited[vertex] = true;
            order.push(vertex);

            for &child in &self.inner[vertex] {
                if !visited[child] {
                    stack.push(child);
                }
            }
        }

        order.reverse();
        order
    }

    fn gen_mapping(lhs: MappingInfo, rhs: MappingInfo) -> Vec<usize> {
        let n = lhs.tree.size();

        let mut lhs_stack = vec![lhs.vertex];
        let mut rhs_stack = vec![rhs.vertex];

        let mut mapping = vec![n; n];

        while let (Some(lhs_vertex), Some(rhs_vertex)) = (lhs_stack.pop(), rhs_stack.pop()) {
            mapping[lhs_vertex] = rhs_vertex;

            let m = lhs_stack.len();
            let len = lhs.tree.inner[lhs_vertex].len();

            for i in 0..len {
                let vertex = lhs.tree.inner[lhs_vertex][i];
                if mapping[vertex] == n {
                    lhs_stack.push(vertex);
                    rhs_stack.push(rhs.tree.inner[rhs_vertex][i])
                }
            }

            lhs_stack[m.saturating_sub(1)..]
                .sort_unstable_by(|&x, &y| lhs.vertex_id[x].cmp(&lhs.vertex_id[y]));

            rhs_stack[m.saturating_sub(1)..]
                .sort_unstable_by(|&x, &y| rhs.vertex_id[x].cmp(&rhs.vertex_id[y]));
        }

        mapping
    }

    fn get_ids(
        &self,
        start: usize,
        mut functor: impl FnMut(&mut usize, &[usize]) -> Option<()>,
    ) -> Option<Vec<usize>> {
        let order = self.dfs_order(start);
        let mut vertex_id = vec![self.size(); self.size()];
        let mut child_ids = vec![];

        for &vertex in &order {
            child_ids.clear();
            for &child in &self.inner[vertex] {
                child_ids.push(vertex_id[child]);
            }
            child_ids.sort_unstable();

            functor(&mut vertex_id[vertex], &child_ids)?;
        }

        Some(vertex_id)
    }

    fn is_isomorphic(&self, rhs_tree: &Self) -> Option<Vec<usize>> {
        match (self.get_roots(), rhs_tree.get_roots()) {
            (Either::Left(lhs), Either::Left(rhs)) => self.try_match(rhs_tree, lhs, rhs),
            (Either::Right((lhs_0, lhs_1)), Either::Right((rhs, _))) => self
                .try_match(rhs_tree, lhs_0, rhs)
                .or_else(|| self.try_match(rhs_tree, lhs_1, rhs)),
            _ => None,
        }
    }

    fn try_match(&self, rhs: &Self, lhs_center: usize, rhs_center: usize) -> Option<Vec<usize>> {
        let mut map = HashMap::new();

        let mut id: usize = 0;

        let lhs_ids = self.get_ids(lhs_center, |vertex, child_ids| {
            if let Some(&found_id) = map.get(child_ids) {
                *vertex = found_id;
            } else {
                map.insert(child_ids.to_vec(), id);
                *vertex = id;
                id += 1;
            };
            Some(())
        })?;

        let rhs_ids = rhs.get_ids(rhs_center, |vertex, child_ids| {
            *vertex = *map.get(child_ids)?;
            Some(())
        })?;

        if lhs_ids[lhs_center] != rhs_ids[rhs_center] {
            return None;
        }

        Some(Tree::gen_mapping(
            MappingInfo { tree: self, vertex: lhs_center, vertex_id: &lhs_ids },
            MappingInfo { tree: rhs, vertex: rhs_center, vertex_id: &rhs_ids },
        ))
    }
}

#[cfg(test)]
mod tests {
    use rand::{rngs::SmallRng, seq::SliceRandom, thread_rng, Rng, SeedableRng};

    use super::*;

    fn gen_tree(size: usize) -> (u64, Vec<Vec<usize>>) {
        let seed = thread_rng().gen();
        let mut prng: SmallRng = SeedableRng::seed_from_u64(seed);

        let prufer_code: Vec<usize> = (0..size - 2).map(|_| prng.gen_range(0, size)).collect();
        (seed, get_edges(&prufer_code))
    }

    fn get_edges(prufer_code: &[usize]) -> Vec<Vec<usize>> {
        let size = prufer_code.len() + 2;
        let mut degree = vec![1; size];

        prufer_code.iter().for_each(|&x| degree[x] += 1);

        let mut ptr = 0;
        while degree[ptr] != 1 {
            ptr += 1;
        }

        let mut leaf = ptr;
        let mut graph = vec![vec![]; size];

        for &v in prufer_code {
            graph[leaf].push(v);
            graph[v].push(leaf);

            degree[v] -= 1;

            if degree[v] == 1 && v < ptr {
                leaf = v;
            } else {
                ptr += 1;
                while degree[ptr] != 1 {
                    ptr += 1;
                }
                leaf = ptr;
            }
        }
        graph[leaf].push(size - 1);
        graph[size - 1].push(leaf);

        graph
    }

    #[test]
    /// for tree size in 3..N gen every tree, gen every permutation of mapping
    fn small_correctness() {}

    #[test]
    /// for tree size in random O(N) gen random tree, gen random permutation of mapping
    fn random_big_correctness() {
        let tree_size = 100_000;
        let iterations = 1_000;

        for i in 0..iterations {
            let (seed, reference_tree) = gen_tree(tree_size);

            let mapping = {
                let mut tmp: Vec<usize> = (0..tree_size).collect();
                tmp.shuffle(&mut thread_rng());
                tmp
            };

            let mut isomorphic_tree = vec![vec![]; tree_size];
            for i in 0..tree_size {
                isomorphic_tree[mapping[i]] = reference_tree[i].clone();
            }
            for vertex in &mut isomorphic_tree {
                for child in vertex.iter_mut() {
                    *child = mapping[*child];
                }
            }

            let reference_tree = Tree { inner: reference_tree };
            let mut isomorphic_tree = Tree { inner: isomorphic_tree };

            let now = std::time::Instant::now();
            let result = reference_tree.is_isomorphic(&isomorphic_tree);
            println!("{:?} per iter", now.elapsed().as_millis());
            match result {
                None => panic!("блRRRRRRRRRRRR"),
                Some(mapping) => {
                    let mut mapped_tree = vec![vec![]; tree_size];
                    for i in 0..tree_size {
                        mapped_tree[mapping[i]] = reference_tree.inner[i].clone();
                    }
                    for vertex in &mut mapped_tree {
                        for child in vertex.iter_mut() {
                            *child = mapping[*child];
                        }
                        vertex.sort_unstable();
                    }
                    for vertex in &mut isomorphic_tree.inner {
                        vertex.sort_unstable();
                    }

                    if mapped_tree != isomorphic_tree.inner {
                        assert!(false);
                        println!("=(");
                        println!("{:?}\n{:?}\n{:?}", reference_tree, isomorphic_tree, mapped_tree);
                    }
                }
            }
        }
    }
}

fn main() {}
