use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use chrono::Utc;
use grade_o_matic::{
    common::error::AppError,
    domains::classes::{
        Class, ClassRepositoryTrait, ClassService, ClassServiceTrait, ClassesWithAssignments,
        dto::class_dto::{CreateClassDto, UpdateClassDto},
    },
};
use sqlx::PgPool;
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Default)]
struct FakeClassRepository {
    store: Mutex<HashMap<Uuid, Class>>,
    fail_find_all: bool,
}

#[async_trait]
impl ClassRepositoryTrait for FakeClassRepository {
    fn new(_pool: PgPool) -> Self
    where
        Self: Sized,
    {
        Self::default()
    }

    async fn list(&self) -> Result<Vec<Class>, sqlx::Error> {
        if self.fail_find_all {
            return Err(sqlx::Error::RowNotFound);
        }
        let store = self.store.lock().await;
        Ok(store.values().cloned().collect())
    }

    async fn list_classes_with_assignments(
        &self,
        _owner_id: Uuid,
    ) -> Result<Vec<ClassesWithAssignments>, sqlx::Error> {
        Ok(vec![])
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Class>, sqlx::Error> {
        let store = self.store.lock().await;
        Ok(store.get(&id).cloned())
    }

    async fn create(&self, class: CreateClassDto) -> Result<Uuid, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let entity = Class {
            id,
            title: class.title,
            description: class.description,
            term: class.term,
            owner_id: class.owner_id.or(Some(class.modified_by)),
            created_by: class.modified_by.into(),
            created_at: Some(now),
            modified_by: class.modified_by.into(),
            modified_at: Some(now),
        };

        let mut store = self.store.lock().await;
        store.insert(id, entity);
        Ok(id)
    }

    async fn update(&self, id: Uuid, class: UpdateClassDto) -> Result<Option<Class>, sqlx::Error> {
        let mut store = self.store.lock().await;
        let Some(existing) = store.get_mut(&id) else {
            return Ok(None);
        };

        existing.title = class.title;
        existing.description = class.description;
        existing.term = class.term;
        existing.owner_id = class.owner_id;
        existing.modified_by = class.modified_by.into();
        existing.modified_at = Some(Utc::now());

        Ok(Some(existing.clone()))
    }

    async fn delete(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let mut store = self.store.lock().await;
        Ok(store.remove(&id).is_some())
    }
}

fn seed_class(id: Uuid) -> Class {
    let user_id = Uuid::new_v4();
    Class {
        id,
        title: "class-1".to_string(),
        description: Some("description".to_string()),
        term: Some("Fall 2026".to_string()),
        owner_id: Some(user_id),
        created_by: user_id.into(),
        created_at: Some(Utc::now()),
        modified_by: user_id.into(),
        modified_at: Some(Utc::now()),
    }
}

fn build_service_with_repo(repo: FakeClassRepository) -> ClassService<FakeClassRepository> {
    ClassService::with_repository(Arc::new(repo))
}

#[tokio::test]
async fn list_returns_class_dtos() {
    let id = Uuid::new_v4();
    let mut map = HashMap::new();
    map.insert(id, seed_class(id));
    let repo = FakeClassRepository {
        store: Mutex::new(map),
        fail_find_all: false,
    };
    let service = build_service_with_repo(repo);

    let classes = service.list().await.expect("list should succeed");
    assert_eq!(classes.len(), 1);
    assert_eq!(classes[0].id, id);
    assert_eq!(classes[0].title, "class-1");
}

#[tokio::test]
async fn get_by_id_returns_not_found_error_when_missing() {
    let service = build_service_with_repo(FakeClassRepository::default());
    let err = service
        .find_by_id(Uuid::new_v4())
        .await
        .expect_err("missing class should error");

    assert!(matches!(err, AppError::NotFound(_)));
}

#[tokio::test]
async fn create_persists_and_returns_class() {
    let service = build_service_with_repo(FakeClassRepository::default());
    let modified_by = Uuid::new_v4();
    let payload = CreateClassDto {
        title: "new class".to_string(),
        description: Some("desc".to_string()),
        term: Some("Fall 2026".to_string()),
        owner_id: Some(modified_by),
        modified_by,
    };

    let created = service
        .create(payload)
        .await
        .expect("create should succeed");
    assert_eq!(created.title, "new class");
    assert_eq!(created.description.as_deref(), Some("desc"));
    assert_eq!(created.term.as_deref(), Some("Fall 2026"));
    assert_eq!(created.owner_id, Some(modified_by));
}

#[tokio::test]
async fn update_returns_not_found_error_when_missing() {
    let service = build_service_with_repo(FakeClassRepository::default());
    let payload = UpdateClassDto {
        id: Uuid::new_v4(),
        title: "updated".to_string(),
        description: Some("updated".to_string()),
        term: Some("Spring 2027".to_string()),
        owner_id: Some(Uuid::new_v4()),
        modified_by: Uuid::new_v4(),
    };

    let err = service
        .update(payload)
        .await
        .expect_err("update should fail for missing class");
    assert!(matches!(err, AppError::NotFound(_)));
}

#[tokio::test]
async fn delete_returns_success_message_when_found() {
    let id = Uuid::new_v4();
    let mut map = HashMap::new();
    map.insert(id, seed_class(id));
    let repo = FakeClassRepository {
        store: Mutex::new(map),
        fail_find_all: false,
    };
    let service = build_service_with_repo(repo);

    let result = service.delete(id).await.expect("delete should succeed");
    assert_eq!(result, "Class deleted");
}
