use std::io::Write;
use std::path;
use rand::SeedableRng;
use modelutils_rs::float;
use modelutils_rs::model2arr::{ArrayModel, Block, CoordXZ, int, uint};
use rand::prelude::StdRng;
use rand::Rng;
use crate::{turtle, nav, inventory};

const CHEST_INV: usize = 27;
const TURT_INV: usize = 16;
const SLOT_SIZE: usize = 64;

fn euclideanf_xz(a: &CoordXZ, b: &CoordXZ) -> float {
    let dx = a.0 as float - b.0 as float;
    let dy = a.1 as float - b.1 as float;
    (dx * dx + dy * dy).sqrt()
}

fn manhatten_turtle(a: CoordXZ, b: CoordXZ) -> uint {
    let dx = (a.0 as int - b.0 as int).abs();
    let dz = (a.1 as int - b.1 as int).abs();
    if dx == 0 || dz == 0 {
        return (dx + dz) as u16;
    }
    (dx + dz + 1) as u16
}

pub fn centroids_to_groupings(nodes: Vec<Vec<(CoordXZ, Block)>>, centroids: Vec<Centroid>) -> Vec<(Vec<Vec<(CoordXZ, Block)>>, usize)> {
    let mut groupings: Vec<(Vec<Vec<(CoordXZ, Block)>>, usize)> = (0..centroids.len())
        .map(|c| (vec![vec![]; nodes[c].len()], 0))
        .collect();

    for (li, layer) in nodes.iter().enumerate() {
        for (point, block) in layer {
            let mut min_distance = float::MAX;
            let mut closest_cluster = 0;

            for (i, ((x, z), _c)) in centroids.iter().enumerate() {
                let distance = euclideanf_xz(&point, &(*x as uint, *z as uint));
                if distance < min_distance {
                    min_distance = distance;
                    closest_cluster = i;
                }
            }

            groupings[closest_cluster].1 += 1;
            groupings[closest_cluster].0[li].push((*point, *block));
        }
    }

    groupings
}

pub type Centroid = (CoordXZ, usize);

pub fn k_means(model_arr: &Vec<Vec<(CoordXZ, Block)>>, dims: (uint, uint, uint), k: usize) -> Vec<Centroid> {
    // let mut rng = rand::thread_rng();
    let mut rng = StdRng::seed_from_u64(0);

    let mut centroids: Vec<Centroid> = (0..k)
        .map(|_| {
            let row_idx = rng.gen_range(0..dims.0);
            let col_idx = rng.gen_range(0..dims.2);
            let point = model_arr[row_idx as usize][col_idx as usize].0;
            (point, 0)
        })
        .collect();

    // K-means
    const MAX_ITER: usize = 100;
    for _i in 0..MAX_ITER {
        let mut centroid_sums: Vec<((usize, usize), usize)> = (0..k)
            .map(|_| ((0, 0), 0))
            .collect();

        for layer in model_arr {
            for (point, _block) in layer {
                let mut min_distance = float::MAX;
                let mut closest_cluster = 0;

                for (i, ((x, z), _c)) in centroids.iter().enumerate() {
                    let distance = euclideanf_xz(&point, &(*x as uint, *z as uint));
                    if distance < min_distance {
                        min_distance = distance;
                        closest_cluster = i;
                    }
                }

                centroid_sums[closest_cluster].0.0 += point.0 as usize;
                centroid_sums[closest_cluster].0.1 += point.1 as usize;
                centroid_sums[closest_cluster].1 += 1;
            }
        }

        // Update centroids
        for (i, ((x, z), count)) in centroid_sums.iter_mut().enumerate() {
            if *count == 0 { continue; }
            let centroid = &mut centroids[i];

            *centroid = (((*x / *count) as uint, (*z / *count) as uint), *count);
        }
    }

    centroids
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
            let cost: Cost = manhatten_turtle(node_a, node_b);
            edges.push((i as Node, j as Node, cost));
        }
    }
    edges.sort_by_key(|&(_, _, cost)| cost);

    let mut parents: Vec<Node> = (0..n).into_iter().map(|u| u as Node).collect();
    let mut adj_list: Vec<Vec<Node>> = (0..n).into_iter().map(|_| Vec::new()).collect();

    for (node_a, node_b, _cost) in edges {
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
    turt_i: usize,
    turt: &'a turtle::Turt<'a>,
    nav: &'a mut nav::Nav<'a>,
    inv: inventory::Inventory<'a>,

    // Build data must be kept in function parameter

    // Persistent data
    start_pos: nav::PosH,
    start_layer: usize,
    total_blocks: usize,
    stack_count: usize,
    fp: path::PathBuf,
}

impl<'a> MultiBuilder<'a> {
    pub fn new(
        turt_i: usize,
        start_pos: nav::PosH,
        turtle_id: usize,
        turt: &'a turtle::Turt<'a>,
        nav: &'a mut nav::Nav<'a>,
    ) -> Self {
        Self {
            turt_i,
            start_pos,
            turt,
            stack_count: 0,
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

    pub fn update_inv(&mut self, blocks_placed: usize) {
        if blocks_placed % 64 == 0 {
            let slot = (blocks_placed / 64) % 16;
            if slot == 0 {
                let mut chest_loc = self.start_pos.clone();
                chest_loc.x += self.turt_i as i64;

                self.stack_count += TURT_INV;
                let offset = ((self.stack_count as f32 / TURT_INV as f32) / CHEST_INV as f32) as i64;
                let max_chest_space = CHEST_INV - ((self.stack_count - TURT_INV) % CHEST_INV);

                chest_loc.z -= offset;
                self.nav.goto_head(&chest_loc, nav::Order::XYZ);

                for i in 0..TURT_INV {
                    self.turt.inv_select(i as u8);
                    self.turt.suck_down();
                }

                if max_chest_space < TURT_INV {
                    chest_loc.z -= 1;
                    self.nav.goto_head(&chest_loc, nav::Order::XYZ);

                    for i in max_chest_space..TURT_INV {
                        self.turt.inv_select(i as u8);
                        self.turt.suck_down();
                    }
                }
                self.inv.full_update();
                while !self.inv.is_full() {
                    std::thread::sleep(std::time::Duration::from_millis(10000));
                    println!("Refill turtle: {}", self.turt_i)
                }
            }
            self.turt.inv_select(slot as u8);
        }
    }

    pub fn run(&mut self, nodes: &Vec<Vec<(CoordXZ, Block)>>, count: usize) {
        let num_chests = (count as f32 / SLOT_SIZE as f32 / CHEST_INV as f32).ceil() as usize;
        let mut need_more_chests = match self.turt.inv_item_detail(0) {
            Some(chests) => {
                if chests.count() < num_chests as i32 {
                    true
                } else {
                    false
                }
            }
            None => true,
        };

        if !self.start_layer == 0 {
            need_more_chests = true;
        }

        if need_more_chests {
            println!("Not enough chests! Need at least: {}", num_chests);
            std::thread::sleep(std::time::Duration::from_millis(10000));
            // return;
        }

        for i in 0..num_chests {
            self.nav.goto_nohead(&nav::Pos::new(
                self.start_pos.x + self.turt_i as i64,
                self.start_pos.y,
                self.start_pos.z - i as i64,
            ), nav::Order::XYZ);
            self.turt.inv_select(0);
            self.turt.d_down();
            self.turt.p_down();
        }


        fn world_coord(start: &nav::PosH, coord: CoordXZ, y: usize) -> nav::Pos {
            nav::Pos {
                x: start.x + coord.0 as i64,
                y: start.y + y as i64,
                z: start.z + coord.1 as i64,
            }
        }

        let mut blocks_placed = 0;

        for y in (self.start_layer..nodes.len()).rev() {
            let layer = &nodes[y];
            // for (y, layer) in nodes.reverse()
            //     .into_iter().rev().skip(self.start_layer).enumerate() {
            if layer.is_empty() { continue; }

            self.start_layer = y;
            self.save_progress();

            let path = nodes_to_mst_to_path(&layer);

            for node in path {
                self.update_inv(blocks_placed);

                let (coord, _block) = layer[node as usize];
                self.nav.goto_nohead(&world_coord(&self.start_pos, coord, y), nav::Order::XYZ);
                self.turt.p_up();

                self.turt.p_up();
                blocks_placed += 1;
            }
        }
    }
}