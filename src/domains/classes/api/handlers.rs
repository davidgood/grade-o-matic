use crate::common::dto::RestApiResponse;
use crate::common::error::AppError;
use crate::common::jwt::Claims;
use crate::domains::classes::domain::service::ClassServiceTrait;
use crate::domains::classes::dto::class_dto::{ClassDto, CreateClassDto, UpdateClassDto};
use axum::extract::State;
use axum::response::IntoResponse;
use axum::{Extension, Json};
use std::sync::Arc;

#[utoipa::path(
    get,
    path = "/classes",
    responses((status = 200, description = "Get classes", body = ClassDto)),
    tag = "Classes"
)]
pub async fn get_classes(
    State(class_service): State<Arc<dyn ClassServiceTrait>>,
) -> Result<impl IntoResponse, AppError> {
    let classes = class_service.list().await?;
    Ok(RestApiResponse::success(classes))
}

#[utoipa::path(
    get,
    path = "/classes/{id}",
    responses((status = 200, description = "Get class by ID", body = ClassDto)),
    tag = "Classes"
)]
pub async fn get_class_by_id(
    State(class_service): State<Arc<dyn ClassServiceTrait>>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let class = class_service.find_by_id(id).await?;
    match class {
        Some(class) => Ok(RestApiResponse::success(class)),
        None => Err(AppError::NotFound("Class not found".to_string())),
    }
}

#[utoipa::path(
    post,
    path = "/classes",
    request_body(
        content = CreateClassDto,
        content_type = "json",
        description = "Class creation"
    ),
    responses((status = 201, description = "Create class", body = ClassDto)),
    tag = "Classes"
)]
pub async fn create_class(
    State(class_service): State<Arc<dyn ClassServiceTrait>>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<CreateClassDto>,
) -> Result<impl IntoResponse, AppError> {
    let mut payload = payload;
    payload.modified_by = claims.sub;

    let class = class_service.create(payload).await?;
    Ok(RestApiResponse::created(class))
}

#[utoipa::path(
    put,
    path = "/classes",
    request_body = UpdateClassDto,
    responses((status = 200, description = "Update class", body = ClassDto)),
    tag = "Classes"
)]
pub async fn update_class(
    State(class_service): State<Arc<dyn ClassServiceTrait>>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<UpdateClassDto>,
) -> Result<impl IntoResponse, AppError> {
    let mut payload = payload;
    payload.modified_by = claims.sub;

    let class = class_service.update(payload).await?;
    Ok(RestApiResponse::success(class))
}

#[utoipa::path(
    delete,
    path = "/classes/{id}",
    responses((status = 200, description = "Class deleted")),
    tag = "Classes"
)]
pub async fn delete_class(
    State(class_service): State<Arc<dyn ClassServiceTrait>>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let message = class_service.delete(id).await?;
    Ok(RestApiResponse::success_with_message(message, ()))
}
