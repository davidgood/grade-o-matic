use async_trait::async_trait;

use crate::domains::assignments::domain::{
    model::{
        Assignment, AssignmentAttachment, AssignmentWithAttachmentCount,
        StudentAssignmentExtension, StudentAssignmentSubmission,
    },
    repository::AssignmentRepositoryTrait,
};
use crate::domains::assignments::dto::assignment_dto::{
    CreateAssignmentDto, UpdateAssignmentDto, UpsertStudentAssignmentExtensionDto,
};
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
        NULL::timestamptz AS extension_due_at,
        a.due_at AS effective_due_at,
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
    NULL::timestamptz AS extension_due_at,
    a.due_at AS effective_due_at,
    a.deadline_type,
    a.points,
    a.created_by,
    a.created_at,
    a.modified_by,
    a.modified_at
FROM assignments a
WHERE a.id = $1
"#;

const FIND_ASSIGNMENT_BY_ID_FOR_STUDENT_QUERY: &str = r#"
SELECT
    a.id,
    a.class_id,
    a.title,
    a.description,
    a.due_at,
    ase.due_at AS extension_due_at,
    COALESCE(ase.due_at, a.due_at) AS effective_due_at,
    a.deadline_type,
    a.points,
    a.created_by,
    a.created_at,
    a.modified_by,
    a.modified_at
FROM assignments a
LEFT JOIN assignment_student_extensions ase
  ON ase.assignment_id = a.id
 AND ase.student_id = $2
WHERE a.id = $1
"#;

const FIND_ASSIGNMENTS_BY_CLASS_ID_QUERY: &str = r#"
SELECT
    a.id,
    a.class_id,
    a.title,
    a.description,
    a.due_at,
    NULL::timestamptz AS extension_due_at,
    a.due_at AS effective_due_at,
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

const FIND_ASSIGNMENTS_BY_CLASS_ID_FOR_STUDENT_QUERY: &str = r#"
SELECT
    a.id,
    a.class_id,
    a.title,
    a.description,
    a.due_at,
    ase.due_at AS extension_due_at,
    COALESCE(ase.due_at, a.due_at) AS effective_due_at,
    a.deadline_type,
    a.points,
    a.created_by,
    a.created_at,
    a.modified_by,
    a.modified_at
FROM assignments a
LEFT JOIN assignment_student_extensions ase
  ON ase.assignment_id = a.id
 AND ase.student_id = $2
WHERE a.class_id = $1
ORDER BY COALESCE(ase.due_at, a.due_at) NULLS LAST, a.created_at DESC
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
                AND COALESCE(ase.due_at, a.due_at) IS NOT NULL
                AND aa.created_at > COALESCE(ase.due_at, a.due_at)
            THEN TRUE
            ELSE FALSE
        END AS is_late,
        gj.status AS grading_status,
        gj.completed_at AS grading_completed_at
    FROM assignment_attachments aa
    INNER JOIN assignments a ON a.id = aa.assignment_id
    LEFT JOIN assignment_student_extensions ase
      ON ase.assignment_id = aa.assignment_id
     AND ase.student_id = $2
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
      NULL::timestamptz AS extension_due_at,
      a.due_at AS effective_due_at,
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
    ORDER BY a.due_at NULLS LAST, a.created_at DESC
"#;

const LIST_STUDENT_EXTENSIONS_QUERY: &str = r#"
    SELECT
        assignment_id,
        student_id,
        due_at,
        created_by,
        created_at,
        modified_by,
        modified_at
    FROM assignment_student_extensions
    WHERE assignment_id = $1
    ORDER BY due_at DESC, student_id ASC
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

    async fn find_by_class_id_for_student(
        &self,
        class_id: Uuid,
        student_id: Uuid,
    ) -> Result<Vec<Assignment>, Error> {
        let assignments =
            sqlx::query_as::<_, Assignment>(FIND_ASSIGNMENTS_BY_CLASS_ID_FOR_STUDENT_QUERY)
                .bind(class_id)
                .bind(student_id)
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

    async fn find_by_id_for_student(
        &self,
        id: Uuid,
        student_id: Uuid,
    ) -> Result<Option<Assignment>, Error> {
        let assignment = sqlx::query_as::<_, Assignment>(FIND_ASSIGNMENT_BY_ID_FOR_STUDENT_QUERY)
            .bind(id)
            .bind(student_id)
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

    async fn list_student_extensions(
        &self,
        assignment_id: Uuid,
    ) -> Result<Vec<StudentAssignmentExtension>, Error> {
        let rows = sqlx::query_as::<_, StudentAssignmentExtension>(LIST_STUDENT_EXTENSIONS_QUERY)
            .bind(assignment_id)
            .fetch_all(&self.pool)
            .await?;
        Ok(rows)
    }

    async fn upsert_student_extension(
        &self,
        extension: UpsertStudentAssignmentExtensionDto,
    ) -> Result<StudentAssignmentExtension, Error> {
        let row = sqlx::query_as::<_, StudentAssignmentExtension>(
            r#"
                INSERT INTO assignment_student_extensions (
                    assignment_id, student_id, due_at, created_by, modified_by
                )
                VALUES ($1, $2, $3, $4, $4)
                ON CONFLICT (assignment_id, student_id)
                DO UPDATE SET
                    due_at = EXCLUDED.due_at,
                    modified_by = EXCLUDED.modified_by,
                    modified_at = NOW()
                RETURNING
                    assignment_id,
                    student_id,
                    due_at,
                    created_by,
                    created_at,
                    modified_by,
                    modified_at
            "#,
        )
        .bind(extension.assignment_id)
        .bind(extension.student_id)
        .bind(extension.due_at)
        .bind(extension.modified_by)
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    async fn delete_student_extension(
        &self,
        assignment_id: Uuid,
        student_id: Uuid,
    ) -> Result<bool, Error> {
        let result = sqlx::query(
            r#"
                DELETE FROM assignment_student_extensions
                WHERE assignment_id = $1 AND student_id = $2
            "#,
        )
        .bind(assignment_id)
        .bind(student_id)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
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
