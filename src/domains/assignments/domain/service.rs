use crate::common::error::AppError;
use crate::domains::assignments::dto::assignment_dto::{
    AssignmentDto, CreateAssignmentDto, UpdateAssignmentDto,
};

use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;

#[async_trait]
pub trait AssignmentServiceTrait: Send + Sync {
    fn create_service(pool: PgPool) -> Arc<dyn AssignmentServiceTrait>
    where
        Self: Sized;

    async fn list(&self) -> Result<Vec<AssignmentDto>, AppError>;

    async fn get_by_id(&self, id: uuid::Uuid) -> Result<Option<AssignmentDto>, AppError>;

    async fn create(&self, assignment: CreateAssignmentDto) -> Result<AssignmentDto, AppError>;

    async fn update(&self, assignment: UpdateAssignmentDto) -> Result<AssignmentDto, AppError>;

    async fn delete(&self, id: uuid::Uuid) -> Result<String, AppError>;
}
