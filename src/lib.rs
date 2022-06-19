#![allow(dead_code)]
use std::net::TcpListener;

use actix_web::dev::Server;
use sqlx::{Pool, Postgres};

// Mod declarations
mod auth;
mod container;
mod http;
mod user;

use http::http as internal_http;

pub fn run(listener: TcpListener, pool: Pool<Postgres>) -> Result<Server, std::io::Error> {
    internal_http::serve(listener, pool)
}
