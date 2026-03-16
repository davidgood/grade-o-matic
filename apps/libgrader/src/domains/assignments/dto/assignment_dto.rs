use crate::domains::assignments::domain::model::{
    Assignment, AssignmentDeadlineType, AssignmentWithAttachmentCount,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use simple_dto_mapper_derive::DtoFrom;
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, DtoFrom)]
#[dto(from = Assignment)]
pub struct AssignmentDto {
    pub id: uuid::Uuid,
    pub class_id: uuid::Uuid,
    pub title: String,
    pub description: Option<String>,
    #[serde(with = "crate::common::ts_format::option")]
    pub due_at: Option<DateTime<Utc>>,
    pub deadline_type: AssignmentDeadlineType,
    pub points: Option<i16>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreateAssignmentDto {
    pub class_id: uuid::Uuid,
    #[validate(length(max = 255, message = "Title cannot exceed 255 characters"))]
    pub title: String,
    pub description: Option<String>,
    pub due_at: Option<DateTime<Utc>>,
    pub deadline_type: AssignmentDeadlineType,
    pub modified_by: uuid::Uuid,
    pub points: Option<i16>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct UpdateAssignmentDto {
    pub id: uuid::Uuid,
    pub class_id: uuid::Uuid,
    #[validate(length(max = 255, message = "Title cannot exceed 255 characters"))]
    pub title: String,
    pub description: Option<String>,
    pub due_at: Option<DateTime<Utc>>,
    pub deadline_type: AssignmentDeadlineType,
    pub modified_by: uuid::Uuid,
    pub points: Option<i16>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate, DtoFrom)]
#[dto(from = AssignmentWithAttachmentCount)]
pub struct AssignmentWithAttachmentCountDto {
    pub id: uuid::Uuid,
    pub class_id: uuid::Uuid,
    pub title: String,
    pub description: Option<String>,
    #[serde(with = "crate::common::ts_format::option")]
    pub due_at: Option<DateTime<Utc>>,
    pub deadline_type: AssignmentDeadlineType,
    pub points: Option<i16>,
    pub attachment_count: i32,
}
