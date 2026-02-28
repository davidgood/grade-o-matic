use async_trait::async_trait;

use crate::domains::assignments::domain::{
    model::Assignment, repository::AssignmentRepositoryTrait,
};

use sqlx::PgPool;

pub struct AssignmentRepository;

const FIND_ASSIGNMENT_QUERY: &str = r#"
    SELECT
        a.id,
        a.title,
        a.description,
        a.due_at
    FROM assignments a
    WHERE 1=1
    "#;

#[async_trait]
impl AssignmentRepositoryTrait for AssignmentRepository {
    async fn find_all(&self, pool: PgPool) -> Result<Vec<Assignment>, sqlx::Error> {
        let assignments = sqlx::query_as::<_, Assignment>(FIND_ASSIGNMENT_QUERY)
            .fetch_all(&pool)
            .await?;
        Ok(assignments)
    }
}
