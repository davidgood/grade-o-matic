use axum::response::Html;
use chrono::Utc;
use minijinja::context;

use crate::common::error::AppError;

use super::render_template;

pub async fn ui_index() -> Result<Html<String>, AppError> {
    let html = render_template("index.html", context! { title => "Grade-O-Matic" })?;
    Ok(Html(html))
}

pub async fn server_time_fragment() -> Html<String> {
    let now = Utc::now().to_rfc3339();
    Html(format!("<code>Server time: {now}</code>"))
}
