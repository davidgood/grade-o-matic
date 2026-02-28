use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use simple_dto_mapper_derive::DtoFrom;
use utoipa::ToSchema;

use crate::domains::assignments::domain::model::Assignment;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, DtoFrom)]
#[dto(from = Assignment)]
pub struct AssignmentDto {
    pub id: uuid::Uuid,
    pub title: String,
    pub description: Option<String>,
    #[serde(with = "crate::common::ts_format::option")]
    pub due_at: Option<DateTime<Utc>>,
}
