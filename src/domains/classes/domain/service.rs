use crate::common::error::AppError;
use crate::domains::classes::dto::class_dto::{ClassDto, CreateClassDto, UpdateClassDto};
use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;

#[async_trait]
pub trait ClassServiceTrait: Send + Sync {
    fn create_class_service(pool: PgPool) -> Arc<dyn ClassServiceTrait>
    where
        Self: Sized;

    async fn list(&self) -> Result<Vec<ClassDto>, AppError>;
    async fn find_by_id(&self, id: uuid::Uuid) -> Result<Option<ClassDto>, AppError>;
    async fn create(&self, class: CreateClassDto) -> Result<ClassDto, AppError>;
    async fn update(&self, class: UpdateClassDto) -> Result<Option<ClassDto>, AppError>;
    async fn delete(&self, id: uuid::Uuid) -> Result<String, AppError>;
}
