use crate::domains::class_memberships::domain::model::ClassMembership;
use crate::domains::class_memberships::domain::repository::ClassMembershipRepositoryTrait;
use crate::domains::class_memberships::dto::class_membership_dto::{
    CreateClassMembershipDto, UpdateClassMembershipDto,
};
use async_trait::async_trait;
use chrono::Utc;
use sqlx::{Error, PgPool};
use uuid::Uuid;

pub struct ClassMembershipRepository {
    pool: PgPool,
}

const FIND_BY_ID_QUERY: &str = r#"
SELECT
    cm.id,
    cm.class_id,
    cm.user_id,
    cm.role,
    cm.created_at,
    cm.modified_at
FROM class_memberships cm
WHERE cm.id = $1
"#;

#[async_trait]
impl ClassMembershipRepositoryTrait for ClassMembershipRepository {
    fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    async fn list_by_class_id(&self, class_id: Uuid) -> Result<Vec<ClassMembership>, Error> {
        let memberships = sqlx::query_as::<_, ClassMembership>(
            r#"
            SELECT
                cm.id,
                cm.class_id,
                cm.user_id,
                cm.role,
                cm.created_at,
                cm.modified_at
            FROM class_memberships cm
            WHERE cm.class_id = $1
            ORDER BY cm.created_at DESC
            "#,
        )
        .bind(class_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(memberships)
    }

    async fn list_by_user_id(&self, user_id: Uuid) -> Result<Vec<ClassMembership>, Error> {
        let memberships = sqlx::query_as::<_, ClassMembership>(
            r#"
            SELECT
                cm.id,
                cm.class_id,
                cm.user_id,
                cm.role,
                cm.created_at,
                cm.modified_at
            FROM class_memberships cm
            WHERE cm.user_id = $1
            ORDER BY cm.created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(memberships)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<ClassMembership>, Error> {
        let membership = sqlx::query_as::<_, ClassMembership>(FIND_BY_ID_QUERY)
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(membership)
    }

    async fn create(&self, membership: CreateClassMembershipDto) -> Result<Uuid, Error> {
        let mut tx = self.pool.begin().await?;
        let id = Uuid::new_v4();

        sqlx::query(
            r#"
            INSERT INTO class_memberships (id, class_id, user_id, role, created_at, modified_at)
            VALUES ($1, $2, $3, $4, $5, $5)
            "#,
        )
        .bind(id)
        .bind(membership.class_id)
        .bind(membership.user_id)
        .bind(membership.role)
        .bind(Utc::now())
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(id)
    }

    async fn update(
        &self,
        id: Uuid,
        membership: UpdateClassMembershipDto,
    ) -> Result<Option<ClassMembership>, Error> {
        let mut tx = self.pool.begin().await?;
        let existing = sqlx::query_as::<_, ClassMembership>(FIND_BY_ID_QUERY)
            .bind(id)
            .fetch_optional(&mut *tx)
            .await?;

        if existing.is_some() {
            sqlx::query(
                r#"
                UPDATE class_memberships
                SET role = $1, modified_at = NOW()
                WHERE id = $2
                "#,
            )
            .bind(membership.role)
            .bind(id)
            .execute(&mut *tx)
            .await?;

            let updated = sqlx::query_as::<_, ClassMembership>(FIND_BY_ID_QUERY)
                .bind(id)
                .fetch_one(&mut *tx)
                .await?;
            tx.commit().await?;
            Ok(Some(updated))
        } else {
            tx.rollback().await?;
            Ok(None)
        }
    }

    async fn delete(&self, id: Uuid) -> Result<bool, Error> {
        let mut tx = self.pool.begin().await?;
        let res = sqlx::query(r#"DELETE FROM class_memberships WHERE id = $1"#)
            .bind(id)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(res.rows_affected() > 0)
    }
}
