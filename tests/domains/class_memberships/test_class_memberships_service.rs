use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use chrono::Utc;
use grade_o_matic::{
    common::error::AppError,
    domains::class_memberships::{
        ClassMembership, ClassMembershipRepositoryTrait, ClassMembershipRole,
        ClassMembershipService, ClassMembershipServiceTrait,
        dto::class_membership_dto::{CreateClassMembershipDto, UpdateClassMembershipDto},
    },
};
use sqlx::PgPool;
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Default)]
struct FakeClassMembershipRepository {
    store: Mutex<HashMap<Uuid, ClassMembership>>,
}

#[async_trait]
impl ClassMembershipRepositoryTrait for FakeClassMembershipRepository {
    fn new(_pool: PgPool) -> Self
    where
        Self: Sized,
    {
        Self::default()
    }

    async fn list_by_class_id(&self, class_id: Uuid) -> Result<Vec<ClassMembership>, sqlx::Error> {
        let store = self.store.lock().await;
        Ok(store
            .values()
            .filter(|m| m.class_id == class_id)
            .cloned()
            .collect())
    }

    async fn list_by_user_id(&self, user_id: Uuid) -> Result<Vec<ClassMembership>, sqlx::Error> {
        let store = self.store.lock().await;
        Ok(store
            .values()
            .filter(|m| m.user_id == user_id)
            .cloned()
            .collect())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<ClassMembership>, sqlx::Error> {
        let store = self.store.lock().await;
        Ok(store.get(&id).cloned())
    }

    async fn create(&self, membership: CreateClassMembershipDto) -> Result<Uuid, sqlx::Error> {
        let id = Uuid::new_v4();
        let entity = ClassMembership {
            id,
            class_id: membership.class_id,
            user_id: membership.user_id,
            role: membership.role,
            created_at: Some(Utc::now()),
            modified_at: Some(Utc::now()),
        };
        let mut store = self.store.lock().await;
        store.insert(id, entity);
        Ok(id)
    }

    async fn update(
        &self,
        id: Uuid,
        membership: UpdateClassMembershipDto,
    ) -> Result<Option<ClassMembership>, sqlx::Error> {
        let mut store = self.store.lock().await;
        let Some(existing) = store.get_mut(&id) else {
            return Ok(None);
        };
        existing.role = membership.role;
        existing.modified_at = Some(Utc::now());
        Ok(Some(existing.clone()))
    }

    async fn delete(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let mut store = self.store.lock().await;
        Ok(store.remove(&id).is_some())
    }
}

fn seed_membership(id: Uuid, class_id: Uuid, user_id: Uuid) -> ClassMembership {
    ClassMembership {
        id,
        class_id,
        user_id,
        role: ClassMembershipRole::Student,
        created_at: Some(Utc::now()),
        modified_at: Some(Utc::now()),
    }
}

fn build_service_with_repo(
    repo: FakeClassMembershipRepository,
) -> ClassMembershipService<FakeClassMembershipRepository> {
    ClassMembershipService::with_repository(Arc::new(repo))
}

#[tokio::test]
async fn list_by_class_id_returns_memberships() {
    let class_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let id = Uuid::new_v4();
    let mut map = HashMap::new();
    map.insert(id, seed_membership(id, class_id, user_id));
    let repo = FakeClassMembershipRepository {
        store: Mutex::new(map),
    };
    let service = build_service_with_repo(repo);

    let memberships = service
        .list_by_class_id(class_id)
        .await
        .expect("list should succeed");
    assert_eq!(memberships.len(), 1);
    assert_eq!(memberships[0].id, id);
    assert!(matches!(memberships[0].role, ClassMembershipRole::Student));
}

#[tokio::test]
async fn create_persists_and_returns_membership() {
    let service = build_service_with_repo(FakeClassMembershipRepository::default());
    let payload = CreateClassMembershipDto {
        class_id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        role: ClassMembershipRole::Ta,
    };

    let created = service
        .create(payload)
        .await
        .expect("create should succeed");
    assert!(matches!(created.role, ClassMembershipRole::Ta));
}

#[tokio::test]
async fn update_returns_none_when_membership_missing() {
    let service = build_service_with_repo(FakeClassMembershipRepository::default());
    let payload = UpdateClassMembershipDto {
        id: Uuid::new_v4(),
        role: ClassMembershipRole::Student,
    };

    let result = service
        .update(payload)
        .await
        .expect("update should not error");
    assert!(result.is_none());
}

#[tokio::test]
async fn create_payload_deserializes_lowercase_role_and_rejects_invalid_role() {
    let valid_payload: CreateClassMembershipDto = serde_json::from_value(serde_json::json!({
        "class_id": Uuid::new_v4(),
        "user_id": Uuid::new_v4(),
        "role": "ta"
    }))
    .expect("valid role should deserialize");

    assert!(matches!(valid_payload.role, ClassMembershipRole::Ta));

    let invalid_payload: Result<CreateClassMembershipDto, _> =
        serde_json::from_value(serde_json::json!({
            "class_id": Uuid::new_v4(),
            "user_id": Uuid::new_v4(),
            "role": "instructor"
        }));

    assert!(invalid_payload.is_err());
}

#[tokio::test]
async fn delete_returns_not_found_error_when_missing() {
    let service = build_service_with_repo(FakeClassMembershipRepository::default());
    let err = service
        .delete(Uuid::new_v4())
        .await
        .expect_err("delete should return not found for missing id");
    assert!(matches!(err, AppError::NotFound(_)));
}
