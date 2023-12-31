use std::cmp::Reverse;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use crate::util::disjoint_set_union::DisjointSetUnion;

pub struct GraphEdge<T: Copy + Hash + Eq + Default> {
    pub a: T,
    pub b: T,
    pub weight: u32,
}

#[inline(never)]
pub fn compute_approx_steiner_tree<T: Copy + Hash + Eq + Default>(required: &Vec<T>, edges: &Vec<GraphEdge<T>>) -> (Vec<(T, T)>, u32) {
    if required.len() < 2 {
        return (Vec::new(), 0);
    }

    let mut v2i = HashMap::new();
    for e in edges {
        let next_i = v2i.len();
        v2i.entry(e.a).or_insert(next_i as u32);
        let next_i = v2i.len();
        v2i.entry(e.b).or_insert(next_i as u32);
    }

    let req_u32 = required.iter().map(|x| *v2i.get(x).unwrap()).collect();
    let mut adj_list = vec![Vec::new(); v2i.len()];
    edges.iter()
        .map(|x| (*v2i.get(&x.a).unwrap(), *v2i.get(&x.b).unwrap(), x.weight))
        .for_each(|(a, b, w)| {
            adj_list[a as usize].push(AdjListEdge {other: b, weight: w});
            adj_list[b as usize].push(AdjListEdge {other: a, weight: w});
        });

    let steiner_tree = compute_approx_steiner_tree_u32(&req_u32, &adj_list);

    let mut i2v = vec![T::default(); v2i.len()];
    v2i.iter().for_each(|(k, v)| i2v[*v as usize] = *k);

    let edges = steiner_tree.edges.iter().map(|(a, b)| (i2v[*a as usize], i2v[*b as usize])).collect();

    (edges, steiner_tree.cost)
}

#[derive(Clone, Copy)]
struct AdjListEdge {
    other: u32,
    weight: u32
}

struct SteinerTreeU32Result {
    edges: Vec<(u32, u32)>,
    cost: u32,
}

#[inline(never)]
fn compute_approx_steiner_tree_u32(required: &Vec<u32>, adj_list: &Vec<Vec<AdjListEdge>>) -> SteinerTreeU32Result {
    if required.len() < 2 {
        return SteinerTreeU32Result {
            edges: Vec::new(),
            cost: 0,
        };
    }
    assert_eq!(HashSet::<u32>::from_iter(required.iter().map(|x| *x)).len(), required.len());

    let n = adj_list.len();
    let mut prev = (0..(n as u32)).collect::<Vec<_>>();
    let mut prev_edge_weight = vec![0; n];
    let mut dist = vec![u32::MAX; n];
    required.iter().for_each(|x| dist[*x as usize] = 0);
    let mut pq = std::collections::BinaryHeap::new();
    required.iter().for_each(|x| pq.push((Reverse(0), *x)));

    let mut dsu = DisjointSetUnion::new(n);

    // Perform dijkstra's starting from each required vertex, computing a "prev" array consisting of the shortest path.
    // Do DSU along the way. At the end of Dijkstra's, the prev array forms N forests, one for each required node.
    // Each forest corresponds to one DSU set.
    while !pq.is_empty() {
        let (Reverse(d_x), x) = pq.pop().unwrap();

        if dist[x as usize] != d_x {
            // stale/outdated pq entry
            continue;
        }

        dsu.union(prev[x as usize] as usize, x as usize);

        for e_y in adj_list[x as usize].iter() {
            let d_y = d_x + e_y.weight;
            let y = e_y.other;
            if d_y < dist[y as usize] {
                dist[y as usize] = d_y;
                pq.push((Reverse(d_y), y));
                prev[y as usize] = x;
                prev_edge_weight[y as usize] = e_y.weight;
            }
        }
    }

    let mut edge_candidates = std::collections::BinaryHeap::new();
    for (x, edges) in adj_list.iter().enumerate() {
        for y in edges.iter() {
            if dsu.find(x) != dsu.find(y.other as usize) {
                assert_ne!(dist[x], u32::MAX);
                assert_ne!(dist[y.other as usize], u32::MAX);
                let total_w = y.weight + dist[x] + dist[y.other as usize];
                edge_candidates.push((Reverse(total_w), y.weight, x as u32, y.other));
            }
        }
    }

    let mut steiner_edges = HashSet::new();
    let mut steiner_weight = 0;
    let mut unions_done = 0;
    while !edge_candidates.is_empty() {
        let (_, w, x, y) = edge_candidates.pop().unwrap();
        if !dsu.union(x as usize, y as usize) {
            continue;
        }
        steiner_weight += w;
        assert!(!steiner_edges.contains(&(x, y)));
        assert!(!steiner_edges.contains(&(y, x)));
        steiner_edges.insert((x, y));

        let mut cur = x;
        loop {
            let prv = prev[cur as usize];
            if prv == cur {
                break;
            }
            let edge = (prv, cur);
            let e2 = (cur, prv);
            assert!(!steiner_edges.contains(&e2));
            if steiner_edges.contains(&edge) {
                break;
            }
            steiner_weight += prev_edge_weight[cur as usize];
            steiner_edges.insert(edge);
            cur = prv;
        }

        let mut cur = y;
        loop {
            let prv = prev[cur as usize];
            if prv == cur {
                break;
            }
            let edge = (prv, cur);
            let e2 = (cur, prv);
            assert!(!steiner_edges.contains(&e2));
            if steiner_edges.contains(&edge) {
                break;
            }
            steiner_weight += prev_edge_weight[cur as usize];
            steiner_edges.insert(edge);
            cur = prv;
        }

        unions_done += 1;
    }
    assert_eq!(unions_done + 1, required.len());

    SteinerTreeU32Result {
        edges: steiner_edges.into_iter().collect(),
        cost: steiner_weight,
    }
}