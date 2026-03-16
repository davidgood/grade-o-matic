use async_trait::async_trait;

use crate::domains::assignments::domain::{
    model::{Assignment, AssignmentAttachment, StudentAssignmentSubmission},
    repository::AssignmentRepositoryTrait,
};

use crate::domains::assignments::domain::model::AssignmentWithAttachmentCount;
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
        a.deadline_type,
        a.points,
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
    a.deadline_type,
    a.points,
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
    a.deadline_type,
    a.points,
    a.created_by,
    a.created_at,
    a.modified_by,
    a.modified_at
    FROM assignments a
    WHERE a.class_id = $1
    ORDER BY a.due_at NULLS LAST, a.created_at DESC
"#;

const LIST_ASSIGNMENT_ATTACHMENTS_QUERY: &str = r#"
    SELECT
        aa.assignment_id,
        aa.file_id,
        uf.file_name,
        uf.origin_file_name,
        uf.file_url,
        uf.content_type,
        uf.file_size,
        aa.created_by,
        aa.created_at
    FROM assignment_attachments aa
    INNER JOIN uploaded_files uf ON uf.id = aa.file_id
    WHERE aa.assignment_id = $1
    ORDER BY aa.created_at DESC
"#;

const LIST_STUDENT_SUBMISSION_HISTORY_QUERY: &str = r#"
    SELECT
        aa.assignment_id,
        aa.file_id,
        uf.file_name,
        uf.origin_file_name,
        uf.file_url,
        uf.content_type,
        uf.file_size,
        aa.created_by AS submitted_by,
        aa.created_at AS submitted_at,
        a.deadline_type,
        CASE
            WHEN a.deadline_type = 'soft_deadline'
                AND a.due_at IS NOT NULL
                AND aa.created_at > a.due_at
            THEN TRUE
            ELSE FALSE
        END AS is_late,
        gj.status AS grading_status,
        gj.completed_at AS grading_completed_at
    FROM assignment_attachments aa
    INNER JOIN assignments a ON a.id = aa.assignment_id
    INNER JOIN uploaded_files uf ON uf.id = aa.file_id
    LEFT JOIN grading_jobs gj
        ON gj.assignment_id = aa.assignment_id
       AND gj.file_id = aa.file_id
    WHERE aa.assignment_id = $1
      AND aa.created_by = $2
    ORDER BY aa.created_at DESC, uf.origin_file_name ASC
"#;

const LIST_ASSIGNMENTS_WITH_ATTACHMENT_COUNT_QUERY: &str = r#"
    SELECT
      a.id,
      a.class_id,
      a.title,
      a.description,
      a.due_at,
      a.deadline_type,
      a.points,
      a.created_by,
      a.created_at,
      a.modified_by,
      a.modified_at,
      COUNT(aa.file_id)::integer AS attachment_count
    FROM assignments a
    LEFT JOIN assignment_attachments aa ON aa.assignment_id = a.id
    WHERE a.class_id = $1
    GROUP BY
      a.id, a.class_id, a.title, a.description, a.due_at, a.deadline_type,
      a.created_by, a.created_at, a.modified_by, a.modified_at
    ORDER BY a.due_at NULLS LAST, a.created_at DESC;
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

    async fn find_by_class_id_with_attachment_count(
        &self,
        id: Uuid,
    ) -> Result<Vec<AssignmentWithAttachmentCount>, Error> {
        let assignments = sqlx::query_as::<_, AssignmentWithAttachmentCount>(
            LIST_ASSIGNMENTS_WITH_ATTACHMENT_COUNT_QUERY,
        )
        .bind(id)
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

    async fn list_attachments(
        &self,
        assignment_id: Uuid,
    ) -> Result<Vec<AssignmentAttachment>, Error> {
        let attachments =
            sqlx::query_as::<_, AssignmentAttachment>(LIST_ASSIGNMENT_ATTACHMENTS_QUERY)
                .bind(assignment_id)
                .fetch_all(&self.pool)
                .await?;
        Ok(attachments)
    }

    async fn list_student_submission_history(
        &self,
        assignment_id: Uuid,
        student_id: Uuid,
    ) -> Result<Vec<StudentAssignmentSubmission>, Error> {
        let rows =
            sqlx::query_as::<_, StudentAssignmentSubmission>(LIST_STUDENT_SUBMISSION_HISTORY_QUERY)
                .bind(assignment_id)
                .bind(student_id)
                .fetch_all(&self.pool)
                .await?;
        Ok(rows)
    }

    async fn add_attachment(
        &self,
        assignment_id: Uuid,
        file_id: Uuid,
        created_by: Uuid,
    ) -> Result<(), Error> {
        sqlx::query(
            r#"
                INSERT INTO assignment_attachments (assignment_id, file_id, created_by)
                VALUES ($1, $2, $3)
                ON CONFLICT (assignment_id, file_id) DO NOTHING
                "#,
        )
        .bind(assignment_id)
        .bind(file_id)
        .bind(created_by)
        .execute(&self.pool)
        .await?;

        // Best-effort enqueue for asynchronous grading workers.
        // This allows older environments (without the new migration) to continue working.
        let enqueue_result = sqlx::query(
            r#"
                INSERT INTO grading_jobs (assignment_id, file_id, submitted_by, status)
                VALUES ($1, $2, $3, 'queued')
                ON CONFLICT (assignment_id, file_id) DO NOTHING
                "#,
        )
        .bind(assignment_id)
        .bind(file_id)
        .bind(created_by)
        .execute(&self.pool)
        .await;

        if let Err(err) = enqueue_result {
            if let sqlx::Error::Database(db_err) = &err
                && db_err.code().as_deref() == Some("42P01")
            {
                tracing::warn!("grading_jobs table missing; skipping enqueue");
            } else {
                return Err(err);
            }
        }
        Ok(())
    }

    async fn remove_attachment(&self, assignment_id: Uuid, file_id: Uuid) -> Result<bool, Error> {
        let result = sqlx::query(
            r#"
                DELETE FROM assignment_attachments
                WHERE assignment_id = $1 AND file_id = $2
                "#,
        )
        .bind(assignment_id)
        .bind(file_id)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    async fn create(&self, assignment: CreateAssignmentDto) -> Result<Uuid, Error> {
        let mut tx = self.pool.begin().await?;
        let id = Uuid::new_v4();

        sqlx::query(
            r#"
                INSERT INTO assignments (
                    id, class_id, title, description, due_at, deadline_type, points, created_by, modified_by
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                "#,
        )
            .bind(id)
            .bind(assignment.class_id)
            .bind(assignment.title.clone())
            .bind(assignment.description.clone())
            .bind(assignment.due_at)
            .bind(assignment.deadline_type)
            .bind(assignment.points)
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
                    deadline_type = $5,
                    points = $6,
                    modified_by = $7,
                    modified_at = NOW()
                WHERE id = $8
                "#,
            )
            .bind(assignment.class_id)
            .bind(assignment.title.clone())
            .bind(assignment.description.clone())
            .bind(assignment.due_at)
            .bind(assignment.deadline_type)
            .bind(assignment.points)
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
