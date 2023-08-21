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

type Node = uint;
type Cost = uint;
type Edge = (Node, Node, Cost);

fn find_parent(parents: &mut Vec<Node>, node: Node) -> Node {
    if parents[node as usize] == node {
        node
    } else {
        parents[node as usize] = find_parent(parents, parents[node as usize]);
        parents[node as usize]
    }
}

fn union(parents: &mut Vec<Node>, node_a: Node, node_b: Node) {
    let parent_a = find_parent(parents, node_a);
    let parent_b = find_parent(parents, node_b);
    if parent_a == parent_b { return; }
    parents[parent_a as usize] = parent_b;
}

fn nodes_to_mst_to_path(nodes: &Vec<(CoordXZ, Block)>) -> Vec<uint> {
    // Kruskal MST
    let n = nodes.len();
    let mut edges: Vec<Edge> = Vec::with_capacity(n * n);
    for i in 0..n {
        for j in i + 1..n {
            let (node_a, _block_a) = nodes[i];
            let (node_b, _block_b) = nodes[j];
            let cost: Cost = cost(node_a, node_b);
            edges.push((i as Node, j as Node, cost));
        }
    }
    edges.sort_by_key(|&(_, _, cost)| cost);

    let mut parents: Vec<Node> = (0..n).into_iter().map(|u| u as Node).collect();
    let mut adj_list: Vec<Vec<Node>> = (0..n).into_iter().map(|_| Vec::new()).collect();

    for (node_a, node_b, cost) in edges {
        if find_parent(&mut parents, node_a) != find_parent(&mut parents, node_b) {
            union(&mut parents, node_a, node_b);
            adj_list[node_a as usize].push(node_b);
        }
    }

    // DFS Path
    let mut stack: Vec<Node> = Vec::new();
    let mut visited: Vec<bool> = (0..n).into_iter().map(|_| false).collect();
    let mut path: Vec<uint> = Vec::with_capacity(n - 1);

    stack.push(0);
    while !stack.is_empty() {
        let node = stack.pop().unwrap();
        if !visited[node as usize] {
            visited[node as usize] = true;
            for child in adj_list[node as usize].iter() {
                stack.push(*child);
                path.push(*child);
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

        let mut blocks_placed = 0;

        for (y, layer) in nodes
            .into_iter().rev().skip(self.start_layer).enumerate() {
            if layer.is_empty() { continue; }

            self.start_layer = y;
            self.save_progress();

            let path = nodes_to_mst_to_path(&layer);

            for node in path {
                let (coord, block) = layer[node as usize];
                self.nav.goto_nohead(&world_coord(&self.start_pos, coord, y), nav::Order::XYZ);
                self.turt.inv_select(((blocks_placed / 64) % 16) as u8);
                self.turt.p_down();
            }
        }
    }
}