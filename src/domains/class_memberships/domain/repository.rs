use crate::domains::class_memberships::domain::model::ClassMembership;
use crate::domains::class_memberships::dto::class_membership_dto::{
    CreateClassMembershipDto, UpdateClassMembershipDto,
};
use async_trait::async_trait;
use sqlx::PgPool;

#[async_trait]
pub trait ClassMembershipRepositoryTrait: Send + Sync {
    fn new(pool: PgPool) -> Self
    where
        Self: Sized;

    async fn list_by_class_id(
        &self,
        class_id: uuid::Uuid,
    ) -> Result<Vec<ClassMembership>, sqlx::Error>;
    async fn list_by_user_id(
        &self,
        user_id: uuid::Uuid,
    ) -> Result<Vec<ClassMembership>, sqlx::Error>;
    async fn find_by_id(&self, id: uuid::Uuid) -> Result<Option<ClassMembership>, sqlx::Error>;
    async fn create(&self, membership: CreateClassMembershipDto)
    -> Result<uuid::Uuid, sqlx::Error>;
    async fn update(
        &self,
        id: uuid::Uuid,
        membership: UpdateClassMembershipDto,
    ) -> Result<Option<ClassMembership>, sqlx::Error>;
    async fn delete(&self, id: uuid::Uuid) -> Result<bool, sqlx::Error>;
}
