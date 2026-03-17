use super::model::{
    Assignment, AssignmentAttachment, AssignmentWithAttachmentCount, StudentAssignmentExtension,
    StudentAssignmentSubmission,
};
use crate::domains::assignments::dto::assignment_dto::{
    CreateAssignmentDto, UpdateAssignmentDto, UpsertStudentAssignmentExtensionDto,
};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

#[async_trait]
pub trait AssignmentRepositoryTrait: Send + Sync {
    fn new(pool: PgPool) -> Self
    where
        Self: Sized;

    async fn find_all(&self) -> Result<Vec<Assignment>, sqlx::Error>;
    async fn find_by_class_id(&self, class_id: Uuid) -> Result<Vec<Assignment>, sqlx::Error>;
    async fn find_by_class_id_for_student(
        &self,
        class_id: Uuid,
        student_id: Uuid,
    ) -> Result<Vec<Assignment>, sqlx::Error>;

    async fn find_by_class_id_with_attachment_count(
        &self,
        class_id: Uuid,
    ) -> Result<Vec<AssignmentWithAttachmentCount>, sqlx::Error>;

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Assignment>, sqlx::Error>;
    async fn find_by_id_for_student(
        &self,
        id: Uuid,
        student_id: Uuid,
    ) -> Result<Option<Assignment>, sqlx::Error>;
    async fn list_attachments(
        &self,
        assignment_id: Uuid,
    ) -> Result<Vec<AssignmentAttachment>, sqlx::Error>;
    async fn list_student_submission_history(
        &self,
        assignment_id: Uuid,
        student_id: Uuid,
    ) -> Result<Vec<StudentAssignmentSubmission>, sqlx::Error>;
    async fn list_student_extensions(
        &self,
        assignment_id: Uuid,
    ) -> Result<Vec<StudentAssignmentExtension>, sqlx::Error>;
    async fn upsert_student_extension(
        &self,
        extension: UpsertStudentAssignmentExtensionDto,
    ) -> Result<StudentAssignmentExtension, sqlx::Error>;
    async fn delete_student_extension(
        &self,
        assignment_id: Uuid,
        student_id: Uuid,
    ) -> Result<bool, sqlx::Error>;

    async fn add_attachment(
        &self,
        assignment_id: Uuid,
        file_id: Uuid,
        created_by: Uuid,
    ) -> Result<(), sqlx::Error>;

    async fn remove_attachment(
        &self,
        assignment_id: Uuid,
        file_id: Uuid,
    ) -> Result<bool, sqlx::Error>;

    async fn create(&self, assignment: CreateAssignmentDto) -> Result<Uuid, sqlx::Error>;

    async fn update(
        &self,
        id: Uuid,
        assignment: UpdateAssignmentDto,
    ) -> Result<Option<Assignment>, sqlx::Error>;

    async fn delete(&self, id: Uuid) -> Result<bool, sqlx::Error>;
}
