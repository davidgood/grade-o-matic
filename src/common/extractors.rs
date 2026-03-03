use crate::common::error::AppError;
use crate::common::jwt::Claims;
use crate::domains::user::UserRole;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;

pub struct AdminUser(pub Claims);
impl<S> FromRequestParts<S> for AdminUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // jwt_auth_web middleware should have inserted Claims into request extensions.
        let claims = parts
            .extensions
            .get::<Claims>()
            .cloned()
            .ok_or(AppError::Unauthorized)?;

        if matches!(claims.user_role, UserRole::Admin) {
            Ok(AdminUser(claims))
        } else {
            Err(AppError::Forbidden)
        }
    }
}
