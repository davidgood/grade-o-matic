use super::model::Assignment;
use crate::domains::assignments::dto::assignment_dto::{CreateAssignmentDto, UpdateAssignmentDto};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

#[async_trait]
pub trait AssignmentRepositoryTrait: Send + Sync {
    fn new(pool: PgPool) -> Self
    where
        Self: Sized;

    async fn find_all(&self) -> Result<Vec<Assignment>, sqlx::Error>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Assignment>, sqlx::Error>;
    async fn create(&self, assignment: CreateAssignmentDto) -> Result<Uuid, sqlx::Error>;
    async fn update(
        &self,
        id: Uuid,
        assignment: UpdateAssignmentDto,
    ) -> Result<Option<Assignment>, sqlx::Error>;
    async fn delete(&self, id: Uuid) -> Result<bool, sqlx::Error>;
}
