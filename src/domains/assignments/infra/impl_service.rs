use std::sync::Arc;

use async_trait::async_trait;

use crate::domains::assignments::domain::{
    model::Assignment,
    repository::AssignmentRepositoryTrait,
    service::AssignmentServiceTrait,
};

pub struct AssignmentService {
    repository: Arc<dyn AssignmentRepositoryTrait>,
}

impl AssignmentService {
    pub fn new(repository: Arc<dyn AssignmentRepositoryTrait>) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl AssignmentServiceTrait for AssignmentService {
    async fn list_assignments(&self) -> Result<Vec<Assignment>, sqlx::Error> {
        self.repository.find_all().await
    }
}
