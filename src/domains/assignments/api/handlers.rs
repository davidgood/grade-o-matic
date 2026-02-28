use crate::common::dto::RestApiResponse;
use crate::common::error::AppError;
use crate::common::jwt::Claims;
use crate::domains::assignments::AssignmentServiceTrait;
use crate::domains::assignments::dto::assignment_dto::{
    AssignmentDto, CreateAssignmentDto, UpdateAssignmentDto,
};
use axum::extract::State;
use axum::response::IntoResponse;
use axum::{Extension, Json};
use std::sync::Arc;

#[utoipa::path(
    get,
    path = "/assignments/{id}",
    responses((status = 200, description = "Get assignment by ID", body = AssignmentDto)),
    tag = "Assignments"
)]
pub async fn get_assignment_by_id(
    State(service): State<Arc<dyn AssignmentServiceTrait>>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let device = service.get_by_id(id).await?;
    Ok(RestApiResponse::success(device))
}

#[utoipa::path(
    get,
    path = "/assignments",
    responses((status = 200, description = "Get assignments", body = AssignmentDto)),
    tag = "Assignments"
)]
pub async fn get_assignments(
    State(service): State<Arc<dyn AssignmentServiceTrait>>,
) -> Result<impl IntoResponse, AppError> {
    let assignments = service.list().await?;
    Ok(RestApiResponse::success(assignments))
}

#[utoipa::path(
    post,
    path = "/assignments",
    request_body(
        content = CreateAssignmentDto,
        content_type = "multipart/form-data",
        description = "Assignment creation"
    ),
    responses((status = 200, description = "Create a new assignment", body = CreateAssignmentDto)),
    tag = "Assignments"
)]
pub async fn create_assignment(
    State(service): State<Arc<dyn AssignmentServiceTrait>>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<CreateAssignmentDto>,
) -> Result<impl IntoResponse, AppError> {
    let mut payload = payload;
    payload.modified_by = claims.sub;

    let device = service.create(payload).await?;
    Ok(RestApiResponse::success(device))
}

#[utoipa::path(
    put,
    path = "/assignments/{id}",
    request_body = UpdateAssignmentDto,
    responses((status = 200, description = "Update assignment", body = UpdateAssignmentDto)),
    tag = "Assignments"
)]
pub async fn update_assignment(
    State(assignment_service): State<Arc<dyn AssignmentServiceTrait>>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<UpdateAssignmentDto>,
) -> Result<impl IntoResponse, AppError> {
    // Set the modified_by field to the current user's ID.
    let mut payload = payload;
    payload.modified_by = claims.sub;

    let device = assignment_service.update(payload).await?;
    Ok(RestApiResponse::success(device))
}

#[utoipa::path(
    delete,
    path = "/assignments/{id}",
    responses((status = 200, description = "Assignment deleted")),
    tag = "Assignments"
)]
pub async fn delete_assignment(
    State(assignment_service): State<Arc<dyn AssignmentServiceTrait>>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let message = assignment_service.delete(id).await?;

    Ok(RestApiResponse::success_with_message(message, ()))
}
