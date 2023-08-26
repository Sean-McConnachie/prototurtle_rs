use modelutils_rs::model2arr::{Block, CoordXZ};
use prototurtle_rs::scripts::model_builder::generation;
use prototurtle_rs::server;

#[tokio::main]
pub async fn main() {
    example_generation();
    // server::run().await;
}

fn example_generation() {
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

    let mst = generation::nodes_to_mst(&nodes);
    println!("{:?}", &mst);

    let paths = generation::mst_to_paths(mst);
    println!("{:?}", &paths);

    let joined = generation::join_paths_greedily((0, 0), paths, &nodes);
    println!("{:?}", &joined);

    let nodes = vec![nodes];

    let centroids = generation::k_means(&nodes, (3, 1, 3), 3);
    println!("{:?}", &centroids);

    let groupings = generation::centroids_to_groupings(nodes, centroids, (3, 1, 3));
    println!("{:?}", &groupings);
}