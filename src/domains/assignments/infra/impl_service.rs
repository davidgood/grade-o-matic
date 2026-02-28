use std::sync::Arc;

use crate::common::error::AppError;
use crate::domains::assignments::domain::{
    repository::AssignmentRepositoryTrait, service::AssignmentServiceTrait,
};
use crate::domains::assignments::dto::assignment_dto::{
    AssignmentDto, CreateAssignmentDto, UpdateAssignmentDto,
};
use crate::domains::assignments::infra::impl_repository::AssignmentRepository;
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct AssignmentService {
    pub pool: PgPool,
    repository: Arc<dyn AssignmentRepositoryTrait + Send + Sync>,
}

#[async_trait]
impl AssignmentServiceTrait for AssignmentService {
    fn create_service(pool: PgPool) -> Arc<dyn AssignmentServiceTrait> {
        Arc::new(Self {
            pool,
            repository: Arc::new(AssignmentRepository {}),
        })
    }

    async fn list(&self) -> Result<Vec<AssignmentDto>, AppError> {
        match self.repository.find_all(self.pool.clone()).await {
            Ok(assignments) => {
                let assignment_dtos: Vec<AssignmentDto> =
                    assignments.into_iter().map(Into::into).collect();
                Ok(assignment_dtos)
            }
            Err(err) => {
                tracing::error!("Error fetching users: {err}");
                Err(AppError::DatabaseError(err))
            }
        }
    }

    async fn get_by_id(&self, id: Uuid) -> Result<Option<AssignmentDto>, AppError> {
        match self.repository.find_by_id(id, self.pool.clone()).await {
            Ok(Some(assignment)) => Ok(Some(AssignmentDto::from(assignment))),
            Ok(None) => Err(AppError::NotFound("Assignment not found".into())),
            Err(err) => {
                tracing::error!("Error retrieving assignment: {err}");
                Err(AppError::DatabaseError(err))
            }
        }
    }

    async fn create(&self, assignment: CreateAssignmentDto) -> Result<AssignmentDto, AppError> {
        let mut tx = self.pool.begin().await?;

        let assignment_id = match self.repository.create(&mut tx, assignment).await {
            Ok(assignment_id) => assignment_id,
            Err(err) => {
                tx.rollback().await?;
                tracing::error!("Error creating assignment: {err}");
                return Err(AppError::DatabaseError(err));
            }
        };

        tx.commit().await?;

        match self
            .repository
            .find_by_id(assignment_id, self.pool.clone())
            .await
        {
            Ok(Some(assignment)) => Ok(AssignmentDto::from(assignment)),
            Ok(None) => Err(AppError::NotFound("Assignment not found".into())),
            Err(err) => {
                tracing::error!("Error retrieving assignment: {err}");
                Err(AppError::DatabaseError(err))
            }
        }
    }

    async fn update(&self, assignment: UpdateAssignmentDto) -> Result<AssignmentDto, AppError> {
        let mut tx = self.pool.begin().await?;

        match self
            .repository
            .update(&mut tx, assignment.id, assignment)
            .await
        {
            Ok(Some(assignment)) => {
                tx.commit().await?;
                Ok(AssignmentDto::from(assignment))
            }
            Ok(None) => {
                tx.rollback().await?;
                Err(AppError::NotFound("Assignment not found".into()))
            }
            Err(err) => {
                tx.rollback().await?;
                tracing::error!("Error updating assignment: {err}");
                Err(AppError::DatabaseError(err))
            }
        }
    }

    async fn delete(&self, id: Uuid) -> Result<String, AppError> {
        let mut tx = self.pool.begin().await?;
        match self.repository.delete(&mut tx, id).await {
            Ok(true) => {
                tx.commit().await?;
                Ok("Assignment deleted".into())
            }
            Ok(false) => {
                tx.rollback().await?;
                Err(AppError::NotFound("Assignment not found".into()))
            }
            Err(err) => {
                tx.rollback().await?;
                tracing::error!("Error deleting assignment: {err}");
                Err(AppError::DatabaseError(err))
            }
        }
    }
}
