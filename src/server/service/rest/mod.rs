use crate::server::data::ActorRegistry;

use actix_web::{
    web::{get, post, scope, Data, ServiceConfig},
    HttpRequest, HttpResponse, Result,
};

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/")
            .route("/health_check", get().to(health_check))
            .route("/sync", post().to(sync)),
    );
}

async fn health_check(_req: HttpRequest) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().finish())
}

async fn sync(actor_registry: Data<ActorRegistry>) -> Result<HttpResponse> {
    let sync_sender = &actor_registry.sync_sender;
    sync_sender.lock().unwrap().send();
    Ok(HttpResponse::Ok().body(""))
}
