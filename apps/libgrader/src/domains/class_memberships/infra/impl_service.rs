use crate::common::error::AppError;
use crate::domains::class_memberships::domain::repository::ClassMembershipRepositoryTrait;
use crate::domains::class_memberships::domain::service::ClassMembershipServiceTrait;
use crate::domains::class_memberships::dto::class_membership_dto::{
    ClassMembershipDto, CreateClassMembershipDto, UpdateClassMembershipDto,
};
use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct ClassMembershipService<R: ClassMembershipRepositoryTrait> {
    repository: Arc<R>,
}

impl<R> ClassMembershipService<R>
where
    R: ClassMembershipRepositoryTrait,
{
    pub fn with_repository(repository: Arc<R>) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<R> ClassMembershipServiceTrait for ClassMembershipService<R>
where
    R: ClassMembershipRepositoryTrait + 'static,
{
    fn create_service(pool: PgPool) -> Arc<dyn ClassMembershipServiceTrait> {
        Arc::new(Self {
            repository: Arc::new(R::new(pool)),
        })
    }

    async fn list_by_class_id(&self, class_id: Uuid) -> Result<Vec<ClassMembershipDto>, AppError> {
        match self.repository.list_by_class_id(class_id).await {
            Ok(memberships) => Ok(memberships.into_iter().map(Into::into).collect()),
            Err(err) => {
                tracing::error!("Error listing class memberships by class id: {err}");
                Err(AppError::DatabaseError(err))
            }
        }
    }

    async fn list_by_user_id(&self, user_id: Uuid) -> Result<Vec<ClassMembershipDto>, AppError> {
        match self.repository.list_by_user_id(user_id).await {
            Ok(memberships) => Ok(memberships.into_iter().map(Into::into).collect()),
            Err(err) => {
                tracing::error!("Error listing class memberships by user id: {err}");
                Err(AppError::DatabaseError(err))
            }
        }
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<ClassMembershipDto>, AppError> {
        match self.repository.find_by_id(id).await {
            Ok(Some(membership)) => Ok(Some(membership.into())),
            Ok(None) => Ok(None),
            Err(err) => {
                tracing::error!("Error finding class membership: {err}");
                Err(AppError::DatabaseError(err))
            }
        }
    }

    async fn create(
        &self,
        membership: CreateClassMembershipDto,
    ) -> Result<ClassMembershipDto, AppError> {
        let membership_id = match self.repository.create(membership).await {
            Ok(id) => id,
            Err(err) => {
                tracing::error!("Error creating class membership: {err}");
                return Err(AppError::DatabaseError(err));
            }
        };

        match self.repository.find_by_id(membership_id).await {
            Ok(Some(created)) => Ok(created.into()),
            Ok(None) => Err(AppError::NotFound("Class membership not found".to_string())),
            Err(err) => {
                tracing::error!("Error loading class membership after create: {err}");
                Err(AppError::DatabaseError(err))
            }
        }
    }

    async fn update(
        &self,
        membership: UpdateClassMembershipDto,
    ) -> Result<Option<ClassMembershipDto>, AppError> {
        match self.repository.update(membership.id, membership).await {
            Ok(Some(updated)) => Ok(Some(updated.into())),
            Ok(None) => Ok(None),
            Err(err) => {
                tracing::error!("Error updating class membership: {err}");
                Err(AppError::DatabaseError(err))
            }
        }
    }

    async fn delete(&self, id: Uuid) -> Result<String, AppError> {
        match self.repository.delete(id).await {
            Ok(true) => Ok("Class membership deleted".to_string()),
            Ok(false) => Err(AppError::NotFound("Class membership not found".to_string())),
            Err(err) => {
                tracing::error!("Error deleting class membership: {err}");
                Err(AppError::DatabaseError(err))
            }
        }
    }
}
