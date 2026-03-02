use super::handlers::*;
use crate::domains::assignments::{
    AssignmentServiceTrait,
    dto::assignment_dto::{AssignmentDto, CreateAssignmentDto, UpdateAssignmentDto},
};

use axum::routing::{delete, post, put};
use axum::{Router, extract::FromRef, routing::get};
use std::sync::Arc;
use utoipa::{
    OpenApi,
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
};

#[derive(OpenApi)]
#[openapi(
    paths(
        get_assignment_by_id,
        get_assignments,
        create_assignment,
        update_assignment,
        delete_assignment,
    ),
    components(schemas(AssignmentDto, CreateAssignmentDto, UpdateAssignmentDto)),
    tags(
        (name = "Assignments", description = "Assignment management endpoints")
    ),
    security(
        ("bearer_auth" = [])
    ),
    modifiers(&AssignmentsApiDoc)
)]
pub struct AssignmentsApiDoc;

impl utoipa::Modify for AssignmentsApiDoc {
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

pub fn assignment_routes<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    Arc<dyn AssignmentServiceTrait>: FromRef<S>,
{
    Router::new()
        .route("/{id}", get(get_assignment_by_id))
        .route("/", get(get_assignments))
        .route("/", post(create_assignment))
        .route("/{id}", put(update_assignment))
        .route("/{id}", delete(delete_assignment))
}
