use async_trait::async_trait;

use super::model::Assignment;

#[async_trait]
pub trait AssignmentServiceTrait: Send + Sync {
    async fn list_assignments(&self) -> Result<Vec<Assignment>, sqlx::Error>;
}
