use async_trait::async_trait;

use crate::domains::assignments::domain::{model::Assignment, repository::AssignmentRepositoryTrait};

pub struct AssignmentRepository;

#[async_trait]
impl AssignmentRepositoryTrait for AssignmentRepository {
    async fn find_all(&self) -> Result<Vec<Assignment>, sqlx::Error> {
        Ok(Vec::new())
    }
}
