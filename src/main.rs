use prototurtle_rs::{init_dirs, server};

#[tokio::main]
pub async fn main() {
    init_dirs();
    server::run().await;
}
