//! src/main.rs

mod database;
mod server;
mod utils;

use env_logger::Env;
use std::{io, net};

#[actix_web::main]
async fn main() -> io::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let configuration = utils::get_configuration().expect("Failed to parse configuration settings");

    let listener = net::TcpListener::bind(configuration.server.address())
        .expect("Failed to bind server to address");

    let pool = sqlx::PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to omsupply-database");

    let database = database::DatabaseConnection::new(pool).await;

    // TODO: replace mock data with tests
    database
        .insert_mock_data()
        .await
        .expect("Failed to insert mock data");

    server::run(listener, database)?.await
}
