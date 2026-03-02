use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
/// Domain model representing a user in the application.
#[derive(Debug, Clone, FromRow)]
pub struct User {
    pub id: uuid::Uuid,
    pub username: String,
    pub email: Option<String>,
    pub created_by: Option<uuid::Uuid>,
    pub created_at: Option<DateTime<Utc>>,
    pub modified_by: Option<uuid::Uuid>,
    pub modified_at: Option<DateTime<Utc>>,
    pub file_id: Option<String>,
    pub origin_file_name: Option<String>,
    pub user_role: UserRole,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "user_role_enum", rename_all = "lowercase")]
pub enum UserRole {
    Admin,
    Instructor,
    Ta,
    Student,
}
