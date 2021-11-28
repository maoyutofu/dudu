use dudu::rest;
use std::io;

#[actix_web::main]
async fn main() -> io::Result<()> {
    std::env::set_var("RUST_LOG", "actix=info,dudu=info");
    env_logger::init();
    rest::Server::new().start().await
}
