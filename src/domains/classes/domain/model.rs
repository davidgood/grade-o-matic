use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Class {
    pub id: uuid::Uuid,
    pub title: String,
    pub description: Option<String>,
    pub term: Option<String>,
    pub owner_id: Option<uuid::Uuid>,
    pub created_by: Option<uuid::Uuid>,
    pub created_at: Option<DateTime<Utc>>,
    pub modified_by: Option<uuid::Uuid>,
    pub modified_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ClassesWithAssignments {
    pub class_id: uuid::Uuid,
    pub class_title: String,
    pub class_term: Option<String>,
    pub assignment_id: Option<uuid::Uuid>,
    pub assignment_title: Option<String>,
    pub assignment_description: Option<String>,
    pub due_at: Option<DateTime<Utc>>,
    pub points: Option<i16>,
}
