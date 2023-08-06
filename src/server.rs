use crate::{cmd, entry};

use rocket::serde::json::Json;
use rocket::{get, post, routes, State};

use std::thread;

use std::sync::{mpsc, RwLock};

struct WebServerChannels {
    next_rx: mpsc::Receiver<String>,
    cmdcomplete_tx: mpsc::Sender<cmd::Resp>,
}

unsafe impl Sync for WebServerChannels {}

impl WebServerChannels {
    fn new() -> (Self, entry::ClientChanels) {
        let (next_tx, next_rx): (mpsc::Sender<String>, mpsc::Receiver<String>) = mpsc::channel();
        let (cmdcomplete_tx, cmdcomplete_rx): (mpsc::Sender<cmd::Resp>, mpsc::Receiver<cmd::Resp>) =
            mpsc::channel();

        (
            Self {
                next_rx,
                cmdcomplete_tx,
            },
            (next_tx, cmdcomplete_rx),
        )
    }
}

impl Default for WebServerChannels {
    fn default() -> Self {
        Self::new().0
    }
}

struct BotNet {
    turtles: RwLock<Vec<WebServerChannels>>,
}

impl BotNet {
    fn new() -> Self {
        Self {
            turtles: RwLock::new(vec![]),
        }
    }

    fn register_turtle(&self, turtleid: usize) -> entry::ClientChanels {
        let mut turts = self.turtles.write().unwrap();
        if turtleid > turts.len() {
            for _ in turts.len()..turtleid + 1 {
                turts.push(WebServerChannels::default());
            }
        }
        let (web_server_channels, client_channels) = WebServerChannels::new();
        turts[turtleid] = web_server_channels;
        client_channels
    }
}

#[post("/<turtleid>")]
async fn register(bot_net: &State<BotNet>, turtleid: usize) {
    println!("Turtle {turtleid} is trying to register!");
    let client_channels = bot_net.register_turtle(turtleid);
    thread::spawn(move || entry::entry_point(turtleid, client_channels));
}

#[get("/<turtleid>")]
async fn next(bot_net: &State<BotNet>, turtleid: usize) -> String {
    match bot_net.turtles.read().unwrap()[turtleid].next_rx.try_recv() {
        Ok(v) => v,
        Err(_) => "WAIT".to_string(),
    }
}

#[post("/<turtleid>", format = "json", data = "<body>")]
async fn cmdcomplete(bot_net: &State<BotNet>, turtleid: usize, body: Json<cmd::LuaResp>) {
    let lua_resp = body.into_inner();
    let resp: cmd::Resp = lua_resp.into();

    match bot_net.turtles.read().unwrap()[turtleid] 
        .cmdcomplete_tx
        .send(resp) {
            Ok(_) => (),
            Err(_) => println!("Turtle {turtleid} has disconnected!")
    };
}

pub async fn run() {
    let bot_net: BotNet = BotNet::new();
    rocket::build()
        .mount("/register", routes![register])
        .mount("/next", routes![next])
        .mount("/cmdcomplete", routes![cmdcomplete])
        .manage(bot_net)
        .launch()
        .await
        .expect("Bye bye server...");
}
