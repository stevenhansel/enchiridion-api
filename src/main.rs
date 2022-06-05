use std::net::TcpListener;

use enchiridion_api::run;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind(format!("127.0.0.1:{}", 8080))
        .expect(format!("Failed to bind port {}", 8080).as_str());

    run(listener)?.await
}
