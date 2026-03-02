//! This module defines the `UserAuth` model used for representing
//! authentication data tied to a user.

use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

use crate::domains::user::UserRole;

/// Represents a user's authentication information, including hashed password.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserAuth {
    pub user_id: uuid::Uuid,
    pub password_hash: String,
}

/// Projection used during login to include role information.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AuthIdentity {
    pub user_id: uuid::Uuid,
    pub password_hash: String,
    pub user_role: UserRole,
}
