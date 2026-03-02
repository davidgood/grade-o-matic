use axum::{Router, middleware, routing::get};

use crate::common::jwt;

use super::{
    assignments::assignments_page,
    handlers::{server_time_fragment, ui_index},
    htmx::assignments::assignments_table_fragment,
};

pub fn web_routes<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    let protected_ui_routes = Router::new()
        .route("/ui/assignments", get(assignments_page))
        .route("/ui/fragments/server-time", get(server_time_fragment))
        .route(
            "/ui/fragments/assignments/table",
            get(assignments_table_fragment),
        )
        .layer(middleware::from_fn(jwt::require_ui_access))
        .layer(middleware::from_fn(jwt::jwt_auth));

    Router::new()
        .route("/", get(ui_index))
        .route("/ui", get(ui_index))
        .merge(protected_ui_routes)
}
