use crate::domains::classes::domain::model::Class;
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
