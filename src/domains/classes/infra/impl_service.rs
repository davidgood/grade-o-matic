use crate::common::error::AppError;
use crate::domains::classes::domain::repository::ClassRepositoryTrait;
use crate::domains::classes::domain::service::ClassServiceTrait;
use crate::domains::classes::dto::class_dto::{ClassDto, CreateClassDto, UpdateClassDto};
use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct ClassService<R: ClassRepositoryTrait> {
    repository: Arc<R>,
}

impl<R> ClassService<R>
where
    R: ClassRepositoryTrait,
{
    pub fn with_repository(repository: Arc<R>) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<R> ClassServiceTrait for ClassService<R>
where
    R: ClassRepositoryTrait + 'static,
{
    fn create_class_service(pool: PgPool) -> Arc<dyn ClassServiceTrait> {
        Arc::new(Self {
            repository: Arc::new(R::new(pool)),
        })
    }

    async fn list(&self) -> Result<Vec<ClassDto>, AppError> {
        match self.repository.list().await {
            Ok(classes) => {
                let class_dtos: Vec<ClassDto> = classes.into_iter().map(Into::into).collect();
                Ok(class_dtos)
            }
            Err(err) => {
                tracing::error!("Error fetching classes: {err}");
                Err(AppError::DatabaseError(err))
            }
        }
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<ClassDto>, AppError> {
        match self.repository.find_by_id(id).await {
            Ok(class) => Ok(class.map(Into::into)),
            Err(err) => {
                tracing::error!("Error fetching class: {err}");
                Err(AppError::DatabaseError(err))
            }
        }
    }

    async fn create(&self, class: CreateClassDto) -> Result<ClassDto, AppError> {
        let class_id = match self.repository.create(class).await {
            Ok(class_id) => class_id,
            Err(err) => {
                tracing::error!("Error creating class: {err}");
                return Err(AppError::DatabaseError(err));
            }
        };

        match self.repository.find_by_id(class_id).await {
            Ok(Some(class)) => Ok(ClassDto::from(class)),
            Ok(None) => Err(AppError::NotFound("Class not found".into())),
            Err(err) => {
                tracing::error!("Error fetching class: {err}");
                Err(AppError::DatabaseError(err))
            }
        }
    }

    async fn update(&self, class: UpdateClassDto) -> Result<Option<ClassDto>, AppError> {
        match self.repository.update(class.id, class).await {
            Ok(Some(class)) => Ok(Option::from(ClassDto::from(class))),
            Ok(None) => Err(AppError::NotFound("Class not found".into())),
            Err(err) => {
                tracing::error!("Error updating class: {err}");
                Err(AppError::DatabaseError(err))
            }
        }
    }

    async fn delete(&self, id: Uuid) -> Result<String, AppError> {
        match self.repository.delete(id).await {
            Ok(true) => Ok("Class deleted".into()),
            Ok(false) => Err(AppError::NotFound("Class not found".into())),
            Err(err) => {
                tracing::error!("Error deleting class: {err}");
                Err(AppError::DatabaseError(err))
            }
        }
    }
}
