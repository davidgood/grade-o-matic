use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ClassMembership {
    pub id: uuid::Uuid,
    pub class_id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub role: ClassMembershipRole,
    pub created_at: Option<DateTime<Utc>>,
    pub modified_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "class_membership_role_enum", rename_all = "lowercase")]
pub enum ClassMembershipRole {
    Ta,
    Student,
}
