use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Assignment {
    pub id: uuid::Uuid,
    pub class_id: uuid::Uuid,
    pub title: String,
    pub description: Option<String>,
    pub due_at: Option<DateTime<Utc>>,
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

pub struct AssignmentWithAttachmentCount {
    pub id: uuid::Uuid,
    pub class_id: uuid::Uuid,
    pub title: String,
    pub description: Option<String>,
    pub due_at: Option<DateTime<Utc>>,
    pub created_by: Option<uuid::Uuid>,
    pub created_at: Option<DateTime<Utc>>,
    pub modified_by: Option<uuid::Uuid>,
    pub modified_at: Option<DateTime<Utc>>,
    pub attachment_count: i32,
}
