use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use chrono::Utc;
use grade_o_matic::{
    common::error::AppError,
    domains::assignments::{
        Assignment, AssignmentAttachment, AssignmentRepositoryTrait, AssignmentService,
        AssignmentServiceTrait, AssignmentWithAttachmentCount, create_assignment_service,
        dto::assignment_dto::{CreateAssignmentDto, UpdateAssignmentDto},
    },
};
use sqlx::{Error, PgPool};
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Default)]
struct FakeAssignmentRepository {
    store: Mutex<HashMap<Uuid, Assignment>>,
    fail_find_all: bool,
    fail_find_by_class_id: bool,
    fail_find_by_class_id_with_count: bool,
    list_attachments_result: Vec<AssignmentAttachment>,
    fail_list_attachments: bool,
    fail_add_attachment: bool,
    fail_remove_attachment: bool,
    remove_attachment_result: bool,
    fail_create: bool,
    create_without_store: bool,
    fail_find_by_id_error: bool,
    fail_update: bool,
    fail_delete: bool,
    force_delete_false: bool,
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
        if self.fail_find_by_class_id {
            return Err(sqlx::Error::RowNotFound);
        }
        let store = self.store.lock().await;
        Ok(store
            .values()
            .filter(|assignment| assignment.class_id == class_id)
            .cloned()
            .collect())
    }

    async fn find_by_class_id_with_attachment_count(
        &self,
        class_id: Uuid,
    ) -> Result<Vec<AssignmentWithAttachmentCount>, Error> {
        if self.fail_find_by_class_id_with_count {
            return Err(sqlx::Error::RowNotFound);
        }
        let store = self.store.lock().await;
        Ok(store
            .values()
            .filter(|assignment| assignment.class_id == class_id)
            .cloned()
            .map(|assignment| AssignmentWithAttachmentCount {
                id: assignment.id,
                class_id: assignment.class_id,
                title: assignment.title,
                description: assignment.description,
                due_at: assignment.due_at,
                points: assignment.points,
                created_by: assignment.created_by,
                created_at: assignment.created_at,
                modified_by: assignment.modified_by,
                modified_at: assignment.modified_at,
                attachment_count: 0,
            })
            .collect())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Assignment>, sqlx::Error> {
        if self.fail_find_by_id_error {
            return Err(sqlx::Error::RowNotFound);
        }
        let store = self.store.lock().await;
        Ok(store.get(&id).cloned())
    }

    async fn list_attachments(
        &self,
        _assignment_id: Uuid,
    ) -> Result<Vec<AssignmentAttachment>, sqlx::Error> {
        if self.fail_list_attachments {
            return Err(sqlx::Error::RowNotFound);
        }
        Ok(self.list_attachments_result.clone())
    }

    async fn add_attachment(
        &self,
        _assignment_id: Uuid,
        _file_id: Uuid,
        _created_by: Uuid,
    ) -> Result<(), sqlx::Error> {
        if self.fail_add_attachment {
            return Err(sqlx::Error::RowNotFound);
        }
        Ok(())
    }

    async fn remove_attachment(
        &self,
        _assignment_id: Uuid,
        _file_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        if self.fail_remove_attachment {
            return Err(sqlx::Error::RowNotFound);
        }
        Ok(self.remove_attachment_result)
    }

    async fn create(&self, assignment: CreateAssignmentDto) -> Result<Uuid, sqlx::Error> {
        if self.fail_create {
            return Err(sqlx::Error::RowNotFound);
        }
        let id = Uuid::new_v4();
        let now = Utc::now();
        let entity = Assignment {
            id,
            class_id: assignment.class_id,
            title: assignment.title,
            description: assignment.description,
            due_at: assignment.due_at,
            points: Some(100),
            created_by: Some(assignment.modified_by),
            created_at: Some(now),
            modified_by: Some(assignment.modified_by),
            modified_at: Some(now),
        };

        if !self.create_without_store {
            let mut store = self.store.lock().await;
            store.insert(id, entity);
        }
        Ok(id)
    }

    async fn update(
        &self,
        id: Uuid,
        assignment: UpdateAssignmentDto,
    ) -> Result<Option<Assignment>, sqlx::Error> {
        if self.fail_update {
            return Err(sqlx::Error::RowNotFound);
        }
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

fn seed_assignment(id: Uuid) -> Assignment {
    let user_id = Uuid::new_v4();
    Assignment {
        id,
        class_id: Uuid::new_v4(),
        title: "assignment-1".to_string(),
        description: Some("description".to_string()),
        due_at: Some(Utc::now()),
        points: Some(100),
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
        ..Default::default()
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
        .find_by_id(Uuid::new_v4())
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
        points: Some(100),
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
        points: Some(100),
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
        ..Default::default()
    };
    let service = build_service_with_repo(repo);

    let result = service.delete(id).await.expect("delete should succeed");
    assert_eq!(result, "Assignment deleted");
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

async fn insert_class(pool: &PgPool, owner_id: Uuid, title: &str) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO classes (id, title, description, term, owner_id, created_by, modified_by)
        VALUES ($1, $2, 'db class', 'Spring 2026', $3, $3, $3)
        "#,
    )
    .bind(id)
    .bind(title)
    .bind(owner_id)
    .execute(pool)
    .await
    .expect("insert class");
    id
}

async fn insert_uploaded_file(pool: &PgPool, user_id: Uuid, name: &str) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO uploaded_files (
            id, user_id, file_name, origin_file_name, file_relative_path,
            file_url, content_type, file_size, file_type, created_by, modified_by
        )
        VALUES ($1, $2, $3, $4, $5, $6, 'application/pdf', 1024, 'document', $2, $2)
        "#,
    )
    .bind(id)
    .bind(user_id)
    .bind(name)
    .bind(name)
    .bind(format!("assignment_uploads/{name}"))
    .bind(format!("https://cdn.example.test/{name}"))
    .execute(pool)
    .await
    .expect("insert uploaded file");
    id
}

#[tokio::test]
async fn db_create_update_and_delete_assignment_happy_path() {
    let Some(pool) = maybe_setup_db().await else {
        return;
    };
    let service = create_assignment_service(pool.clone());
    let instructor_id = insert_user(
        &pool,
        &unique_username("assignment_instructor"),
        "instructor",
    )
    .await;
    let class_id = insert_class(&pool, instructor_id, "Service Class").await;

    let created = service
        .create(CreateAssignmentDto {
            class_id,
            title: "Project 1".to_string(),
            description: Some("Implement parser".to_string()),
            due_at: Some(Utc::now() + chrono::Duration::days(7)),
            points: Some(100),
            modified_by: instructor_id,
        })
        .await
        .expect("create assignment");
    assert_eq!(created.title, "Project 1");
    assert_eq!(created.points, Some(100));

    let updated = service
        .update(UpdateAssignmentDto {
            id: created.id,
            class_id,
            title: "Project 1 Revised".to_string(),
            description: Some("Implement parser + tests".to_string()),
            due_at: Some(Utc::now() + chrono::Duration::days(10)),
            points: Some(120),
            modified_by: instructor_id,
        })
        .await
        .expect("update assignment");
    assert_eq!(updated.title, "Project 1 Revised");
    assert_eq!(updated.points, Some(120));

    let deleted = service.delete(created.id).await.expect("delete assignment");
    assert_eq!(deleted, "Assignment deleted");

    let err = service
        .find_by_id(created.id)
        .await
        .expect_err("deleted assignment should not exist");
    assert!(matches!(err, AppError::NotFound(_)));
}

#[tokio::test]
async fn db_list_by_class_with_attachment_count_filters_and_counts() {
    let Some(pool) = maybe_setup_db().await else {
        return;
    };
    let service = create_assignment_service(pool.clone());
    let instructor_id =
        insert_user(&pool, &unique_username("counts_instructor"), "instructor").await;
    let class_id = insert_class(&pool, instructor_id, "Counts Class").await;
    let other_class_id = insert_class(&pool, instructor_id, "Other Class").await;

    let target_assignment = service
        .create(CreateAssignmentDto {
            class_id,
            title: "Counted Assignment".to_string(),
            description: Some("with attachments".to_string()),
            due_at: None,
            points: Some(50),
            modified_by: instructor_id,
        })
        .await
        .expect("create target assignment");

    let _other_assignment = service
        .create(CreateAssignmentDto {
            class_id: other_class_id,
            title: "Other Assignment".to_string(),
            description: Some("different class".to_string()),
            due_at: None,
            points: Some(10),
            modified_by: instructor_id,
        })
        .await
        .expect("create other assignment");

    let file_1 = insert_uploaded_file(&pool, instructor_id, "a1.pdf").await;
    let file_2 = insert_uploaded_file(&pool, instructor_id, "a2.pdf").await;

    service
        .attach_file(target_assignment.id, file_1, instructor_id)
        .await
        .expect("attach first file");
    service
        .attach_file(target_assignment.id, file_2, instructor_id)
        .await
        .expect("attach second file");

    let rows = service
        .list_by_class_with_attachment_count(class_id)
        .await
        .expect("list with counts");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].id, target_assignment.id);
    assert_eq!(rows[0].attachment_count, 2);

    let other_rows = service
        .list_by_class_with_attachment_count(other_class_id)
        .await
        .expect("list other class");
    assert_eq!(other_rows.len(), 1);
    assert_eq!(other_rows[0].attachment_count, 0);
}

#[tokio::test]
async fn db_attach_list_and_remove_file_happy_and_edge_paths() {
    let Some(pool) = maybe_setup_db().await else {
        return;
    };
    let service = create_assignment_service(pool.clone());
    let instructor_id =
        insert_user(&pool, &unique_username("files_instructor"), "instructor").await;
    let class_id = insert_class(&pool, instructor_id, "Files Class").await;
    let assignment = service
        .create(CreateAssignmentDto {
            class_id,
            title: "Upload Work".to_string(),
            description: Some("submit assets".to_string()),
            due_at: None,
            points: Some(25),
            modified_by: instructor_id,
        })
        .await
        .expect("create assignment");
    let file_id = insert_uploaded_file(&pool, instructor_id, "work.pdf").await;

    service
        .attach_file(assignment.id, file_id, instructor_id)
        .await
        .expect("attach file");
    service
        .attach_file(assignment.id, file_id, instructor_id)
        .await
        .expect("reattach same file should be noop");

    let attachments = service
        .list_attachments(assignment.id)
        .await
        .expect("list attachments");
    assert_eq!(attachments.len(), 1);
    assert_eq!(attachments[0].file_id, file_id);

    let removed = service
        .remove_file(assignment.id, file_id)
        .await
        .expect("remove existing file");
    assert!(removed);

    let removed_again = service
        .remove_file(assignment.id, file_id)
        .await
        .expect("remove missing file");
    assert!(!removed_again);
}

#[tokio::test]
async fn db_update_and_delete_missing_assignment_returns_not_found() {
    let Some(pool) = maybe_setup_db().await else {
        return;
    };
    let service = create_assignment_service(pool.clone());
    let instructor_id =
        insert_user(&pool, &unique_username("missing_instructor"), "instructor").await;
    let class_id = insert_class(&pool, instructor_id, "Missing Paths Class").await;

    let update_err = service
        .update(UpdateAssignmentDto {
            id: Uuid::new_v4(),
            class_id,
            title: "missing".to_string(),
            description: Some("missing".to_string()),
            due_at: None,
            points: Some(5),
            modified_by: instructor_id,
        })
        .await
        .expect_err("missing update should error");
    assert!(matches!(update_err, AppError::NotFound(_)));

    let delete_err = service
        .delete(Uuid::new_v4())
        .await
        .expect_err("missing delete should error");
    assert!(matches!(delete_err, AppError::NotFound(_)));
}

#[tokio::test]
async fn list_returns_database_error_when_repo_fails() {
    let service = build_service_with_repo(FakeAssignmentRepository {
        fail_find_all: true,
        ..Default::default()
    });
    let err = service.list().await.expect_err("list should fail");
    assert!(matches!(err, AppError::DatabaseError(_)));
}

#[tokio::test]
async fn list_by_class_returns_rows() {
    let id = Uuid::new_v4();
    let class_id = Uuid::new_v4();
    let mut map = HashMap::new();
    let mut seeded = seed_assignment(id);
    seeded.class_id = class_id;
    map.insert(id, seeded);

    let service = build_service_with_repo(FakeAssignmentRepository {
        store: Mutex::new(map),
        ..Default::default()
    });
    let rows = service
        .list_by_class(class_id)
        .await
        .expect("list by class");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].id, id);
}

#[tokio::test]
async fn list_by_class_returns_database_error_when_repo_fails() {
    let service = build_service_with_repo(FakeAssignmentRepository {
        fail_find_by_class_id: true,
        ..Default::default()
    });
    let err = service
        .list_by_class(Uuid::new_v4())
        .await
        .expect_err("should fail");
    assert!(matches!(err, AppError::DatabaseError(_)));
}

#[tokio::test]
async fn list_by_class_with_attachment_count_returns_database_error_when_repo_fails() {
    let service = build_service_with_repo(FakeAssignmentRepository {
        fail_find_by_class_id_with_count: true,
        ..Default::default()
    });
    let err = service
        .list_by_class_with_attachment_count(Uuid::new_v4())
        .await
        .expect_err("should fail");
    assert!(matches!(err, AppError::DatabaseError(_)));
}

#[tokio::test]
async fn list_attachments_attach_and_remove_cover_success_and_error_paths() {
    let assignment_id = Uuid::new_v4();
    let file_id = Uuid::new_v4();
    let attachments = vec![AssignmentAttachment {
        assignment_id,
        file_id,
        file_name: "a.pdf".to_string(),
        origin_file_name: "a.pdf".to_string(),
        file_url: "/f/a.pdf".to_string(),
        content_type: "application/pdf".to_string(),
        file_size: 1024,
        created_by: Some(Uuid::new_v4()),
        created_at: Utc::now(),
    }];

    let service = build_service_with_repo(FakeAssignmentRepository {
        list_attachments_result: attachments.clone(),
        remove_attachment_result: true,
        ..Default::default()
    });
    let listed = service
        .list_attachments(assignment_id)
        .await
        .expect("list attachments");
    assert_eq!(listed.len(), 1);
    service
        .attach_file(assignment_id, file_id, Uuid::new_v4())
        .await
        .expect("attach file");
    let removed = service
        .remove_file(assignment_id, file_id)
        .await
        .expect("remove file");
    assert!(removed);

    let failing = build_service_with_repo(FakeAssignmentRepository {
        fail_list_attachments: true,
        fail_add_attachment: true,
        fail_remove_attachment: true,
        ..Default::default()
    });
    assert!(matches!(
        failing
            .list_attachments(assignment_id)
            .await
            .expect_err("list should fail"),
        AppError::DatabaseError(_)
    ));
    assert!(matches!(
        failing
            .attach_file(assignment_id, file_id, Uuid::new_v4())
            .await
            .expect_err("attach should fail"),
        AppError::DatabaseError(_)
    ));
    assert!(matches!(
        failing
            .remove_file(assignment_id, file_id)
            .await
            .expect_err("remove should fail"),
        AppError::DatabaseError(_)
    ));
}

#[tokio::test]
async fn create_returns_database_error_or_not_found_for_missing_row() {
    let create_error_service = build_service_with_repo(FakeAssignmentRepository {
        fail_create: true,
        ..Default::default()
    });
    let err = create_error_service
        .create(CreateAssignmentDto {
            class_id: Uuid::new_v4(),
            title: "new".to_string(),
            description: None,
            due_at: None,
            points: Some(10),
            modified_by: Uuid::new_v4(),
        })
        .await
        .expect_err("create should fail");
    assert!(matches!(err, AppError::DatabaseError(_)));

    let missing_row_service = build_service_with_repo(FakeAssignmentRepository {
        create_without_store: true,
        ..Default::default()
    });
    let err = missing_row_service
        .create(CreateAssignmentDto {
            class_id: Uuid::new_v4(),
            title: "new".to_string(),
            description: None,
            due_at: None,
            points: Some(10),
            modified_by: Uuid::new_v4(),
        })
        .await
        .expect_err("missing row should fail");
    assert!(matches!(err, AppError::NotFound(_)));
}

#[tokio::test]
async fn find_update_and_delete_return_database_error_or_not_found_paths() {
    let service = build_service_with_repo(FakeAssignmentRepository {
        fail_find_by_id_error: true,
        fail_update: true,
        fail_delete: true,
        ..Default::default()
    });
    assert!(matches!(
        service
            .find_by_id(Uuid::new_v4())
            .await
            .expect_err("find should fail"),
        AppError::DatabaseError(_)
    ));
    assert!(matches!(
        service
            .update(UpdateAssignmentDto {
                id: Uuid::new_v4(),
                class_id: Uuid::new_v4(),
                title: "t".to_string(),
                description: None,
                due_at: None,
                points: Some(1),
                modified_by: Uuid::new_v4(),
            })
            .await
            .expect_err("update should fail"),
        AppError::DatabaseError(_)
    ));
    assert!(matches!(
        service
            .delete(Uuid::new_v4())
            .await
            .expect_err("delete should fail"),
        AppError::DatabaseError(_)
    ));

    let not_found_delete = build_service_with_repo(FakeAssignmentRepository {
        force_delete_false: true,
        ..Default::default()
    });
    assert!(matches!(
        not_found_delete
            .delete(Uuid::new_v4())
            .await
            .expect_err("delete should be not found"),
        AppError::NotFound(_)
    ));
}
