use async_trait::async_trait;

use super::model::Assignment;

#[async_trait]
pub trait AssignmentRepositoryTrait: Send + Sync {
    async fn find_all(&self) -> Result<Vec<Assignment>, sqlx::Error>;
}
