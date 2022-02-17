use self::{
    middleware::{compress as compress_middleware, logger as logger_middleware},
    settings::Settings,
    sync::Synchroniser,
};
use openssl::ssl::{SslAcceptor, SslAcceptorBuilder, SslFiletype, SslMethod};

use graphql::{
    config as graphql_config,
    loader::{get_loaders, LoaderRegistry},
};
use log::{error, info, warn};
use repository::{get_storage_connection_manager, run_db_migrations};
use service::{auth_data::AuthData, service_provider::ServiceProvider, token_bucket::TokenBucket};

use actix_cors::Cors;
use actix_web::{web::Data, App, HttpServer};
use std::{net::TcpListener, sync::RwLock};
use tokio::sync::oneshot;

pub mod configuration;
pub mod environment;
pub mod middleware;
pub mod settings;
pub mod sync;
pub mod test_utils;

/// Starts the server
///
/// This method doesn't return until a message is send to the off_switch.
pub async fn start_server(
    settings: Settings,
    mut off_switch: oneshot::Receiver<()>,
) -> std::io::Result<()> {
    let auth_data = Data::new(AuthData {
        auth_token_secret: settings.auth.token_secret.to_owned(),
        token_bucket: RwLock::new(TokenBucket::new()),
        debug_no_ssl: false,
        // TODO: disable once frontend supports auth!
        debug_no_access_control: true,
    });

    let connection_manager = get_storage_connection_manager(&settings.database);

    info!("Run DB migrations...");
    match run_db_migrations(&connection_manager.connection().unwrap()) {
        Ok(_) => info!("DB migrations succeeded"),
        Err(err) => {
            let msg = format!("Failed to run DB migrations: {}", err);
            error!("{}", msg);
            panic!("{}", msg);
        }
    };

    let connection_manager_data_app = Data::new(connection_manager.clone());

    let service_provider = ServiceProvider::new(connection_manager.clone());
    let service_provider_data = Data::new(service_provider);

    let loaders = get_loaders(&connection_manager, service_provider_data.clone()).await;
    let loader_registry_data = Data::new(LoaderRegistry { loaders });

    let mut http_server = HttpServer::new(move || {
        App::new()
            .app_data(connection_manager_data_app.clone())
            .wrap(logger_middleware())
            .wrap(Cors::permissive())
            .wrap(compress_middleware())
            .configure(graphql_config(
                connection_manager_data_app.clone(),
                loader_registry_data.clone(),
                service_provider_data.clone(),
                auth_data.clone(),
            ))
    });
    match load_certs() {
        Ok(ssl_builder) => {
            http_server = http_server.bind_openssl(
                format!("{}:{}", settings.server.host, settings.server.port),
                ssl_builder,
            )?;
        }
        Err(err) => {
            error!("Failed to load certificates: {}", err);
            warn!("Run in HTTP mode");

            let listener = TcpListener::bind(settings.server.address())
                .expect("Failed to bind server to address");
            http_server = http_server.listen(listener)?;
        }
    }
    let mut running_sever = http_server.run();
    let mut synchroniser = Synchroniser::new(settings.sync, connection_manager).unwrap();
    // Do the initial pull before doing anything else
    match synchroniser.initial_pull().await {
        Ok(_) => {}
        // TODO: remove for production
        Err(err) => error!("Failed to perform the initial sync: {}", err),
    };

    // http_server is the only one that should quit; a proper shutdown signal can cause this,
    // and so we want an orderly exit. This achieves it nicely.
    let result = tokio::select! {
        result = (&mut running_sever) => result,
        _ = (&mut off_switch) => Ok(running_sever.stop(true).await),
        () = async {
            synchroniser.run().await;
        } => unreachable!("Synchroniser unexpectedly died!?"),
    };

    info!("Remote server stopped");
    result
}

fn load_certs() -> Result<SslAcceptorBuilder, anyhow::Error> {
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder.set_private_key_file("certs/key.pem", SslFiletype::PEM)?;
    builder.set_certificate_chain_file("certs/cert.pem")?;
    Ok(builder)
}
