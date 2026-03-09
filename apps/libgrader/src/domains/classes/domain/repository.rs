use crate::domains::classes::domain::model::{Class, ClassesWithAssignments};
use crate::domains::classes::dto::class_dto::{CreateClassDto, UpdateClassDto};
use async_trait::async_trait;
use sqlx::PgPool;

#[async_trait]
pub trait ClassRepositoryTrait: Send + Sync {
    fn new(pool: PgPool) -> Self
    where
        Self: Sized;

    async fn list(&self) -> Result<Vec<Class>, sqlx::Error>;

    async fn list_classes_with_assignments(
        &self,
        owner_id: uuid::Uuid,
    ) -> Result<Vec<ClassesWithAssignments>, sqlx::Error>;

    async fn find_by_id(&self, id: uuid::Uuid) -> Result<Option<Class>, sqlx::Error>;

    async fn create(&self, class: CreateClassDto) -> Result<uuid::Uuid, sqlx::Error>;

    async fn update(
        &self,
        id: uuid::Uuid,
        class: UpdateClassDto,
    ) -> Result<Option<Class>, sqlx::Error>;

    async fn delete(&self, id: uuid::Uuid) -> Result<bool, sqlx::Error>;
}
