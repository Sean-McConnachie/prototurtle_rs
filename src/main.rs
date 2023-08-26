use prototurtle_rs::scripts::model_builder::generation::other::{example_generation, groupings_to_arr, load_obj_to_arr};
use prototurtle_rs::server;

#[tokio::main]
pub async fn main() {
    load_obj_to_arr("assets/octo.obj");
    // server::run().await;
}
