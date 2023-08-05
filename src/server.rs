use crate::cmd;

use rocket::serde::json::Json;
use rocket::{get, post, routes};

#[post("/<turtleid>")]
async fn register(turtleid: usize) {
    cmd::COMMANDS[turtleid].commands.register();
}

#[get("/<turtleid>")]
async fn next(turtleid: usize) -> String {
    cmd::COMMANDS[turtleid].commands.next()
}

#[post("/<turtleid>", format = "json", data = "<body>")]
async fn cmdcomplete(turtleid: usize, body: Json<cmd::LuaResp>) {
    let lua_resp = body.into_inner();
    let resp: cmd::Resp = lua_resp.into();
    cmd::COMMANDS[turtleid].commands.completed(resp);
}

pub async fn run() {
    rocket::build()
        .mount("/register", routes![register])
        .mount("/next", routes![next])
        .mount("/cmdcomplete", routes![cmdcomplete])
        .launch()
        .await
        .expect("Bye bye server...");
}
