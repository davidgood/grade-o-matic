use axum::response::Html;
use minijinja::context;

use crate::common::error::AppError;

use super::render_template;

pub async fn assignments_page() -> Result<Html<String>, AppError> {
    let html = render_template(
        "assignments/index.html",
        context! { title => "Assignments" },
    )?;
    Ok(Html(html))
}
