use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use simple_dto_mapper_derive::DtoFrom;
use utoipa::ToSchema;
use validator::Validate;

use crate::domains::classes::domain::model::Class;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, DtoFrom)]
#[dto(from = Class)]
pub struct ClassDto {
    pub id: uuid::Uuid,
    pub title: String,
    pub description: Option<String>,
    pub term: Option<String>,
    pub owner_id: Option<uuid::Uuid>,
    #[serde(with = "crate::common::ts_format::option")]
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreateClassDto {
    #[validate(length(max = 255, message = "Title cannot exceed 255 characters"))]
    pub title: String,
    pub description: Option<String>,
    pub term: Option<String>,
    pub owner_id: Option<uuid::Uuid>,
    pub modified_by: uuid::Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct UpdateClassDto {
    pub id: uuid::Uuid,
    #[validate(length(max = 255, message = "Title cannot exceed 255 characters"))]
    pub title: String,
    pub description: Option<String>,
    pub term: Option<String>,
    pub owner_id: Option<uuid::Uuid>,
    pub modified_by: uuid::Uuid,
}
