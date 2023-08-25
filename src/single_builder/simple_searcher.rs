//! This definitely doesn't have a good runtime (:


/// (x, z)
pub type CoordXZ = (usize, usize);
pub type Placed = bool;

pub struct SimpleSearcher {
    pub heuristic: Vec<Vec<usize>>,

    pub nodes: Vec<(Placed, CoordXZ)>,
}

impl SimpleSearcher {
    pub fn new(nodes: Vec<CoordXZ>) -> Self {
        Self {
            heuristic: Self::calc_heuristic(&nodes),
            nodes: nodes.into_iter().map(|n| (false, n)).collect(),
        }
    }

    /// Manhatten_distance (+1 if not on same axis)
    pub fn heuristic(a: &CoordXZ, b: &CoordXZ) -> usize {
        let manhatten = ((a.0 as i64 - b.0 as i64).abs() + (a.1 as i64 - b.1 as i64).abs()) as usize;
        if a.0 == b.0 || a.1 == b.1 {
            return manhatten + 1;
        }
        manhatten
    }

    fn calc_heuristic(nodes: &Vec<CoordXZ>) -> Vec<Vec<usize>> {
        let mut heuristic= Vec::with_capacity(nodes.len());
        for (_i, node) in nodes.iter().enumerate() {
            // heuristic.push(vec![0; nodes.len()]);
            let mut row = Vec::with_capacity(nodes.len());
            for (j, other_node) in nodes.iter().enumerate() {
                // if node.0 == other_node.0 && node.1 == other_node.1 {
                //     heuristic[i][j] = usize::MAX;
                //     continue;
                // }
                row.push((j, Self::heuristic(node, other_node)));
            }
            row.sort_by(|a, b| a.1.cmp(&b.1));
            let new = row.iter().map(|n| n.0).collect::<Vec<usize>>();
            heuristic.push(new);
        }
        heuristic
    }

    pub fn get_node(&self, node: &CoordXZ) -> Option<&(Placed, CoordXZ)> {
        for n in &self.nodes {
            if n.1 == *node {
                return Some(n);
            }
        }
        None
    }

    pub fn find_closest(&self, node: &CoordXZ) -> Option<usize> {
        let mut closest = None;
        let mut closest_dist = usize::MAX;
        for (i, n) in self.nodes.iter().enumerate() {
            let dist = Self::heuristic(node, &n.1);
            if dist < closest_dist {
                closest = Some(i);
                closest_dist = dist;
            }
        }
        closest
    }

    pub fn next_node(&mut self, curr_node: usize) -> Option<usize> {
        for node in &self.heuristic[curr_node] {
            if !self.nodes[*node].0 {
                self.nodes[*node].0 = true;
                return Some(*node);
            }
        }
        None
    }
}
