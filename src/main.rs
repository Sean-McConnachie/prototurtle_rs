use prototurtle_rs::server;

#[tokio::main]
pub async fn main() {
    server::run().await;
}
