use axum::Extension;
use axum::extract::State;
use axum::response::Html;
use minijinja::context;
use std::sync::Arc;

use super::render_template;
use crate::common::error::AppError;
use crate::common::jwt::Claims;
use crate::domains::classes::ClassServiceTrait;

pub async fn assignments_page(
    State(class_service): State<Arc<dyn ClassServiceTrait>>,
    Extension(claims): Extension<Claims>,
) -> Result<Html<String>, AppError> {
    let assignments = class_service
        .list_classes_with_assignments(claims.sub)
        .await?;

    let html = render_template(
        "assignments/index.html",
        context! {
            title => "Assignments",
            assignments => assignments,
            count => assignments.len(),
        },
    )?;
    Ok(Html(html))
}
