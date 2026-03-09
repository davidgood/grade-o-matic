use crate::common::error::AppError;
use crate::domains::class_memberships::dto::class_membership_dto::{
    ClassMembershipDto, CreateClassMembershipDto, UpdateClassMembershipDto,
};
use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;

#[async_trait]
pub trait ClassMembershipServiceTrait: Send + Sync {
    fn create_service(pool: PgPool) -> Arc<dyn ClassMembershipServiceTrait>
    where
        Self: Sized;

    async fn list_by_class_id(
        &self,
        class_id: uuid::Uuid,
    ) -> Result<Vec<ClassMembershipDto>, AppError>;
    async fn list_by_user_id(
        &self,
        user_id: uuid::Uuid,
    ) -> Result<Vec<ClassMembershipDto>, AppError>;
    async fn find_by_id(&self, id: uuid::Uuid) -> Result<Option<ClassMembershipDto>, AppError>;
    async fn create(
        &self,
        membership: CreateClassMembershipDto,
    ) -> Result<ClassMembershipDto, AppError>;
    async fn update(
        &self,
        membership: UpdateClassMembershipDto,
    ) -> Result<Option<ClassMembershipDto>, AppError>;
    async fn delete(&self, id: uuid::Uuid) -> Result<String, AppError>;
}
