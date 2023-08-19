use crate::scripts::{chunk_digger, silly_builder};
use crate::{cmd, inventory, nav, turtle};
use modelutils_rs::{float, model, model2arr, vec3::Vec3};

use std::sync::mpsc;
use rocket::log::private::warn;
use crate::single_builder::control::BuildController;

const RESOLUTION: float = 100.0;
const SIZE: usize = 50;
const DIMS: (usize, usize, usize) = (SIZE, SIZE, SIZE);

pub type ClientChannels = (mpsc::Sender<String>, mpsc::Receiver<cmd::Resp>);

fn get_model(path: &str) -> (Vec<Vec<Vec<bool>>>, (Vec3, Vec3)) {
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
        let scale = model.scale_for_box(Vec3::new(DIMS.0 as float, DIMS.1 as float, DIMS.2 as float));
        model.scale(Vec3::from_scalar(scale.min_val()));

        let dims = model.model_dims();
        // Convert to array
        return (model2arr::model_2_arr(model, DIMS, RESOLUTION), dims);
    }
    return (vec![], (Vec3::ZERO, Vec3::ZERO));
}

pub fn entry_point(turtleid: usize, chans: ClientChannels) {
    let turt = turtle::Turt::new(chans.0.clone(), &chans.1);
    let mut nav = nav::Nav::new(turtleid, &turt, chans.0.clone(), &chans.1);


    nav.gps_init();

    println!("Turtle: {turtleid}\tLOC {nav}");

    let (blocks, dims) = get_model("shapes/rotated_puppet.obj");

    let mut controller = BuildController::new(blocks, turtleid, &turt, &mut nav);
    controller.run();

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
