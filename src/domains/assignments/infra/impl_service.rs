use std::sync::Arc;

use async_trait::async_trait;
use sqlx::PgPool;
use crate::common::error::AppError;
use crate::domains::assignments::domain::{
    repository::AssignmentRepositoryTrait,
    service::AssignmentServiceTrait,
};
use crate::domains::assignments::dto::assignment_dto::AssignmentDto;
use crate::domains::assignments::infra::impl_repository::AssignmentRepository;

#[derive(Clone)]
pub struct AssignmentService {
    pub pool: PgPool,
    repository: Arc<dyn AssignmentRepositoryTrait + Send + Sync>,
}

#[async_trait]
impl AssignmentServiceTrait for AssignmentService {
    fn create_service(
        pool: PgPool
    ) -> Arc<dyn AssignmentServiceTrait> {
        Arc::new(Self {
            pool,
            repository: Arc::new(AssignmentRepository {}),
        })
    }

    async fn list_assignments(&self) -> Result<Vec<AssignmentDto>, AppError> {
        match self.repository.find_all(self.pool.clone()).await {
            Ok(assignments) => {
                let assignment_dtos: Vec<AssignmentDto> = assignments.into_iter().map(Into::into).collect();
                Ok(assignment_dtos)
            }
            Err(err) => {
                tracing::error!("Error fetching users: {err}");
                Err(AppError::DatabaseError(err))
            }
        }
    }
}
