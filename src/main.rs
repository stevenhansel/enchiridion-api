use std::net::TcpListener;

use secrecy::ExposeSecret;
use sqlx::PgPool;

use enchiridion_api::auth::{AuthService, AuthServiceParameters};
use enchiridion_api::config::Configuration;
use enchiridion_api::container::Container;
use enchiridion_api::email;
use enchiridion_api::role::{RoleRepository, RoleRepositoryParameters};
use enchiridion_api::startup::run;
use enchiridion_api::user::{UserRepository, UserRepositoryParameters};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let config = Configuration::with_env_file().expect("Failed to read configuration");

    let pool = PgPool::connect(config.database_url.expose_secret())
        .await
        .unwrap();

    let mailgun_adapter = email::MailgunAdapter::new(
        config.mailgun_baseurl.clone(),
        config.mailgun_domain.clone(),
        config.mailgun_api_key.clone(),
    );
    let email_client = email::Client::new(Box::new(mailgun_adapter));

    let container = Container::builder()
        .with_component_parameters::<UserRepository>(UserRepositoryParameters { _db: pool.clone() })
        .with_component_parameters::<RoleRepository>(RoleRepositoryParameters { _db: pool.clone() })
        .with_component_parameters::<AuthService>(AuthServiceParameters {
            _configuration: config.clone(),
            _email: email_client,
        })
        .build();

    let listener = TcpListener::bind(config.address)?;
    run(listener, container)?.await
}
