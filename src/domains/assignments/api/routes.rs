use axum::{Router, routing::get};

use super::handlers::list_assignments;

pub fn assignment_routes<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    Router::new().route("/", get(list_assignments))
}
