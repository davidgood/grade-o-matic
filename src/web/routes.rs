use axum::{Router, routing::get};

use super::{
    assignments::assignments_page,
    handlers::{server_time_fragment, ui_index},
    htmx::assignments::assignments_table_fragment,
};

pub fn web_routes<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    Router::new()
        .route("/", get(ui_index))
        .route("/ui", get(ui_index))
        .route("/ui/assignments", get(assignments_page))
        .route("/ui/fragments/server-time", get(server_time_fragment))
        .route(
            "/ui/fragments/assignments/table",
            get(assignments_table_fragment),
        )
}
