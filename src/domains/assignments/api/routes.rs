use axum::{routing::get, Router};

use super::handlers::list_assignments;

pub fn assignment_routes<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    Router::new().route("/", get(list_assignments))
}
