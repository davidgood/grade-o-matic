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
