use std::sync::Arc;
use async_trait::async_trait;
use sqlx::PgPool;
use crate::common::error::AppError;
use crate::domains::assignments::dto::assignment_dto::AssignmentDto;

#[async_trait]
pub trait AssignmentServiceTrait: Send + Sync {
    fn create_service(
        pool: PgPool
    ) -> Arc<dyn AssignmentServiceTrait>
    where
        Self: Sized;

    async fn list_assignments(&self) -> Result<Vec<AssignmentDto>, AppError>;

}
