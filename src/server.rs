use crate::cmd;

use rocket::serde::json::Json;
use rocket::{get, post, routes};

#[get("/")]
async fn next() -> String {
    let n = cmd::COMMANDS.next();
    println!("next() -> `{n}`");
    return n;
}

#[post("/", format = "json", data = "<body>")]
async fn cmdcomplete(body: Json<cmd::LuaResp>) {
    let lua_resp = body.into_inner();
    let resp: cmd::Resp = lua_resp.into();
    println!("complete() -> `{:?}`", resp);
    cmd::COMMANDS.completed(resp);
}

pub async fn run() {
    rocket::build()
        .mount("/next", routes![next])
        .mount("/cmdcomplete", routes![cmdcomplete])
        .launch()
        .await
        .expect("Bye bye server...");
}

