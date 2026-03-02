use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use chrono::Utc;
use grade_o_matic::{
    common::error::AppError,
    domains::assignments::{
        Assignment, AssignmentRepositoryTrait, AssignmentService, AssignmentServiceTrait,
        dto::assignment_dto::{CreateAssignmentDto, UpdateAssignmentDto},
    },
};
use sqlx::PgPool;
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Default)]
struct FakeAssignmentRepository {
    store: Mutex<HashMap<Uuid, Assignment>>,
    fail_find_all: bool,
}

#[async_trait]
impl AssignmentRepositoryTrait for FakeAssignmentRepository {
    fn new(_pool: PgPool) -> Self
    where
        Self: Sized,
    {
        Self::default()
    }

    async fn find_all(&self) -> Result<Vec<Assignment>, sqlx::Error> {
        if self.fail_find_all {
            return Err(sqlx::Error::RowNotFound);
        }
        let store = self.store.lock().await;
        Ok(store.values().cloned().collect())
    }

    async fn find_by_class_id(&self, class_id: Uuid) -> Result<Vec<Assignment>, sqlx::Error> {
        let store = self.store.lock().await;
        Ok(store
            .values()
            .filter(|assignment| assignment.class_id == class_id)
            .cloned()
            .collect())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Assignment>, sqlx::Error> {
        let store = self.store.lock().await;
        Ok(store.get(&id).cloned())
    }

    async fn create(&self, assignment: CreateAssignmentDto) -> Result<Uuid, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let entity = Assignment {
            id,
            class_id: assignment.class_id,
            title: assignment.title,
            description: assignment.description,
            due_at: assignment.due_at,
            created_by: Some(assignment.modified_by),
            created_at: Some(now),
            modified_by: Some(assignment.modified_by),
            modified_at: Some(now),
        };

        let mut store = self.store.lock().await;
        store.insert(id, entity);
        Ok(id)
    }

    async fn update(
        &self,
        id: Uuid,
        assignment: UpdateAssignmentDto,
    ) -> Result<Option<Assignment>, sqlx::Error> {
        let mut store = self.store.lock().await;
        let Some(existing) = store.get_mut(&id) else {
            return Ok(None);
        };

        existing.class_id = assignment.class_id;
        existing.title = assignment.title;
        existing.description = assignment.description;
        existing.due_at = assignment.due_at;
        existing.modified_by = Some(assignment.modified_by);
        existing.modified_at = Some(Utc::now());

        Ok(Some(existing.clone()))
    }

    async fn delete(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let mut store = self.store.lock().await;
        Ok(store.remove(&id).is_some())
    }
}

fn seed_assignment(id: Uuid) -> Assignment {
    let user_id = Uuid::new_v4();
    Assignment {
        id,
        class_id: Uuid::new_v4(),
        title: "assignment-1".to_string(),
        description: Some("description".to_string()),
        due_at: Some(Utc::now()),
        created_by: Some(user_id),
        created_at: Some(Utc::now()),
        modified_by: Some(user_id),
        modified_at: Some(Utc::now()),
    }
}

fn build_service_with_repo(
    repo: FakeAssignmentRepository,
) -> AssignmentService<FakeAssignmentRepository> {
    AssignmentService::with_repository(Arc::new(repo))
}

#[tokio::test]
async fn list_returns_assignment_dtos() {
    let id = Uuid::new_v4();
    let mut map = HashMap::new();
    map.insert(id, seed_assignment(id));
    let repo = FakeAssignmentRepository {
        store: Mutex::new(map),
        fail_find_all: false,
    };
    let service = build_service_with_repo(repo);

    let assignments = service.list().await.expect("list should succeed");
    assert_eq!(assignments.len(), 1);
    assert_eq!(assignments[0].id, id);
    assert_eq!(assignments[0].title, "assignment-1");
}

#[tokio::test]
async fn get_by_id_returns_not_found_error_when_missing() {
    let service = build_service_with_repo(FakeAssignmentRepository::default());
    let err = service
        .get_by_id(Uuid::new_v4())
        .await
        .expect_err("missing assignment should error");

    assert!(matches!(err, AppError::NotFound(_)));
}

#[tokio::test]
async fn create_persists_and_returns_assignment() {
    let service = build_service_with_repo(FakeAssignmentRepository::default());
    let modified_by = Uuid::new_v4();
    let payload = CreateAssignmentDto {
        class_id: Uuid::new_v4(),
        title: "new assignment".to_string(),
        description: Some("desc".to_string()),
        due_at: Some(Utc::now()),
        modified_by,
    };

    let created = service
        .create(payload)
        .await
        .expect("create should succeed");
    assert_eq!(created.title, "new assignment");
    assert_eq!(created.description.as_deref(), Some("desc"));
}

#[tokio::test]
async fn update_returns_not_found_error_when_missing() {
    let service = build_service_with_repo(FakeAssignmentRepository::default());
    let payload = UpdateAssignmentDto {
        id: Uuid::new_v4(),
        class_id: Uuid::new_v4(),
        title: "updated".to_string(),
        description: Some("updated".to_string()),
        due_at: Some(Utc::now()),
        modified_by: Uuid::new_v4(),
    };

    let err = service
        .update(payload)
        .await
        .expect_err("update should fail for missing assignment");
    assert!(matches!(err, AppError::NotFound(_)));
}

#[tokio::test]
async fn delete_returns_success_message_when_found() {
    let id = Uuid::new_v4();
    let mut map = HashMap::new();
    map.insert(id, seed_assignment(id));
    let repo = FakeAssignmentRepository {
        store: Mutex::new(map),
        fail_find_all: false,
    };
    let service = build_service_with_repo(repo);

    let result = service.delete(id).await.expect("delete should succeed");
    assert_eq!(result, "Assignment deleted");
}
