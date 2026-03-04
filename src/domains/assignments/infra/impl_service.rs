use std::sync::Arc;

use crate::common::error::AppError;
use crate::domains::assignments::domain::{
    model::AssignmentAttachment, repository::AssignmentRepositoryTrait,
    service::AssignmentServiceTrait,
};
use crate::domains::assignments::dto::assignment_dto::{
    AssignmentDto, AssignmentWithAttachmentCountDto, CreateAssignmentDto, UpdateAssignmentDto,
};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct AssignmentService<R: AssignmentRepositoryTrait> {
    repository: Arc<R>,
}

impl<R> AssignmentService<R>
where
    R: AssignmentRepositoryTrait,
{
    pub fn with_repository(repository: Arc<R>) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<R> AssignmentServiceTrait for AssignmentService<R>
where
    R: AssignmentRepositoryTrait + 'static,
{
    fn create_service(pool: PgPool) -> Arc<dyn AssignmentServiceTrait> {
        Arc::new(Self {
            repository: Arc::new(R::new(pool)),
        })
    }

    async fn list(&self) -> Result<Vec<AssignmentDto>, AppError> {
        match self.repository.find_all().await {
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

    async fn list_by_class(&self, class_id: Uuid) -> Result<Vec<AssignmentDto>, AppError> {
        match self.repository.find_by_class_id(class_id).await {
            Ok(assignments) => {
                let assignment_dtos: Vec<AssignmentDto> =
                    assignments.into_iter().map(Into::into).collect();
                Ok(assignment_dtos)
            }
            Err(err) => {
                tracing::error!("Error fetching assignments by class: {err}");
                Err(AppError::DatabaseError(err))
            }
        }
    }

    async fn list_by_class_with_attachment_count(
        &self,
        class_id: Uuid,
    ) -> Result<Vec<AssignmentWithAttachmentCountDto>, AppError> {
        match self
            .repository
            .find_by_class_id_with_attachment_count(class_id)
            .await
        {
            Ok(assignments) => {
                let assignment_dtos: Vec<AssignmentWithAttachmentCountDto> =
                    assignments.into_iter().map(Into::into).collect();
                Ok(assignment_dtos)
            }
            Err(err) => {
                tracing::error!("Error fetching assignments by class: {err}");
                Err(AppError::DatabaseError(err))
            }
        }
    }

    async fn list_attachments(
        &self,
        assignment_id: Uuid,
    ) -> Result<Vec<AssignmentAttachment>, AppError> {
        self.repository
            .list_attachments(assignment_id)
            .await
            .map_err(|err| {
                tracing::error!("Error fetching assignment attachments: {err}");
                AppError::DatabaseError(err)
            })
    }

    async fn attach_file(
        &self,
        assignment_id: Uuid,
        file_id: Uuid,
        created_by: Uuid,
    ) -> Result<(), AppError> {
        self.repository
            .add_attachment(assignment_id, file_id, created_by)
            .await
            .map_err(|err| {
                tracing::error!("Error attaching file to assignment: {err}");
                AppError::DatabaseError(err)
            })
    }

    async fn remove_file(&self, assignment_id: Uuid, file_id: Uuid) -> Result<bool, AppError> {
        self.repository
            .remove_attachment(assignment_id, file_id)
            .await
            .map_err(|err| {
                tracing::error!("Error removing file from assignment: {err}");
                AppError::DatabaseError(err)
            })
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<AssignmentDto>, AppError> {
        match self.repository.find_by_id(id).await {
            Ok(Some(assignment)) => Ok(Some(AssignmentDto::from(assignment))),
            Ok(None) => Err(AppError::NotFound("Assignment not found".into())),
            Err(err) => {
                tracing::error!("Error retrieving assignment: {err}");
                Err(AppError::DatabaseError(err))
            }
        }
    }

    async fn create(&self, assignment: CreateAssignmentDto) -> Result<AssignmentDto, AppError> {
        let assignment_id = match self.repository.create(assignment).await {
            Ok(assignment_id) => assignment_id,
            Err(err) => {
                tracing::error!("Error creating assignment: {err}");
                return Err(AppError::DatabaseError(err));
            }
        };

        match self.repository.find_by_id(assignment_id).await {
            Ok(Some(assignment)) => Ok(AssignmentDto::from(assignment)),
            Ok(None) => Err(AppError::NotFound("Assignment not found".into())),
            Err(err) => {
                tracing::error!("Error retrieving assignment: {err}");
                Err(AppError::DatabaseError(err))
            }
        }
    }

    async fn update(&self, assignment: UpdateAssignmentDto) -> Result<AssignmentDto, AppError> {
        match self.repository.update(assignment.id, assignment).await {
            Ok(Some(assignment)) => Ok(AssignmentDto::from(assignment)),
            Ok(None) => Err(AppError::NotFound("Assignment not found".into())),
            Err(err) => {
                tracing::error!("Error updating assignment: {err}");
                Err(AppError::DatabaseError(err))
            }
        }
    }

    async fn delete(&self, id: Uuid) -> Result<String, AppError> {
        match self.repository.delete(id).await {
            Ok(true) => Ok("Assignment deleted".into()),
            Ok(false) => Err(AppError::NotFound("Assignment not found".into())),
            Err(err) => {
                tracing::error!("Error deleting assignment: {err}");
                Err(AppError::DatabaseError(err))
            }
        }
    }
}
