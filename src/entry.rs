use crate::scripts::chunk_digger;
use crate::{cmd, nav, turtle};

use std::sync::mpsc;

pub type ClientChanels = (mpsc::Sender<String>, mpsc::Receiver<cmd::Resp>);

pub fn entry_point(turtleid: usize, chans: ClientChanels) {
    println!("Turtle {turtleid} is joining the bot net!");

    let turt = turtle::Turt::new(chans.0.clone(), &chans.1);
    let mut nav = nav::Nav::new(turtleid, &turt, chans.0.clone(), &chans.1);

    nav.gps_init();

    println!("Turtle: {turtleid}\tLOC {nav}");

    let mut d = chunk_digger::ChunkDigger::init(turtleid, &turt, &mut nav);
    //d.get_or_init_progress();
    d.set_corners(
        nav::Pos {
            x: -631,
            y: -39,
            z: 437,
        },
        nav::Pos {
            x: -633,
            y: -45,
            z: 435,
        },
    );
    d.run();

    turt.disconnect();
}
