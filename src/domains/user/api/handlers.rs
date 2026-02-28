use crate::{
    common::{
        dto::RestApiResponse, error::AppError, jwt::Claims,
        multipart_helper::parse_multipart_to_maps,
    },
    domains::{
        file::dto::file_dto::UploadFileDto,
        user::{
            dto::user_dto::{CreateUserMultipartDto, SearchUserDto, UpdateUserDto, UserDto},
            UserAssetPattern, UserServiceTrait,
        },
    },
};

use axum::{
    extract::{Multipart, State},
    response::IntoResponse,
    Extension, Json,
};

use validator::Validate;
use uuid::Uuid;
use std::sync::Arc;

#[utoipa::path(
    get,
    path = "/user/{id}",
    responses((status = 200, description = "Get user by ID", body = UserDto)),
    tag = "Users"
)]
pub async fn get_user_by_id(
    State(user_service): State<Arc<dyn UserServiceTrait>>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let user = user_service.get_user_by_id(id).await?;
    Ok(RestApiResponse::success(user))
}

#[utoipa::path(
    post,
    path = "/user/list",
    request_body = SearchUserDto,
    responses((status = 200, description = "List users by condition", body = [UserDto])),
    tag = "Users"
)]
pub async fn get_user_list(
    State(user_service): State<Arc<dyn UserServiceTrait>>,
    Json(payload): Json<SearchUserDto>,
) -> Result<impl IntoResponse, AppError> {
    let users = user_service.get_user_list(payload).await?;
    Ok(RestApiResponse::success(users))
}

#[utoipa::path(
    get,
    path = "/user",
    responses((status = 200, description = "List all users", body = [UserDto])),
    tag = "Users"
)]
pub async fn get_users(
    State(user_service): State<Arc<dyn UserServiceTrait>>,
) -> Result<impl IntoResponse, AppError> {
    let users = user_service.get_users().await?;
    Ok(RestApiResponse::success(users))
}

#[utoipa::path(
    post,
    path = "/user",
    request_body(
        content = CreateUserMultipartDto,
        content_type = "multipart/form-data",
        description = "User creation with optional profile picture upload"
    ),
    responses((status = 200, description = "Create a new user", body = UserDto)),
    tag = "Users"
)]
pub async fn create_user(
    State(user_service): State<Arc<dyn UserServiceTrait>>,
    State(asset_pattern): State<UserAssetPattern>,
    Extension(claims): Extension<Claims>,
    multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let modified_by = claims.sub;

    let (mut fields, mut files) = parse_multipart_to_maps(multipart, &asset_pattern.0).await?;

    // Validate required fields.
    let username = fields
        .remove("username")
        .ok_or(AppError::ValidationError("Missing username".into()))?;
    let email = fields
        .remove("email")
        .ok_or(AppError::ValidationError("Missing email".into()))?;

    // Prepare the CreateUser DTO.
    let create_user = CreateUserMultipartDto {
        username,
        email,
        modified_by,
        profile_picture: None,
    };

    // Validate the CreateUser DTO.
    create_user
        .validate()
        .map_err(|err| AppError::ValidationError(format!("Invalid input: {}", err)))?;

    let mut upload_file_dto = None;

    // If a profile picture was uploaded, handle it.
    if files.contains_key("profile_picture") {
        let profile_files = files
            .remove("profile_picture")
            .ok_or(AppError::ValidationError("Missing profile picture".into()))?;

        if let Some(file) = profile_files.into_iter().next() {
            upload_file_dto = Some(UploadFileDto {
                file,
                user_id: None,
                modified_by: claims.sub,
            });
        }
    }

    let user = user_service.create_user(create_user, upload_file_dto.as_mut()).await?;

    Ok(RestApiResponse::success(user))
}

#[utoipa::path(
    put,
    path = "/user/{id}",
    request_body = UpdateUserDto,
    responses((status = 200, description = "Update user", body = UserDto)),
    tag = "Users"
)]
pub async fn update_user(
    State(user_service): State<Arc<dyn UserServiceTrait>>,
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
    Json(payload): Json<UpdateUserDto>,
) -> Result<impl IntoResponse, AppError> {
    payload.validate().map_err(|err| {
        tracing::error!("Validation error: {err}");
        AppError::ValidationError(format!("Invalid input: {}", err))
    })?;

    // Set the modified_by field to the current user's ID.
    let mut payload = payload;
    payload.modified_by = claims.sub;

    let user = user_service.update_user(id, payload).await?;
    Ok(RestApiResponse::success(user))
}

#[utoipa::path(
    delete,
    path = "/user/{id}",
    responses((status = 200, description = "User deleted")),
    tag = "Users"
)]
pub async fn delete_user(
    State(user_service): State<Arc<dyn UserServiceTrait>>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let message = user_service.delete_user(id).await?;
    Ok(RestApiResponse::success_with_message(message, ()))
}
