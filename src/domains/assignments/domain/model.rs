use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Assignment {
    pub id: uuid::Uuid,
    pub class_id: String,
    pub title: String,
    pub description: Option<String>,
    pub due_at: Option<DateTime<Utc>>,
    pub created_by: uuid::Uuid,
    pub created_at: Option<DateTime<Utc>>,
    pub modified_by: uuid::Uuid,
    pub modified_at: Option<DateTime<Utc>>,
}
