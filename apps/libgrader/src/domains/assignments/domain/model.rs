use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema, sqlx::Type, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "assignment_deadline_type_enum", rename_all = "snake_case")]
pub enum AssignmentDeadlineType {
    HardCutoff,
    SoftDeadline,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Assignment {
    pub id: uuid::Uuid,
    pub class_id: uuid::Uuid,
    pub title: String,
    pub description: Option<String>,
    pub due_at: Option<DateTime<Utc>>,
    pub deadline_type: AssignmentDeadlineType,
    pub points: Option<i16>,
    pub created_by: Option<uuid::Uuid>,
    pub created_at: Option<DateTime<Utc>>,
    pub modified_by: Option<uuid::Uuid>,
    pub modified_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AssignmentAttachment {
    pub assignment_id: uuid::Uuid,
    pub file_id: uuid::Uuid,
    pub file_name: String,
    pub origin_file_name: String,
    pub file_url: String,
    pub content_type: String,
    pub file_size: i64,
    pub created_by: Option<uuid::Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct StudentAssignmentSubmission {
    pub assignment_id: uuid::Uuid,
    pub file_id: uuid::Uuid,
    pub file_name: String,
    pub origin_file_name: String,
    pub file_url: String,
    pub content_type: String,
    pub file_size: i64,
    pub submitted_by: uuid::Uuid,
    pub submitted_at: DateTime<Utc>,
    pub deadline_type: AssignmentDeadlineType,
    pub is_late: bool,
    pub grading_status: Option<String>,
    pub grading_completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]

pub struct AssignmentWithAttachmentCount {
    pub id: uuid::Uuid,
    pub class_id: uuid::Uuid,
    pub title: String,
    pub description: Option<String>,
    pub due_at: Option<DateTime<Utc>>,
    pub deadline_type: AssignmentDeadlineType,
    pub created_by: Option<uuid::Uuid>,
    pub created_at: Option<DateTime<Utc>>,
    pub modified_by: Option<uuid::Uuid>,
    pub modified_at: Option<DateTime<Utc>>,
    pub attachment_count: i32,
    pub points: Option<i16>,
}
