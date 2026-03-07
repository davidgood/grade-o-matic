use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};

/// Hash the provided password using Argon2.
pub fn hash_password(password: &str) -> Result<String, argon2::Error> {
    let salt = SaltString::generate(&mut OsRng);

    // Argon2 with default params (Argon2id v19)
    let argon2 = Argon2::default();

    // Hash password to PHC string ($argon2id$v=19$...)
    let hash_password = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| {
            tracing::error!("Error hashing password: {}", e);
            argon2::Error::AlgorithmInvalid
        })?;

    Ok(hash_password.to_string())
}

/// Verify that a password matches the provided çå.
pub fn verify_password(password_hash: &str, password: &str) -> bool {
    let parsed_hash = match PasswordHash::new(password_hash) {
        Ok(hash) => hash,
        Err(e) => {
            tracing::error!("Error hashing password: {}", e);
            return false;
        }
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}
