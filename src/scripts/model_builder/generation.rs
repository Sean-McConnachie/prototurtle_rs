use rand::Rng;
use rand::SeedableRng;
use modelutils_rs::float;
use rand::prelude::StdRng;
use modelutils_rs::model2arr::{ArrayModel, Block, CoordXZ, int, uint};

pub type Centroid = (CoordXZ, usize);
type Node = uint;
type Cost = uint;
type Edge = (Node, Node, Cost);

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

pub fn array_model_to_nodes(array_model: ArrayModel) -> Vec<Vec<(CoordXZ, Block)>> {
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

pub fn centroids_to_groupings(
    nodes: Vec<Vec<(CoordXZ, Block)>>,
    centroids: Vec<Centroid>,
    dims: (uint, uint, uint),
) -> Vec<(Vec<Vec<(CoordXZ, Block)>>, usize)> {
    let mut groupings: Vec<(Vec<Vec<(CoordXZ, Block)>>, usize)> = (0..centroids.len())
        .map(|_| (vec![vec![]; dims.1 as usize], 0))
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

pub fn k_means(model_arr: &Vec<Vec<(CoordXZ, Block)>>, dims: (uint, uint, uint), k: usize) -> Vec<Centroid> {
    // let mut rng = rand::thread_rng();
    let mut rng = StdRng::seed_from_u64(0);

    let mut centroids: Vec<Centroid> = (0..k)
        .map(|_| {
            let ind = rng.gen_range(0..model_arr[0].len());
            let point = model_arr[0][ind].0;
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

pub fn nodes_to_mst(nodes: &Vec<(CoordXZ, Block)>) -> Vec<Vec<Node>> {
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
    adj_list
}

pub fn mst_to_paths(adj_list: Vec<Vec<Node>>) -> Vec<Vec<uint>> {
    let n = adj_list.len();

    // Push node with highest degree
    let mut max_degree = 0;
    let mut max_degree_node = 0;
    for (i, adj) in adj_list.iter().enumerate() {
        if adj.len() > max_degree {
            max_degree = adj.len();
            max_degree_node = i;
        }
    }

    type Seen = uint;
    type Visited = bool;

    let mut stack: Vec<Node> = Vec::new();
    let mut time = 0;
    let mut node_data: Vec<(Seen, Visited)> = (0..n)
        .into_iter().map(|_| (0 as Seen, false as Visited)).collect();

    stack.push(max_degree_node as Node);
    // node_data[max_degree_node] = (time, false);
    time += 1;

    // Perform DFS
    while !stack.is_empty() {
        let node = stack.pop().unwrap();
        let node = node as usize;

        if !node_data[node].1 {
            node_data[node].1 = true;
            node_data[node].0 = time;
            time += 1;
            for child in adj_list[node].iter() {
                stack.push(*child);
            }
            if adj_list[node].is_empty() {
                time += 1;
            }
        }
    }

    // Sort nodes by seen time
    let mut nodes: Vec<Node> = (0..n).into_iter().map(|u| u as Node).collect();
    nodes.sort_by_key(|&node| node_data[node as usize].0);

    // Create multiple paths based on seen time
    let mut paths: Vec<Vec<Node>> = vec![];
    let mut curr_path = vec![max_degree_node as Node];
    let mut prev_time = node_data[nodes[0] as usize].0;
    for i in 1..n {
        let curr_time = node_data[nodes[i] as usize].0;
        if curr_time - 1 != prev_time {
            paths.push(curr_path);
            curr_path = vec![];
        }
        curr_path.push(nodes[i]);
        prev_time = curr_time;
    }
    if !curr_path.is_empty() {
        paths.push(curr_path);
    }

    paths
}

pub fn join_paths_greedily(
    mut start_point: CoordXZ,
    mut paths: Vec<Vec<Node>>,
    nodes: &Vec<(CoordXZ, Block)>,
) -> Vec<Node> {
    let mut final_path = vec![0; nodes.len()];

    let mut min_distance = uint::MAX;
    let mut curr_path = 0;
    let mut reverse = false;

    let mut c = 0;
    while !paths.is_empty() {
        min_distance = uint::MAX;

        for (i, path) in paths.iter().enumerate() {
            let front_dist = manhatten_turtle(start_point, nodes[path[0] as usize].0);
            let back_dist = manhatten_turtle(start_point, nodes[*path.last().unwrap() as usize].0);
            if front_dist < min_distance {
                min_distance = front_dist;
                curr_path = i;
                reverse = false;
            }
            if back_dist < min_distance {
                min_distance = back_dist;
                curr_path = i;
                reverse = true;
            }
        }

        let path = &paths[curr_path];

        if reverse {
            path.iter().rev().for_each(|node| {
                final_path[c] = *node;
                c += 1;
            });
            start_point = nodes[path[0] as usize].0;
        } else {
            path.iter().for_each(|node| {
                final_path[c] = *node;
                c += 1;
            });
            start_point = nodes[*path.last().unwrap() as usize].0;
        }

        paths.remove(curr_path);
    }

    final_path
}

pub fn example_generation() {
    let nodes: Vec<(CoordXZ, Block)> = vec![
        ((0, 0), 0),
        ((0, 1), 0),
        ((0, 2), 0),
        ((1, 0), 0),
        ((1, 1), 0),
        ((1, 2), 0),
        ((2, 0), 0),
        ((2, 1), 0),
        ((2, 2), 0),
    ];

    let mst = nodes_to_mst(&nodes);
    println!("{:?}", &mst);

    let paths = mst_to_paths(mst);
    println!("{:?}", &paths);

    let joined = join_paths_greedily((0, 0), paths, &nodes);
    println!("{:?}", &joined);
}