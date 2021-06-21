use crate::database::{DataLoader, DatabaseConnection};
use crate::server::graphql::schema::Schema;

pub async fn graphql(
    req: actix_web::web::Json<juniper::http::GraphQLRequest>,
    schema: actix_web::web::Data<Schema>,
    database: actix_web::web::Data<DatabaseConnection>,
) -> Result<actix_web::HttpResponse, actix_web::Error> {
    // Instantiate context per request to avoid cached/bached data leaking between requests.
    let context = DataLoader::new(database.get_ref().clone());

    let response = req.execute(&schema, &context).await;
    let json = serde_json::to_string(&response)?;
    Ok(actix_web::HttpResponse::Ok()
        .content_type("application/json")
        .body(json))
}
