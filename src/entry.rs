use modelutils_rs::{DEG2RAD, float};
use modelutils_rs::coords::Order;
use modelutils_rs::model2arr::{Block, CoordXZ, model_2_arr, uint};
use modelutils_rs::model::{Faces, Model, Points};
use modelutils_rs::vec3::Vec3;
use crate::scripts::chunk_digger::{ChunkDigger, ChunkDiggerConfig};
use crate::scripts::model_builder::generation::{array_model_to_nodes, centroids_to_groupings, k_means};
use crate::scripts::model_builder::runtime::{ModelBuilder, ModelBuilderConfig};
use crate::server::ChannelsClient;
use crate::turtle_core::control::TurtControl;
use crate::turtle_core::inventory::TurtBlock;
use crate::turtle_core::navigation::{Head, Pos, PosH, TurtNavigation};
use crate::{TurtleIdentifier, TurtleIndex};

// const PATH: &str = "assets/octo.obj";
// const ACTIVE_TURTLES: &[usize] = &[
//     4,
//     9,
//     10,
//     11,
//     12,
//     19,
//     20,
//     21
// ];
// const START_POS: PosH = PosH {
//     x: -1524,
//     y: 63,
//     z: -459,
//     h: Head::N,
// };
// const DEPTH: i64 = 64;
// const WIDTH: i64 = 64 / 8;
// const HEIGHT: i64 = 108 - 63;
//
// fn id_to_i(turtleid: usize) -> usize {
//     let mut v = None;
//     for (i, t_id) in ACTIVE_TURTLES.iter().enumerate() {
//         if t_id == &turtleid {
//             v = Some(i);
//             break;
//         }
//     }
//     match v {
//         Some(v) => v,
//         None => panic!("Rip."),
//     }
// }
//
//
// pub fn turtle_registered(identifier: TurtleIdentifier, channels_client: ChannelsClient) {
//     let turt = TurtControl::new(
//         channels_client.0.clone(),
//         &channels_client.1);
//     let mut nav = TurtNavigation::new(
//         identifier,
//         &turt,
//         true,
//         channels_client.0.clone(),
//         &channels_client.1);
//
//     let ind = id_to_i(identifier) as i64;
//
//     println!("[pre] Turtle {} registered! {}", identifier, nav);
//
//     nav.gps_init();
//     println!("Turtle {} registered! {}", identifier, nav);
//
//     let conf = ChunkDiggerConfig {
//         p1: Pos::new(START_POS.x + ind * WIDTH, START_POS.y, START_POS.z - DEPTH),
//         p2: Pos::new(START_POS.x + (ind + 1) * WIDTH, START_POS.y + HEIGHT, START_POS.z - DEPTH),
//         place_floor: TurtBlock::None,
//         chest_size: 54, // iron chest
//         check_inv_every_n_blocks: 64,
//     };
//     let mut digger = ChunkDigger::init(
//         (
//             identifier,
//             ind as TurtleIndex,
//             &turt,
//             &mut nav
//         ), conf);
//     digger.run();
//
//     turt.disconnect();
// }
//


const PATH: &str = "assets/octo.obj";
const ACTIVE_TURTLES: &[usize] = &[
    4,
    9,
    10,
    11,
    12,
    19,
    20,
    21
];
const SIZE: uint = 200;
const START_POS: PosH = PosH {
    x: -2320,
    y: 60,
    z: -1072,
    h: Head::N,
};


const RESOLUTION: float = 100.0;
const DIMS: (uint, uint, uint) = (SIZE, SIZE, SIZE);

fn id_to_i(turtleid: usize) -> usize {
    let mut v = None;
    for (i, t_id) in ACTIVE_TURTLES.iter().enumerate() {
        if t_id == &turtleid {
            v = Some(i);
            break;
        }
    }
    match v {
        Some(v) => v,
        None => panic!("Rip."),
    }
}

fn get_model(path: &str) -> Vec<(Vec<Vec<(CoordXZ, Block)>>, usize)> {
    let box_scale = Vec3::new(
        DIMS.0 as float - 1.0,
        DIMS.1 as float - 1.0,
        DIMS.2 as float - 1.0,
    );

    let (models, _materials) = modelutils_rs::load_default(path).unwrap();
    let models = models
        .into_iter()
        .map(|m| Model::new(
            Points::from_flat_vec(m.mesh.positions),
            Faces::from_triangles(m.mesh.indices),
        ))
        .collect::<Vec<Model>>();
    for mut model in models.into_iter() {
        model.rotate(Vec3::new(0.0, 225.0 * DEG2RAD, 0.0), Order::XYZ);
        // Align model to origin
        let bounds = model.model_dims();
        model.mv(bounds.0 * Vec3::from_scalar(-1.0));
        // Scale model to fit in 10x10x10 cube
        let scale = model.scale_for_box(box_scale);
        model.scale(Vec3::from_scalar(scale.min_val()));

        let dims = model.model_dims();
        // Convert to array
        let arr = model_2_arr(model, DIMS, RESOLUTION);
        let nodes = array_model_to_nodes(arr);
        let centroids = k_means(&nodes, DIMS, ACTIVE_TURTLES.len());
        let groupings = centroids_to_groupings(nodes, centroids, DIMS);

        return groupings;
    }
    panic!()
}


pub fn turtle_registered(identifier: TurtleIdentifier, channels_client: ChannelsClient) {
    let turt = TurtControl::new(
        channels_client.0.clone(),
        &channels_client.1);
    let mut nav = TurtNavigation::new(
        identifier,
        &turt,
        true,
        channels_client.0.clone(),
        &channels_client.1);

    let ind = id_to_i(identifier);


    nav.gps_init();
    println!("Turtle {} registered! {}", identifier, nav);

    let mut model_builder = ModelBuilder::new(
        (
            identifier,
            ind,
            &turt,
            &mut nav
        ), ModelBuilderConfig {
            start_pos: Pos::new(START_POS.x, START_POS.y, START_POS.z),
            max_chests: 7,
            allowed_blocks: vec![
                "minecraft:yellow_terracotta".to_string(),
                "minecraft:pink_terracotta".to_string(),
                "minecraft:orange_terracotta".to_string(),
                "minecraft:light_blue_terracotta".to_string(),
                "minecraft:lime_terracotta".to_string(),
                "minecraft:blue_terracotta".to_string(),
                "minecraft:white_terracotta".to_string(),
                "minecraft:magenta_terracotta".to_string(),
                "minecraft:terracotta".to_string(),
                "minecraft:purple_terracotta".to_string(),
                "minecraft:black_terracotta".to_string(),
                "minecraft:green_terracotta".to_string(),
                "minecraft:gray_terracotta".to_string(),
                "minecraft:brown_terracotta".to_string(),
                "minecraft:cyan_terracotta".to_string(),
                "minecraft:red_terracotta".to_string(),
                "minecraft:light_gray_terracotta".to_string(),
            ],
        });

    let groupings = get_model(PATH);
    let groupings = &groupings[ind];
    model_builder.run(&groupings.0, groupings.1);

    turt.disconnect();
}
