use crate::common::error::AppError;
use crate::domains::assignments::domain::model::{
    AssignmentAttachment, StudentAssignmentSubmission,
};
use crate::domains::assignments::dto::assignment_dto::{
    AssignmentDto, AssignmentWithAttachmentCountDto, CreateAssignmentDto, UpdateAssignmentDto,
};

use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;

#[async_trait]
pub trait AssignmentServiceTrait: Send + Sync {
    fn create_service(pool: PgPool) -> Arc<dyn AssignmentServiceTrait>
    where
        Self: Sized;

    async fn list(&self) -> Result<Vec<AssignmentDto>, AppError>;
    async fn list_by_class(&self, class_id: uuid::Uuid) -> Result<Vec<AssignmentDto>, AppError>;

    async fn list_by_class_with_attachment_count(
        &self,
        class_id: uuid::Uuid,
    ) -> Result<Vec<AssignmentWithAttachmentCountDto>, AppError>;

    async fn list_attachments(
        &self,
        assignment_id: uuid::Uuid,
    ) -> Result<Vec<AssignmentAttachment>, AppError>;
    async fn list_student_submission_history(
        &self,
        assignment_id: uuid::Uuid,
        student_id: uuid::Uuid,
    ) -> Result<Vec<StudentAssignmentSubmission>, AppError>;
    async fn attach_file(
        &self,
        assignment_id: uuid::Uuid,
        file_id: uuid::Uuid,
        created_by: uuid::Uuid,
    ) -> Result<(), AppError>;
    async fn remove_file(
        &self,
        assignment_id: uuid::Uuid,
        file_id: uuid::Uuid,
    ) -> Result<bool, AppError>;

    async fn find_by_id(&self, id: uuid::Uuid) -> Result<Option<AssignmentDto>, AppError>;

    async fn create(&self, assignment: CreateAssignmentDto) -> Result<AssignmentDto, AppError>;

    async fn update(&self, assignment: UpdateAssignmentDto) -> Result<AssignmentDto, AppError>;

    async fn delete(&self, id: uuid::Uuid) -> Result<String, AppError>;
}
