use crate::{cmd, nav, turtle};
use modelutils_rs::{float, model, model2arr, vec3::Vec3};

use std::sync::mpsc;
use modelutils_rs::model2arr::{ArrayModel, uint};
use crate::multi_builder::MultiBuilder;

const RESOLUTION: float = 100.0;
const SIZE: uint = 5;
const DIMS: (uint, uint, uint) = (SIZE, SIZE, SIZE);
const START_POS: nav::PosH = nav::PosH {
    x: -2490,
    y: 64,
    z: -1032,
    h: nav::Head::N,
};

pub type ClientChannels = (mpsc::Sender<String>, mpsc::Receiver<cmd::Resp>);

fn get_model(path: &str) -> (ArrayModel, (Vec3, Vec3)) {
    let box_scale = Vec3::new(
        DIMS.0 as float - 1.0,
        DIMS.1 as float - 1.0,
        DIMS.2 as float - 1.0,
    );

    let (models, _materials) = modelutils_rs::load_default(path).unwrap();
    let mut models = models
        .into_iter()
        .map(|m| model::Model::new(
            model::Points::from_flat_vec(m.mesh.positions),
            model::Faces::from_triangles(m.mesh.indices),
        ))
        .collect::<Vec<model::Model>>();
    for mut model in models.into_iter() {
        // Align model to origin
        let bounds = model.model_dims();
        model.mv(bounds.0 * Vec3::from_scalar(-1.0));
        // Scale model to fit in 10x10x10 cube
        let scale = model.scale_for_box(box_scale);
        model.scale(Vec3::from_scalar(scale.min_val()));

        let dims = model.model_dims();
        // Convert to array
        return (model2arr::model_2_arr(model, DIMS, RESOLUTION), dims);
    }
    panic!()
}

pub fn entry_point(turtleid: usize, chans: ClientChannels) {
    let turt = turtle::Turt::new(chans.0.clone(), &chans.1);
    let mut nav = nav::Nav::new(true, turtleid, &turt, chans.0.clone(), &chans.1);

    nav.gps_init();

    for i in 0..5 {
        nav.m_forw()
    }
    turt.disconnect();
    return;


    println!("Turtle: {turtleid}\tLOC {nav}");

    let (blocks, dims) = get_model("shapes/cube.obj");

    let nodes = MultiBuilder::get_nodes(blocks);
    let mut controller = MultiBuilder::new(START_POS, turtleid, &turt, &mut nav);
    controller.run(nodes);

    turt.disconnect();
}


// use crate::scripts::chunk_digger;
// use crate::{cmd, nav, turtle};
//
// use std::sync::mpsc;
//
// pub type ClientChanels = (mpsc::Sender<String>, mpsc::Receiver<cmd::Resp>);
//
// const ACTIVE_TURTLES: &[usize] = &[9, 10, 11, 12, 18, 19, 20, 21];
// const DEPTH_PT: i64 = 64; // 64
// const WIDTH_PT: i64 = 64 / 8; // 64 /8
// const HEIGHT_PT: i64 = 3 * 10; // *4
// const START_POS: nav::Pos = nav::Pos {
//     x: -1872,
//     y: 70,
//     z: -785,
// };
//
// pub fn entry_point(turtleid: usize, chans: ClientChanels) {
//     // Expands north-east
//     let rel_turt: i64 = {
//         let mut v = None;
//         for (i, t_id) in ACTIVE_TURTLES.iter().enumerate() {
//             if t_id == &turtleid {
//                 v = Some(i);
//                 break;
//             }
//         }
//         match v {
//             Some(v) => v as i64,
//             None => panic!("Rip."),
//         }
//     };
//
//     let (p1, p2) = (
//         nav::Pos {
//             x: START_POS.x + (rel_turt * WIDTH_PT),
//             y: START_POS.y,
//             z: START_POS.z,
//         },
//         nav::Pos {
//             x: START_POS.x + ((rel_turt + 1) * WIDTH_PT) - 1,
//             y: START_POS.y + HEIGHT_PT,
//             z: START_POS.z - DEPTH_PT - 1,
//         },
//     );
//
//     println!("Turtle {turtleid} is joining the bot net! Mining {p1} -> {p2}");
//
//     let turt = turtle::Turt::new(chans.0.clone(), &chans.1);
//     let mut nav = nav::Nav::new(turtleid, &turt, chans.0.clone(), &chans.1);
//
//     nav.gps_init();
//
//     println!("Turtle: {turtleid}\tLOC {nav}");
//
//     let mut d = chunk_digger::ChunkDigger::init(turtleid, &turt, &mut nav);
//     d.get_or_init_progress();
//     d.set_corners(p1, p2);
//     d.run();
//
//     turt.disconnect();
// }
