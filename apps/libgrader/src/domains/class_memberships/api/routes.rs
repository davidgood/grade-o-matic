use super::handlers::*;
use crate::domains::class_memberships::{
    ClassMembershipServiceTrait,
    dto::class_membership_dto::{
        ClassMembershipDto, CreateClassMembershipDto, UpdateClassMembershipDto,
    },
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
        get_class_memberships_by_class_id,
        get_class_memberships_by_user_id,
        get_class_membership_by_id,
        create_class_membership,
        update_class_membership,
        delete_class_membership,
    ),
    components(schemas(
        ClassMembershipDto,
        CreateClassMembershipDto,
        UpdateClassMembershipDto
    )),
    tags(
        (name = "ClassMemberships", description = "Class membership management endpoints")
    ),
    security(
        ("bearer_auth" = [])
    ),
    modifiers(&ClassMembershipsApiDoc)
)]
pub struct ClassMembershipsApiDoc;

impl utoipa::Modify for ClassMembershipsApiDoc {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let Some(components) = openapi.components.as_mut() else {
            return;
        };

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

pub fn class_membership_routes<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    Arc<dyn ClassMembershipServiceTrait>: FromRef<S>,
{
    Router::new()
        .route("/class/{class_id}", get(get_class_memberships_by_class_id))
        .route("/user/{user_id}", get(get_class_memberships_by_user_id))
        .route("/{id}", get(get_class_membership_by_id))
        .route("/", post(create_class_membership))
        .route("/", put(update_class_membership))
        .route("/{id}", delete(delete_class_membership))
}
