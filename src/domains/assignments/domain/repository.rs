use super::model::Assignment;
use crate::domains::assignments::dto::assignment_dto::{CreateAssignmentDto, UpdateAssignmentDto};
use async_trait::async_trait;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

#[async_trait]
pub trait AssignmentRepositoryTrait: Send + Sync {
    async fn find_all(&self, pool: PgPool) -> Result<Vec<Assignment>, sqlx::Error>;
    async fn find_by_id(&self, id: Uuid, pool: PgPool) -> Result<Option<Assignment>, sqlx::Error>;
    async fn create(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        assignment: CreateAssignmentDto,
    ) -> Result<Uuid, sqlx::Error>;

    async fn update(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        id: Uuid,
        assignment: UpdateAssignmentDto,
    ) -> Result<Option<Assignment>, sqlx::Error>;

    async fn delete(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        id: Uuid,
    ) -> Result<bool, sqlx::Error>;
}
