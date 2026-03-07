use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use chrono::Utc;
use grade_o_matic_web::{
    common::error::AppError,
    domains::classes::{
        Class, ClassRepositoryTrait, ClassService, ClassServiceTrait, ClassesWithAssignments,
        create_class_service,
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
    fail_list_with_assignments: bool,
    list_with_assignments: Vec<ClassesWithAssignments>,
    fail_create: bool,
    create_without_store: bool,
    fail_find_by_id_error: bool,
    fail_update: bool,
    fail_delete: bool,
    force_delete_false: bool,
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
        if self.fail_list_with_assignments {
            return Err(sqlx::Error::RowNotFound);
        }
        Ok(self.list_with_assignments.clone())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Class>, sqlx::Error> {
        if self.fail_find_by_id_error {
            return Err(sqlx::Error::RowNotFound);
        }
        let store = self.store.lock().await;
        Ok(store.get(&id).cloned())
    }

    async fn create(&self, class: CreateClassDto) -> Result<Uuid, sqlx::Error> {
        if self.fail_create {
            return Err(sqlx::Error::RowNotFound);
        }
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

        if !self.create_without_store {
            let mut store = self.store.lock().await;
            store.insert(id, entity);
        }
        Ok(id)
    }

    async fn update(&self, id: Uuid, class: UpdateClassDto) -> Result<Option<Class>, sqlx::Error> {
        if self.fail_update {
            return Err(sqlx::Error::RowNotFound);
        }
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
        if self.fail_delete {
            return Err(sqlx::Error::RowNotFound);
        }
        if self.force_delete_false {
            return Ok(false);
        }
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
        ..Default::default()
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
        ..Default::default()
    };
    let service = build_service_with_repo(repo);

    let result = service.delete(id).await.expect("delete should succeed");
    assert_eq!(result, "Class deleted");
}

#[tokio::test]
async fn list_returns_database_error_when_repo_fails() {
    let service = build_service_with_repo(FakeClassRepository {
        fail_find_all: true,
        ..Default::default()
    });

    let err = service.list().await.expect_err("list should fail");
    assert!(matches!(err, AppError::DatabaseError(_)));
}

#[tokio::test]
async fn list_classes_with_assignments_returns_rows() {
    let owner = Uuid::new_v4();
    let class_id = Uuid::new_v4();
    let assignment_id = Uuid::new_v4();
    let service = build_service_with_repo(FakeClassRepository {
        list_with_assignments: vec![ClassesWithAssignments {
            class_id,
            class_title: "Algorithms".to_string(),
            class_term: Some("Fall 2026".to_string()),
            assignment_id: Some(assignment_id),
            assignment_title: Some("Homework 1".to_string()),
            assignment_description: Some("Graph traversals".to_string()),
            due_at: None,
            points: Some(10),
        }],
        ..Default::default()
    });

    let rows = service
        .list_classes_with_assignments(owner)
        .await
        .expect("rows should map");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].class_id, class_id);
    assert_eq!(rows[0].assignment_id, Some(assignment_id));
}

#[tokio::test]
async fn list_classes_with_assignments_returns_database_error_when_repo_fails() {
    let service = build_service_with_repo(FakeClassRepository {
        fail_list_with_assignments: true,
        ..Default::default()
    });
    let err = service
        .list_classes_with_assignments(Uuid::new_v4())
        .await
        .expect_err("should fail");
    assert!(matches!(err, AppError::DatabaseError(_)));
}

#[tokio::test]
async fn create_returns_database_error_when_repo_create_fails() {
    let service = build_service_with_repo(FakeClassRepository {
        fail_create: true,
        ..Default::default()
    });

    let err = service
        .create(CreateClassDto {
            title: "new class".to_string(),
            description: None,
            term: None,
            owner_id: None,
            modified_by: Uuid::new_v4(),
        })
        .await
        .expect_err("create should fail");
    assert!(matches!(err, AppError::DatabaseError(_)));
}

#[tokio::test]
async fn create_returns_not_found_when_created_row_cannot_be_loaded() {
    let service = build_service_with_repo(FakeClassRepository {
        create_without_store: true,
        ..Default::default()
    });

    let err = service
        .create(CreateClassDto {
            title: "new class".to_string(),
            description: None,
            term: None,
            owner_id: None,
            modified_by: Uuid::new_v4(),
        })
        .await
        .expect_err("create should fail when class cannot be fetched");
    assert!(matches!(err, AppError::NotFound(_)));
}

#[tokio::test]
async fn find_by_id_returns_database_error_when_repo_fails() {
    let service = build_service_with_repo(FakeClassRepository {
        fail_find_by_id_error: true,
        ..Default::default()
    });

    let err = service
        .find_by_id(Uuid::new_v4())
        .await
        .expect_err("find should fail");
    assert!(matches!(err, AppError::DatabaseError(_)));
}

#[tokio::test]
async fn update_returns_database_error_when_repo_fails() {
    let service = build_service_with_repo(FakeClassRepository {
        fail_update: true,
        ..Default::default()
    });
    let err = service
        .update(UpdateClassDto {
            id: Uuid::new_v4(),
            title: "updated".to_string(),
            description: None,
            term: None,
            owner_id: None,
            modified_by: Uuid::new_v4(),
        })
        .await
        .expect_err("update should fail");
    assert!(matches!(err, AppError::DatabaseError(_)));
}

#[tokio::test]
async fn delete_returns_not_found_when_repo_reports_false() {
    let service = build_service_with_repo(FakeClassRepository {
        force_delete_false: true,
        ..Default::default()
    });
    let err = service
        .delete(Uuid::new_v4())
        .await
        .expect_err("delete should fail");
    assert!(matches!(err, AppError::NotFound(_)));
}

#[tokio::test]
async fn delete_returns_database_error_when_repo_fails() {
    let service = build_service_with_repo(FakeClassRepository {
        fail_delete: true,
        ..Default::default()
    });
    let err = service
        .delete(Uuid::new_v4())
        .await
        .expect_err("delete should fail");
    assert!(matches!(err, AppError::DatabaseError(_)));
}

fn unique_username(prefix: &str) -> String {
    format!("{prefix}_{}", Uuid::new_v4().simple())
}

async fn maybe_setup_db() -> Option<PgPool> {
    match std::env::var("RUN_DB_INTEGRATION_TESTS").ok().as_deref() {
        Some("1") => Some(
            crate::test_helpers::setup_test_db()
                .await
                .expect("setup db"),
        ),
        _ => None,
    }
}

async fn insert_user(pool: &PgPool, username: &str, role: &str) -> Uuid {
    let id = Uuid::new_v4();
    let email = format!("{username}@example.test");
    sqlx::query(
        r#"
        INSERT INTO users (id, username, email, user_role, created_by, modified_by)
        VALUES ($1, $2, $3, $4, NULL, NULL)
        "#,
    )
    .bind(id)
    .bind(username)
    .bind(email)
    .bind(role)
    .execute(pool)
    .await
    .expect("insert user");
    id
}

async fn insert_class(
    pool: &PgPool,
    owner_id: Uuid,
    title: &str,
    term: Option<&str>,
    description: Option<&str>,
) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO classes (id, title, description, term, owner_id, created_by, modified_by)
        VALUES ($1, $2, $3, $4, $5, $5, $5)
        "#,
    )
    .bind(id)
    .bind(title)
    .bind(description)
    .bind(term)
    .bind(owner_id)
    .execute(pool)
    .await
    .expect("insert class");
    id
}

async fn insert_assignment(
    pool: &PgPool,
    class_id: Uuid,
    modified_by: Uuid,
    title: &str,
    points: Option<i16>,
) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO assignments (id, class_id, title, description, due_at, points, created_by, modified_by)
        VALUES ($1, $2, $3, 'db test assignment', NOW() + interval '1 day', $4, $5, $5)
        "#,
    )
    .bind(id)
    .bind(class_id)
    .bind(title)
    .bind(points)
    .bind(modified_by)
    .execute(pool)
    .await
    .expect("insert assignment");
    id
}

#[tokio::test]
async fn db_list_classes_with_assignments_filters_by_owner() {
    let Some(pool) = maybe_setup_db().await else {
        return;
    };
    let service = create_class_service(pool.clone());

    let owner_id = insert_user(&pool, &unique_username("owner_instructor"), "instructor").await;
    let other_owner_id =
        insert_user(&pool, &unique_username("other_instructor"), "instructor").await;

    let class_with_assignment = insert_class(
        &pool,
        owner_id,
        "Owned With Assignment",
        Some("Spring 2026"),
        Some("owner class with assignment"),
    )
    .await;
    let class_without_assignment = insert_class(
        &pool,
        owner_id,
        "Owned Empty",
        Some("Spring 2026"),
        Some("owner class without assignment"),
    )
    .await;
    let _other_class = insert_class(
        &pool,
        other_owner_id,
        "Other Owner Class",
        Some("Spring 2026"),
        Some("should not be visible"),
    )
    .await;
    let _assignment_id = insert_assignment(
        &pool,
        class_with_assignment,
        owner_id,
        "Owner Assignment",
        Some(25),
    )
    .await;

    let rows = service
        .list_classes_with_assignments(owner_id)
        .await
        .expect("list classes with assignments");

    assert_eq!(rows.len(), 2);
    assert!(rows.iter().any(|r| {
        r.class_id == class_with_assignment
            && r.assignment_title.as_deref() == Some("Owner Assignment")
            && r.points == Some(25)
    }));
    assert!(rows.iter().any(|r| {
        r.class_id == class_without_assignment
            && r.assignment_id.is_none()
            && r.assignment_title.is_none()
    }));
}

#[tokio::test]
async fn db_create_update_and_delete_class_happy_path() {
    let Some(pool) = maybe_setup_db().await else {
        return;
    };
    let service = create_class_service(pool.clone());
    let instructor_id = insert_user(&pool, &unique_username("crud_instructor"), "instructor").await;

    let created = service
        .create(CreateClassDto {
            title: "Databases 101".to_string(),
            description: Some("Intro to relational design".to_string()),
            term: Some("Fall 2026".to_string()),
            owner_id: Some(instructor_id),
            modified_by: instructor_id,
        })
        .await
        .expect("create class");

    assert_eq!(created.title, "Databases 101");
    assert_eq!(created.owner_id, Some(instructor_id));

    let updated = service
        .update(UpdateClassDto {
            id: created.id,
            title: "Databases 201".to_string(),
            description: Some("Advanced relational systems".to_string()),
            term: Some("Spring 2027".to_string()),
            owner_id: Some(instructor_id),
            modified_by: instructor_id,
        })
        .await
        .expect("update class")
        .expect("updated class");

    assert_eq!(updated.title, "Databases 201");
    assert_eq!(
        updated.description.as_deref(),
        Some("Advanced relational systems")
    );
    assert_eq!(updated.term.as_deref(), Some("Spring 2027"));

    let deleted = service.delete(created.id).await.expect("delete class");
    assert_eq!(deleted, "Class deleted");

    let err = service
        .find_by_id(created.id)
        .await
        .expect_err("deleted class should not be found");
    assert!(matches!(err, AppError::NotFound(_)));
}

#[tokio::test]
async fn db_delete_returns_not_found_for_missing_class() {
    let Some(pool) = maybe_setup_db().await else {
        return;
    };
    let service = create_class_service(pool);

    let err = service
        .delete(Uuid::new_v4())
        .await
        .expect_err("missing class delete should fail");
    assert!(matches!(err, AppError::NotFound(_)));
}
