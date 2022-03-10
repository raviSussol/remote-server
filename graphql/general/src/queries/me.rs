use async_graphql::*;
use graphql_core::{
    standard_graphql_error::{validate_auth, StandardGraphqlError},
    ContextExt,
};

use graphql_types::types::UserNode;
use service::permission_validation::{Resource, ResourceAccessRequest};
use service::user_account::UserAccountService;

#[derive(Union)]
pub enum UserResponse {
    Response(UserNode),
}

pub fn me(ctx: &Context<'_>) -> Result<UserResponse> {
    let user = validate_auth(
        ctx,
        &ResourceAccessRequest {
            resource: Resource::RouteMe,
            store_id: None,
        },
    )?;

    let service_provider = ctx.service_provider();
    let service_ctx = service_provider.context()?;
    let user_service = UserAccountService::new(&service_ctx.connection);
    let user = match user_service.find_user(&user.user_id) {
        Ok(Some(user)) => user,
        Ok(None) => {
            return Err(StandardGraphqlError::InternalError(
                "Can't find user account data".to_string(),
            )
            .extend());
        }
        Err(err) => return Err(err.into()),
    };

    Ok(UserResponse::Response(UserNode::from_domain(user)))
}
