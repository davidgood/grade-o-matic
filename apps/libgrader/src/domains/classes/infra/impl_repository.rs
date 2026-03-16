use crate::domains::classes::domain::model::{Class, ClassesWithAssignments};
use crate::domains::classes::domain::repository::ClassRepositoryTrait;
use crate::domains::classes::dto::class_dto::{CreateClassDto, UpdateClassDto};
use async_trait::async_trait;
use chrono::Utc;
use sqlx::{Error, PgPool};
use uuid::Uuid;

pub struct ClassRepository {
    pool: PgPool,
}

const FIND_CLASS_BY_ID_QUERY: &str = r#"
SELECT
    c.id,
    c.title,
    c.description,
    c.term,
    c.owner_id,
    c.created_by,
    c.created_at,
    c.modified_by,
    c.modified_at
    FROM classes c
    WHERE c.id = $1"#;

pub const LIST_CLASSES_WITH_ASSIGNMENTS: &str = r#"
    SELECT c.id as class_id,
           c.title as class_title,
           c.term as class_term,
           a.id as assignment_id,
           a.title as assignment_title,
           a.description as assignment_description,
           a.due_at,
           a.deadline_type,
           a.points
    FROM classes c
    LEFT JOIN assignments a on c.id = a.class_id
    WHERE c.owner_id = $1
    ORDER BY c.title ASC, a.due_at DESC NULLS LAST, a.title ASC
    LIMIT 1000;
"#;

#[async_trait]
impl ClassRepositoryTrait for ClassRepository {
    fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    async fn list(&self) -> Result<Vec<Class>, Error> {
        let classes = sqlx::query_as::<_, Class>("SELECT * FROM classes")
            .fetch_all(&self.pool)
            .await?;
        Ok(classes)
    }

    async fn list_classes_with_assignments(
        &self,
        owner_id: Uuid,
    ) -> Result<Vec<ClassesWithAssignments>, Error> {
        let classes = sqlx::query_as::<_, ClassesWithAssignments>(LIST_CLASSES_WITH_ASSIGNMENTS)
            .bind(owner_id)
            .fetch_all(&self.pool)
            .await?;
        Ok(classes)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Class>, Error> {
        let class = sqlx::query_as::<_, Class>(FIND_CLASS_BY_ID_QUERY)
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(class)
    }

    async fn create(&self, class: CreateClassDto) -> Result<Uuid, Error> {
        let mut tx = self.pool.begin().await?;
        let id = Uuid::new_v4();
        sqlx::query(
            r#"
                INSERT INTO classes (
                    id,
                    title,
                    description,
                    term,
                    owner_id,
                    created_at,
                    created_by,
                    modified_by
                )
                VALUES ($1, $2, $3, $4, COALESCE($5, $6), $7, $6, $6)
                "#,
        )
        .bind(id)
        .bind(class.title)
        .bind(class.description)
        .bind(class.term)
        .bind(class.owner_id)
        .bind(class.modified_by)
        .bind(Utc::now())
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(id)
    }

    async fn update(&self, id: Uuid, class: UpdateClassDto) -> Result<Option<Class>, Error> {
        let mut tx = self.pool.begin().await?;
        let existing = sqlx::query_as::<_, Class>(FIND_CLASS_BY_ID_QUERY)
            .bind(id)
            .fetch_optional(&mut *tx)
            .await?;

        if existing.is_some() {
            sqlx::query(
                r#"
                UPDATE classes
                SET title = $1,
                    description = $2,
                    term = $3,
                    owner_id = COALESCE($4, owner_id),
                    modified_by = $5,
                    modified_at = NOW()
                WHERE id = $6
                "#,
            )
            .bind(class.title)
            .bind(class.description)
            .bind(class.term)
            .bind(class.owner_id)
            .bind(class.modified_by)
            .bind(id)
            .execute(&mut *tx)
            .await?;

            let updated_class = sqlx::query_as::<_, Class>(FIND_CLASS_BY_ID_QUERY)
                .bind(id)
                .fetch_one(&mut *tx)
                .await?;

            tx.commit().await?;
            Ok(Some(updated_class))
        } else {
            tx.rollback().await?;
            Ok(None)
        }
    }

    async fn delete(&self, id: Uuid) -> Result<bool, Error> {
        let mut tx = self.pool.begin().await?;

        let res = sqlx::query(r#"DELETE FROM classes WHERE id = $1"#)
            .bind(id)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;
        Ok(res.rows_affected() > 0)
    }
}
