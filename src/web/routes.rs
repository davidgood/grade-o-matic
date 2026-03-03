use axum::{
    Router,
    extract::FromRef,
    middleware,
    routing::{get, post},
};
use std::sync::Arc;

use crate::common::jwt;
use crate::domains::assignments::AssignmentServiceTrait;
use crate::domains::auth::AuthServiceTrait;
use crate::domains::classes::ClassServiceTrait;

use super::{
    assignments::assignments_page,
    handlers::{
        admin_create_user_submit, admin_users_page, login_page, login_submit, logout,
        server_time_fragment, ui_index,
    },
    htmx::assignments::assignments_table_fragment,
    instructors::{
        create_class_page, create_class_submit, edit_class_page, edit_class_submit,
        instructor_class_detail_page, instructors_page,
    },
};
use crate::domains::user::UserServiceTrait;

pub fn web_routes<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    Arc<dyn AuthServiceTrait>: FromRef<S>,
    Arc<dyn AssignmentServiceTrait>: FromRef<S>,
    Arc<dyn ClassServiceTrait>: FromRef<S>,
    Arc<dyn UserServiceTrait>: FromRef<S>,
{
    let protected_ui_routes = Router::new()
        .route("/", get(ui_index))
        .route("/ui", get(ui_index))
        .route("/ui/assignments", get(assignments_page))
        .route("/ui/fragments/server-time", get(server_time_fragment))
        .route(
            "/ui/fragments/assignments/table",
            get(assignments_table_fragment),
        )
        .route("/ui/instructors", get(instructors_page))
        .route("/ui/instructors/classes/new", get(create_class_page))
        .route("/ui/instructors/classes/new", post(create_class_submit))
        .route("/ui/instructors/classes/{id}/edit", get(edit_class_page))
        .route("/ui/instructors/classes/{id}/edit", post(edit_class_submit))
        .route(
            "/ui/instructors/classes/{id}",
            get(instructor_class_detail_page),
        )
        .route("/ui/logout", get(logout))
        .layer(middleware::from_fn(jwt::require_ui_access))
        .layer(middleware::from_fn(jwt::jwt_auth_web));

    let admin_ui_routes = Router::new()
        .route("/ui/admin/users", get(admin_users_page))
        .route("/ui/admin/users", post(admin_create_user_submit))
        .layer(middleware::from_fn(jwt::require_admin_access))
        .layer(middleware::from_fn(jwt::jwt_auth_web));

    Router::new()
        .route("/ui/login", get(login_page))
        .route("/ui/login", post(login_submit))
        .merge(protected_ui_routes)
        .merge(admin_ui_routes)
}
