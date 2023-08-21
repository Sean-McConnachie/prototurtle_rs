use std::arch::x86_64::_mm_sha1msg1_epu32;
use std::collections::{HashMap, HashSet};
use std::io::{Read, Write};
use std::path;
use modelutils_rs::model2arr::{ArrayModel, Block, CoordXZ, int, uint};
use crate::{turtle, nav, inventory};

fn cost(a: CoordXZ, b: CoordXZ) -> uint {
    let dx = (a.0 as int - b.0 as int).abs();
    let dz = (a.1 as int - b.1 as int).abs();
    if dx == 0 || dz == 0 {
        return (dx + dz) as u16;
    }
    (dx + dz + 1) as u16
}

pub fn mst_from_nodes(mut nodes: Vec<(CoordXZ, Block)>) -> HashMap<CoordXZ, (Block, Vec<CoordXZ>)> {
    let mut edges: Vec<(CoordXZ, CoordXZ, u16)> = Vec::new();
    for i in 0..nodes.len() {
        for j in i + 1..nodes.len() {
            let (node_a, _block_a) = nodes[i];
            let (node_b, _block_b) = nodes[j];
            let cost = cost(node_a, node_b);
            edges.push((node_a, node_b, cost));
        }
    }
    edges.sort_by_key(|&(_, _, cost)| cost);

    let mut parent: HashMap<CoordXZ, CoordXZ> = HashMap::new();
    for (node, _block) in &nodes {
        parent.insert(*node, *node);
    }

    fn find(mut node: CoordXZ, parent: &mut HashMap<CoordXZ, CoordXZ>) -> CoordXZ {
        while parent[&node] != node {
            node = parent[&node];
        }
        node
    }

    let mut minimum_spanning_tree: HashMap<CoordXZ, (Block, Vec<CoordXZ>)> = HashMap::new();
    for (node_a, node_b, cost) in edges {
        let root_a = find(node_a, &mut parent);
        let root_b = find(node_b, &mut parent);
        if root_a != root_b {
            // Update the adjacency list for the parent node
            minimum_spanning_tree.entry(root_a).or_insert(
                (Block::MIN, Vec::new())
            ).1.push(node_b);

            parent.insert(root_a, root_b);
        }
    }

    for (node, (block, _)) in minimum_spanning_tree.iter_mut() {
        if let Some((_original_node, original_block)) = nodes.iter().find(|(n, _)| *n == *node) {
            *block = *original_block;
        }
    }

    minimum_spanning_tree
}

fn mst_to_path(mst: HashMap<CoordXZ, (Block, Vec<CoordXZ>)>) -> Vec<(Block, CoordXZ)> {
    if mst.is_empty() { return Vec::new(); }

    let mut path: Vec<(Block, CoordXZ)> = Vec::new();
    let mut stack: Vec<CoordXZ> = Vec::new();
    let mut visited: HashSet<&CoordXZ> = HashSet::new();

    let node = mst.keys().next().unwrap();
    stack.push(*node);
    visited.insert(node);

    // DFS
    while !stack.is_empty() {
        let node = stack.pop().unwrap();

        if let Some(desc) = mst.get(&node) {
            path.push((desc.0, node));
            for child in desc.1.iter() {
                if !visited.contains(child) {
                    stack.push(*child);
                    visited.insert(child);
                }
            }
        }
    }

    path
}

pub struct MultiBuilder<'a> {
    // Used for controlling the turtle
    turt: &'a turtle::Turt<'a>,
    nav: &'a mut nav::Nav<'a>,
    inv: inventory::Inventory<'a>,

    // Build data must be kept in function parameter

    // Persistent data
    start_pos: nav::PosH,
    start_layer: usize,
    total_blocks: usize,
    fp: path::PathBuf,
}

impl<'a> MultiBuilder<'a> {
    pub fn new(
        start_pos: nav::PosH,
        turtle_id: usize,
        turt: &'a turtle::Turt<'a>,
        nav: &'a mut nav::Nav<'a>,
    ) -> Self {
        Self {
            start_pos,
            turt,
            nav,
            inv: inventory::Inventory::init(&turt),

            start_layer: 0,
            total_blocks: 0,
            fp: path::PathBuf::from(format!("progress/{turtle_id}.turtle")),
        }
    }

    pub fn get_nodes(array_model: ArrayModel) -> Vec<Vec<(CoordXZ, Block)>> {
        let mut layer_nodes = Vec::with_capacity(array_model.dims.1 as usize);
        for y in 0..array_model.dims.1 {
            let mut nodes = vec![];
            for x in 0..array_model.dims.0 {
                for z in 0..array_model.dims.2 {
                    let block = array_model.get((x as usize, y as usize, z as usize));
                    if block != 0 {
                        nodes.push(((x, z), block));
                    }
                }
            }
            layer_nodes.push(nodes);
        }
        layer_nodes
    }

    pub fn get_or_init_progress(&mut self) {
        if !self.fp.exists() {
            self.save_progress()
        } else {
            let lines: Vec<String> = std::fs::read_to_string(&self.fp)
                .unwrap()
                .lines()
                .map(String::from)
                .collect();
            self.start_layer = lines[0].parse::<usize>().unwrap();
            self.total_blocks = lines[1].parse::<usize>().unwrap();
        }
    }

    pub fn save_progress(&self) {
        let mut file = std::fs::File::create(&self.fp).unwrap();
        file.write_all(format!("{}\n{}\n", self.start_layer, self.total_blocks).as_bytes()).unwrap();
    }

    pub fn run(&mut self, nodes: Vec<Vec<(CoordXZ, Block)>>) {
        fn world_coord(start: &nav::PosH, coord: CoordXZ, y: usize) -> nav::Pos {
            nav::Pos {
                x: start.x + coord.0 as i64,
                y: start.y + y as i64,
                z: start.z + coord.1 as i64,
            }
        }

        fn relative_coord(start: &nav::PosH, coord: &nav::PosH) -> CoordXZ {
            ((coord.x - start.x) as uint, (coord.z - start.z) as uint)
        }

        let mut blocks_placed = 0;

        for (y, layer) in nodes
            .into_iter().skip(self.start_layer).enumerate() {
            if layer.is_empty() { continue; }

            self.start_layer = y;
            self.save_progress();

            let rel_coord = relative_coord(&self.start_pos, self.nav.pos());
            let mst = mst_from_nodes(layer);
            let path = mst_to_path(mst);

            for (_block, coord) in path {
                self.nav.goto_nohead(&world_coord(&self.start_pos, coord, y), nav::Order::YXZ);
                self.turt.inv_select(((blocks_placed / 64) % 16) as u8);
                self.turt.p_down();
            }
        }
    }
}