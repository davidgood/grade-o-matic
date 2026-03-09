use super::handlers::*;
use crate::domains::classes::{
    ClassServiceTrait,
    dto::class_dto::{ClassDto, CreateClassDto, UpdateClassDto},
};

use axum::Router;
use axum::extract::FromRef;
use axum::routing::{delete, get, post, put};
use std::sync::Arc;
use utoipa::{
    OpenApi,
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
};

#[derive(OpenApi)]
#[openapi(
    paths(
        get_classes,
        get_class_by_id,
        create_class,
        update_class,
        delete_class,
    ),
    components(schemas(ClassDto, CreateClassDto, UpdateClassDto)),
    tags(
        (name = "Classes", description = "Class management endpoints")
    ),
    security(
        ("bearer_auth" = [])
    ),
    modifiers(&ClassesApiDoc)
)]
pub struct ClassesApiDoc;
impl utoipa::Modify for ClassesApiDoc {
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

pub fn class_routes<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    Arc<dyn ClassServiceTrait>: FromRef<S>,
{
    Router::new()
        .route("/", get(get_classes))
        .route("/{id}", get(get_class_by_id))
        .route("/", post(create_class))
        .route("/", put(update_class))
        .route("/{id}", delete(delete_class))
}
