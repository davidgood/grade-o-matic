use axum::{
    Router,
    extract::FromRef,
    middleware,
    routing::{get, post},
};
use std::sync::Arc;

use crate::common::jwt;
use crate::domains::auth::AuthServiceTrait;

use super::{
    assignments::assignments_page,
    handlers::{login_page, login_submit, logout, server_time_fragment, ui_index},
    htmx::assignments::assignments_table_fragment,
};

pub fn web_routes<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    Arc<dyn AuthServiceTrait>: FromRef<S>,
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
        .route("/ui/logout", get(logout))
        .layer(middleware::from_fn(jwt::require_ui_access))
        .layer(middleware::from_fn(jwt::jwt_auth_web));

    Router::new()
        .route("/ui/login", get(login_page))
        .route("/ui/login", post(login_submit))
        .merge(protected_ui_routes)
}
