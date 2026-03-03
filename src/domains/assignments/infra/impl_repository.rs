use async_trait::async_trait;

use crate::domains::assignments::domain::{
    model::Assignment, repository::AssignmentRepositoryTrait,
};

use crate::domains::assignments::dto::assignment_dto::{CreateAssignmentDto, UpdateAssignmentDto};
use sqlx::{Error, PgPool};
use uuid::Uuid;

pub struct AssignmentRepository {
    pool: PgPool,
}

const FIND_ALL_ASSIGNMENTS_QUERY: &str = r#"
    SELECT
        a.id,
        a.class_id,
        a.title,
        a.description,
        a.due_at,
        a.created_by,
        a.created_at,
        a.modified_by,
        a.modified_at
    FROM assignments a
    WHERE 1=1
    "#;

const FIND_ASSIGNMENT_BY_ID_QUERY: &str = r#"
SELECT
    a.id,
    a.class_id,
    a.title,
    a.description,
    a.due_at,
    a.created_by,
    a.created_at,
    a.modified_by,
    a.modified_at
    FROM assignments a
    WHERE a.id = $1"#;

const FIND_ASSIGNMENTS_BY_CLASS_ID_QUERY: &str = r#"
SELECT
    a.id,
    a.class_id,
    a.title,
    a.description,
    a.due_at,
    a.created_by,
    a.created_at,
    a.modified_by,
    a.modified_at
    FROM assignments a
    WHERE a.class_id = $1
    ORDER BY a.due_at NULLS LAST, a.created_at DESC
"#;

#[async_trait]
impl AssignmentRepositoryTrait for AssignmentRepository {
    fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    async fn find_all(&self) -> Result<Vec<Assignment>, Error> {
        let assignments = sqlx::query_as::<_, Assignment>(FIND_ALL_ASSIGNMENTS_QUERY)
            .fetch_all(&self.pool)
            .await?;
        Ok(assignments)
    }

    async fn find_by_class_id(&self, class_id: Uuid) -> Result<Vec<Assignment>, Error> {
        let assignments = sqlx::query_as::<_, Assignment>(FIND_ASSIGNMENTS_BY_CLASS_ID_QUERY)
            .bind(class_id)
            .fetch_all(&self.pool)
            .await?;
        Ok(assignments)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Assignment>, Error> {
        let assignment = sqlx::query_as::<_, Assignment>(FIND_ASSIGNMENT_BY_ID_QUERY)
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(assignment)
    }

    async fn create(&self, assignment: CreateAssignmentDto) -> Result<Uuid, Error> {
        let mut tx = self.pool.begin().await?;
        let id = Uuid::new_v4();

        sqlx::query(
            r#"
                INSERT INTO assignments (id, class_id, title, description, due_at, created_by, modified_by)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                "#,
        )
            .bind(id)
            .bind(assignment.class_id)
            .bind(assignment.title.clone())
            .bind(assignment.description.clone())
            .bind(assignment.due_at)
            .bind(assignment.modified_by)
            .bind(assignment.modified_by)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;
        Ok(id)
    }

    async fn update(
        &self,
        id: Uuid,
        assignment: UpdateAssignmentDto,
    ) -> Result<Option<Assignment>, Error> {
        let mut tx = self.pool.begin().await?;
        let existing = sqlx::query_as::<_, Assignment>(FIND_ASSIGNMENT_BY_ID_QUERY)
            .bind(id)
            .fetch_optional(&mut *tx)
            .await?;

        if existing.is_some() {
            sqlx::query(
                r#"
                UPDATE assignments
                SET class_id = $1,
                    title = $2,
                    description = $3,
                    due_at = $4,
                    modified_by = $5,
                    modified_at = NOW()
                WHERE id = $6
                "#,
            )
            .bind(assignment.class_id)
            .bind(assignment.title.clone())
            .bind(assignment.description.clone())
            .bind(assignment.due_at)
            .bind(assignment.modified_by)
            .bind(id)
            .execute(&mut *tx)
            .await?;

            let updated_assignment = sqlx::query_as::<_, Assignment>(FIND_ASSIGNMENT_BY_ID_QUERY)
                .bind(id)
                .fetch_one(&mut *tx)
                .await?;

            tx.commit().await?;
            return Ok(Some(updated_assignment));
        }
        tx.rollback().await?;
        Ok(None)
    }

    async fn delete(&self, id: Uuid) -> Result<bool, Error> {
        let mut tx = self.pool.begin().await?;
        let res = sqlx::query(r#"DELETE FROM assignments WHERE id = $1"#)
            .bind(id)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(res.rows_affected() > 0)
    }
}
