use axum::Extension;
use axum::extract::State;
use axum::response::Html;
use minijinja::context;
use std::sync::Arc;

use crate::common::error::AppError;
use crate::common::jwt::Claims;
use crate::domains::classes::ClassServiceTrait;

use super::super::render_template;

pub async fn assignments_table_fragment(
    State(class_service): State<Arc<dyn ClassServiceTrait>>,
    Extension(claims): Extension<Claims>,
) -> Result<Html<String>, AppError> {
    let assignments = class_service
        .list_classes_with_assignments(claims.sub)
        .await?;

    let html = render_template(
        "assignments/partials/_table.html",
        context! {
            assignments => assignments,
            count => assignments.len(),
        },
    )?;
    Ok(Html(html))
}
