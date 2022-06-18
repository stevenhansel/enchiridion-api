use std::net::TcpListener;

use enchiridion_api::run;
use sqlx::PgPool;

mod container;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let pool = PgPool::connect("postgres://postgres:postgres@localhost/enchiridion")
        .await
        .unwrap();

    let listener = TcpListener::bind(format!("127.0.0.1:{}", 8080))
        .expect(format!("Failed to bind port {}", 8080).as_str());

    run(listener, pool)?.await
}
