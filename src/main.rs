use std::net::TcpListener;

use sqlx::PgPool;
use secrecy::ExposeSecret;

use enchiridion_api::startup::run;
use enchiridion_api::config::Configuration;
use enchiridion_api::container::Container;
use enchiridion_api::user::{UserRepository, UserRepositoryParameters};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let config = Configuration::for_development().expect("Failed to read configuration");

    let pool = PgPool::connect(config.database_url.expose_secret())
        .await
        .unwrap();

    let container = Container::builder()
        .with_component_parameters::<UserRepository>(UserRepositoryParameters { _db: pool })
        .build();

    let listener = TcpListener::bind(config.address)?;
    run(listener, container)?.await
}
