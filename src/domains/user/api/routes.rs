use super::handlers::*;
use crate::domains::user::{
    UserAssetPattern, UserServiceTrait,
    dto::user_dto::{CreateUserMultipartDto, SearchUserDto, UpdateUserDto, UserDto},
};

use axum::{
    Router,
    extract::FromRef,
    routing::{delete, get, post, put},
};
use std::sync::Arc;

use utoipa::{
    OpenApi,
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
};

#[derive(OpenApi)]
#[openapi(
    paths(
        get_user_by_id,
        get_users,
        get_user_list,
        create_user,
        update_user,
        delete_user,
    ),
    components(schemas(UserDto, SearchUserDto, CreateUserMultipartDto, UpdateUserDto)),
    tags(
        (name = "Users", description = "User management endpoints")
    ),
    security(
        ("bearer_auth" = [])
    ),
    modifiers(&UserApiDoc)
)]
/// This struct is used to generate OpenAPI documentation for the user routes.
pub struct UserApiDoc;

impl utoipa::Modify for UserApiDoc {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.as_mut().unwrap();
        components.add_security_scheme(
            "bearer_auth",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .description(Some("Input your `<your‑jwt>`"))
                    .build(),
            ),
        )
    }
}

pub fn user_routes<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    Arc<dyn UserServiceTrait>: FromRef<S>,
    UserAssetPattern: FromRef<S>,
{
    Router::new()
        .route("/", get(get_users))
        .route("/", post(create_user))
        .route("/list", post(get_user_list))
        .route("/{id}", get(get_user_by_id))
        .route("/{id}", put(update_user))
        .route("/{id}", delete(delete_user))
}
