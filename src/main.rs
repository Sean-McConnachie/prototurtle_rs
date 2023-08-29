use prototurtle_rs::{init_dirs, server};
use prototurtle_rs::scripts::model_builder::generation::other::load_obj_to_arr;

#[tokio::main]
pub async fn main() {

    load_obj_to_arr("assets/octo.obj");

    return;
    init_dirs();
    server::run().await;
}
