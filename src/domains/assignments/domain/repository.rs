use async_trait::async_trait;
use sqlx::PgPool;
use super::model::Assignment;

#[async_trait]
pub trait AssignmentRepositoryTrait: Send + Sync {
    async fn find_all(&self, pool: PgPool) -> Result<Vec<Assignment>, sqlx::Error>;
}
