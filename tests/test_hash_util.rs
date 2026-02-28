use grade_o_matic::common::hash_util::{hash_password, verify_password};

#[test]
fn test_password_hash_and_verify() {
    let password = "super_secret_password";
    let hash = hash_password(password).expect("Failed to hash password");

    assert!(verify_password(&hash, password));
    assert!(!verify_password(&hash, "wrong_password"));
}

#[test]
fn test_argon2_jvm_verify() {
    let password = "mySecretPassword";
    let hash = "$argon2i$v=19$m=65536,t=2,p=1$vNVL5PZ1hRwgLUlGmCQVTA$fg1d0/f8pdtMnzQTeh2YE6R0E8vfqMOQOs5k6Y22Qi0";

    assert!(verify_password(hash, password));
}
