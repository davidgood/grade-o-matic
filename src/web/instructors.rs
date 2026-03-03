use axum::response::Html;
use axum::{
    Extension,
    extract::{Path, State},
};
use minijinja::context;
use std::sync::Arc;
use uuid::Uuid;

use crate::common::error::AppError;
use crate::common::jwt::Claims;
use crate::domains::assignments::AssignmentServiceTrait;
use crate::domains::classes::ClassServiceTrait;
use crate::domains::user::UserRole;

use super::render_template;

pub async fn instructors_page(
    State(class_service): State<Arc<dyn ClassServiceTrait>>,
    Extension(claims): Extension<Claims>,
) -> Result<Html<String>, AppError> {
    let classes = class_service.list().await?;
    let owned_classes = classes
        .into_iter()
        .filter(|class| class.owner_id == Some(claims.sub))
        .collect::<Vec<_>>();

    let html = render_template(
        "instructors/index.html",
        context! {
            title => "Instructors",
            classes => owned_classes,
        },
    )?;
    Ok(Html(html))
}

pub async fn instructor_class_detail_page(
    Path(class_id): Path<Uuid>,
    State(class_service): State<Arc<dyn ClassServiceTrait>>,
    State(assignment_service): State<Arc<dyn AssignmentServiceTrait>>,
    Extension(claims): Extension<Claims>,
) -> Result<Html<String>, AppError> {
    let class = class_service
        .find_by_id(class_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Class not found".into()))?;

    // Allow owner instructors and admins to access class details.
    if !matches!(claims.user_role, UserRole::Admin) && class.owner_id != Some(claims.sub) {
        return Err(AppError::Forbidden);
    }

    let assignments = assignment_service.list_by_class(class_id).await?;

    let html = render_template(
        "instructors/class_detail.html",
        context! {
            title => class.title.clone(),
            class => class,
            assignments => assignments,
        },
    )?;
    Ok(Html(html))
}
