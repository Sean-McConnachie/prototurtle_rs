use crate::scripts::chunk_digger;
use crate::{cmd, nav, turtle};

use std::sync::mpsc;

pub type ClientChanels = (mpsc::Sender<String>, mpsc::Receiver<cmd::Resp>);

const ACTIVE_TURTLES: &[usize] = &[9, 10, 11, 12];
const DEPTH_PT: i64 = 8 * 16;
const WIDTH_PT: i64 = 1 * 16;
const HEIGHT_PT: i64 = 6;
const START_POS: nav::Pos = nav::Pos {
    x: -27 - (4 * 16),
    y: 8,
    z: 31,
};

pub fn entry_point(turtleid: usize, chans: ClientChanels) {
    // Expands north-east
    let rel_turt: i64 = {
        let mut v = None;
        for (i, t_id) in ACTIVE_TURTLES.iter().enumerate() {
            if t_id == &turtleid {
                v = Some(i);
                break;
            }
        }
        match v {
            Some(v) => v as i64,
            None => panic!("Rip."),
        }
    };

    let (p1, p2) = (
        nav::Pos {
            x: START_POS.x + (rel_turt * WIDTH_PT),
            y: START_POS.y,
            z: START_POS.z,
        },
        nav::Pos {
            x: START_POS.x + ((rel_turt + 1) * WIDTH_PT) - 1,
            y: START_POS.y + HEIGHT_PT,
            z: START_POS.z - DEPTH_PT - 1,
        },
    );

    println!("Turtle {turtleid} is joining the bot net! Mining {p1} -> {p2}");

    let turt = turtle::Turt::new(chans.0.clone(), &chans.1);
    let mut nav = nav::Nav::new(turtleid, &turt, chans.0.clone(), &chans.1);

    nav.gps_init();

    println!("Turtle: {turtleid}\tLOC {nav}");

    let mut d = chunk_digger::ChunkDigger::init(turtleid, &turt, &mut nav);
    d.get_or_init_progress();
    d.set_corners(p1, p2);
    d.run();

    turt.disconnect();
}
