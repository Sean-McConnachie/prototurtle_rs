use crate::{cmd, turtle, nav};

use std::sync::mpsc;

pub type ClientChanels = (mpsc::Sender<String>, mpsc::Receiver<cmd::Resp>);

pub fn entry_point(turtleid: usize, chans: ClientChanels) {
    println!("Turtle {turtleid} is joining the bot net!");
    let turt = turtle::Turt::new(chans.0.clone(), &chans.1);
    let mut nav = nav::Nav::new(turtleid, &turt, chans.0.clone(), &chans.1);

    nav.gps_init();

    println!("{nav}");

    turt.disconnect();
}
