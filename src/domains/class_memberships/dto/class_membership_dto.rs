use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use simple_dto_mapper_derive::DtoFrom;
use utoipa::ToSchema;

use crate::domains::class_memberships::domain::model::{ClassMembership, ClassMembershipRole};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, DtoFrom)]
#[dto(from = ClassMembership)]
pub struct ClassMembershipDto {
    pub id: uuid::Uuid,
    pub class_id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub role: ClassMembershipRole,
    #[serde(with = "crate::common::ts_format::option")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(with = "crate::common::ts_format::option")]
    pub modified_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateClassMembershipDto {
    pub class_id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub role: ClassMembershipRole,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateClassMembershipDto {
    pub id: uuid::Uuid,
    pub role: ClassMembershipRole,
}
