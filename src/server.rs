use crate::turtle_core::data::{TurtRawResponse, TurtResponse};

use rocket::serde::json::Json;
use rocket::{get, post, routes, State};

use std::thread;

use std::sync::{mpsc, RwLock};
use crate::entry;

const TURTLE_CAPACITY: usize = 32;

pub type ChannelsClient = (mpsc::Sender<String>, mpsc::Receiver<TurtResponse>);

struct ChannelsServer {
    next_rx: mpsc::Receiver<String>,
    cmdcomplete_tx: mpsc::Sender<TurtResponse>,
}

unsafe impl Sync for ChannelsServer {}

impl ChannelsServer {
    fn new() -> (Self, ChannelsClient) {
        let (next_tx, next_rx): (mpsc::Sender<String>, mpsc::Receiver<String>) = mpsc::channel();
        let (cmdcomplete_tx, cmdcomplete_rx): (mpsc::Sender<TurtResponse>, mpsc::Receiver<TurtResponse>) =
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

impl Default for ChannelsServer {
    fn default() -> Self {
        Self::new().0
    }
}

struct BotNet {
    turtles: RwLock<Vec<ChannelsServer>>,
}

impl BotNet {
    fn new() -> Self {
        let mut turtles = Vec::with_capacity(TURTLE_CAPACITY);
        for _ in 0..TURTLE_CAPACITY {
            turtles.push(ChannelsServer::default());
        }
        Self {
            turtles: RwLock::new(turtles),
        }
    }

    fn register_turtle(&self, turtleid: usize) -> ChannelsClient {
        let mut turts = self.turtles.write().unwrap();
        if turtleid > turts.len() {
            for _ in turts.len()..turtleid + 1 {
                turts.push(ChannelsServer::default());
            }
        }
        let (web_server_channels, client_channels) = ChannelsServer::new();
        turts[turtleid] = web_server_channels;
        client_channels
    }
}

#[post("/<turtleid>")]
async fn register(bot_net: &State<BotNet>, turtleid: usize) {
    println!("Turtle {turtleid} is trying to register!");
    let channels_client = bot_net.register_turtle(turtleid);
    thread::spawn(move || {
        entry::turtle_registered(turtleid.to_string(), channels_client)
    });
}

#[get("/<turtleid>")]
async fn next(bot_net: &State<BotNet>, turtleid: usize) -> String {
    match bot_net.turtles.read().unwrap()[turtleid].next_rx.try_recv() {
        Ok(v) => v,
        Err(_) => "WAIT".to_string(),
    }
}

#[post("/<turtleid>", format = "json", data = "<body>")]
async fn cmdcomplete(bot_net: &State<BotNet>, turtleid: usize, body: Json<TurtRawResponse>) {
    let raw_resp = body.into_inner();
    let resp: TurtResponse = raw_resp.into();

    match bot_net.turtles.read().unwrap()[turtleid]
        .cmdcomplete_tx
        .send(resp)
    {
        Ok(_) => (),
        Err(_) => println!("Turtle {turtleid} has disconnected!"),
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
