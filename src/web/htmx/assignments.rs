use axum::response::Html;
use minijinja::context;

use crate::common::error::AppError;

use super::super::render_template;

pub async fn assignments_table_fragment() -> Result<Html<String>, AppError> {
    let html = render_template(
        "partials/empty_state.html",
        context! { message => "No assignments yet." },
    )?;
    Ok(Html(html))
}
