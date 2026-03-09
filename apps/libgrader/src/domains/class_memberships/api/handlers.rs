use crate::common::dto::RestApiResponse;
use crate::common::error::AppError;
use crate::domains::class_memberships::domain::service::ClassMembershipServiceTrait;
use crate::domains::class_memberships::dto::class_membership_dto::{
    ClassMembershipDto, CreateClassMembershipDto, UpdateClassMembershipDto,
};
use axum::Json;
use axum::extract::State;
use axum::response::IntoResponse;
use std::sync::Arc;
use uuid::Uuid;

#[utoipa::path(
    get,
    path = "/class-memberships/class/{class_id}",
    responses((status = 200, description = "Get class memberships by class ID", body = [ClassMembershipDto])),
    tag = "ClassMemberships"
)]
pub async fn get_class_memberships_by_class_id(
    State(service): State<Arc<dyn ClassMembershipServiceTrait>>,
    axum::extract::Path(class_id): axum::extract::Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let memberships = service.list_by_class_id(class_id).await?;
    Ok(RestApiResponse::success(memberships))
}

#[utoipa::path(
    get,
    path = "/class-memberships/user/{user_id}",
    responses((status = 200, description = "Get class memberships by user ID", body = [ClassMembershipDto])),
    tag = "ClassMemberships"
)]
pub async fn get_class_memberships_by_user_id(
    State(service): State<Arc<dyn ClassMembershipServiceTrait>>,
    axum::extract::Path(user_id): axum::extract::Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let memberships = service.list_by_user_id(user_id).await?;
    Ok(RestApiResponse::success(memberships))
}

#[utoipa::path(
    get,
    path = "/class-memberships/{id}",
    responses((status = 200, description = "Get class membership by ID", body = ClassMembershipDto)),
    tag = "ClassMemberships"
)]
pub async fn get_class_membership_by_id(
    State(service): State<Arc<dyn ClassMembershipServiceTrait>>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    match service.find_by_id(id).await? {
        Some(membership) => Ok(RestApiResponse::success(membership)),
        None => Err(AppError::NotFound("Class membership not found".to_string())),
    }
}

#[utoipa::path(
    post,
    path = "/class-memberships",
    request_body = CreateClassMembershipDto,
    responses((status = 201, description = "Create class membership", body = ClassMembershipDto)),
    tag = "ClassMemberships"
)]
pub async fn create_class_membership(
    State(service): State<Arc<dyn ClassMembershipServiceTrait>>,
    Json(payload): Json<CreateClassMembershipDto>,
) -> Result<impl IntoResponse, AppError> {
    let membership = service.create(payload).await?;
    Ok(RestApiResponse::created(membership))
}

#[utoipa::path(
    put,
    path = "/class-memberships",
    request_body = UpdateClassMembershipDto,
    responses((status = 200, description = "Update class membership role", body = ClassMembershipDto)),
    tag = "ClassMemberships"
)]
pub async fn update_class_membership(
    State(service): State<Arc<dyn ClassMembershipServiceTrait>>,
    Json(payload): Json<UpdateClassMembershipDto>,
) -> Result<impl IntoResponse, AppError> {
    match service.update(payload).await? {
        Some(updated) => Ok(RestApiResponse::success(updated)),
        None => Err(AppError::NotFound("Class membership not found".to_string())),
    }
}

#[utoipa::path(
    delete,
    path = "/class-memberships/{id}",
    responses((status = 200, description = "Class membership deleted")),
    tag = "ClassMemberships"
)]
pub async fn delete_class_membership(
    State(service): State<Arc<dyn ClassMembershipServiceTrait>>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let message = service.delete(id).await?;
    Ok(RestApiResponse::success_with_message(message, ()))
}
