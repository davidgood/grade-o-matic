use grade_o_matic_web::common::config::Config;
use std::{
    env, fs,
    path::PathBuf,
    sync::{LazyLock, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

static GLOBAL_ENV_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

const TRACKED_VARS: &[&str] = &[
    "RUST_LOG",
    "DEBUG",
    "JWT_SECRET",
    "JWT_SECRET_KEY",
    "DATABASE_NAME",
    "DATABASE_USER",
    "DATABASE_PASSWORD",
    "DATABASE_HOST",
    "DATABASE_PORT",
    "DATABASE_URL",
    "DB_MAX_OPEN_CONNS",
    "DB_MAX_IDLE_CONNS",
    "DB_CONN_MAX_LIFETIME",
    "OIDC_ENABLED",
    "OIDC_ISSUER_URL",
    "OIDC_CLIENT_ID",
    "OIDC_REDIRECT_URL",
    "LISTEN_ADDRESS",
    "LISTEN_PORT",
    "ASSETS_PUBLIC_PATH",
    "ASSETS_PUBLIC_URL",
    "ASSETS_PRIVATE_PATH",
    "ASSETS_PRIVATE_URL",
    "ASSET_ALLOWED_EXTENSIONS",
    "ASSET_MAX_SIZE",
];

struct EnvGuard {
    previous: Vec<(&'static str, Option<String>)>,
}

impl EnvGuard {
    fn clear(vars: &'static [&'static str]) -> Self {
        let mut previous = Vec::with_capacity(vars.len());
        for key in vars {
            previous.push((*key, env::var(key).ok()));
            unsafe {
                env::remove_var(key);
            }
        }
        Self { previous }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        for (key, value) in &self.previous {
            match value {
                Some(v) => unsafe {
                    env::set_var(key, v);
                },
                None => unsafe {
                    env::remove_var(key);
                },
            }
        }
    }
}

struct CwdGuard {
    original: PathBuf,
    temp_dir: PathBuf,
}

impl CwdGuard {
    fn enter_temp_dir() -> Self {
        let original = env::current_dir().expect("failed to read current dir");
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before UNIX_EPOCH")
            .as_nanos();
        let temp_dir = env::temp_dir().join(format!("grade-o-matic-config-tests-{unique}"));
        fs::create_dir_all(&temp_dir).expect("failed to create temporary cwd");
        env::set_current_dir(&temp_dir).expect("failed to switch cwd");
        Self { original, temp_dir }
    }
}

impl Drop for CwdGuard {
    fn drop(&mut self) {
        let _ = env::set_current_dir(&self.original);
        let _ = fs::remove_dir_all(&self.temp_dir);
    }
}

fn set_env(key: &str, value: &str) {
    unsafe {
        env::set_var(key, value);
    }
}

#[test]
fn config_defaults_when_env_missing() {
    let _lock = GLOBAL_ENV_LOCK.lock().expect("poisoned env lock");
    let _cwd = CwdGuard::enter_temp_dir();
    let _env = EnvGuard::clear(TRACKED_VARS);

    let config = Config::from_env().expect("expected default config");

    assert_eq!(config.log, "info");
    assert!(!config.debug);
    assert_eq!(config.jwt_secret, "");
    assert_eq!(config.service_host, "0.0.0.0");
    assert_eq!(config.service_port, 3030);
    assert_eq!(config.assets_public_path, "apps/web/assets/public");
    assert_eq!(config.assets_public_url, "/assets/public");
    assert_eq!(config.assets_private_path, "apps/web/assets/private");
    assert_eq!(config.assets_private_url, "/assets/private");
    assert_eq!(config.asset_max_size, 50 * 1024 * 1024);
    assert!(!config.oidc_enabled);
    assert!(config.database_url.is_none());
}

#[test]
fn config_resolves_legacy_asset_paths_from_repo_root_layout() {
    let _lock = GLOBAL_ENV_LOCK.lock().expect("poisoned env lock");
    let _cwd = CwdGuard::enter_temp_dir();
    let _env = EnvGuard::clear(TRACKED_VARS);

    fs::create_dir_all("apps/web/assets/public").expect("failed to create public asset path");
    fs::create_dir_all("apps/web/assets/private").expect("failed to create private asset path");

    set_env("ASSETS_PUBLIC_PATH", "assets/public");
    set_env("ASSETS_PRIVATE_PATH", "assets/private");

    let config = Config::from_env().expect("expected legacy asset paths to resolve");

    assert_eq!(config.assets_public_path, "apps/web/assets/public");
    assert_eq!(config.assets_private_path, "apps/web/assets/private");
}

#[test]
fn config_fails_on_invalid_numeric_and_duration_values() {
    let _lock = GLOBAL_ENV_LOCK.lock().expect("poisoned env lock");
    let _cwd = CwdGuard::enter_temp_dir();
    let _env = EnvGuard::clear(TRACKED_VARS);

    set_env("LISTEN_PORT", "abc");
    let err = Config::from_env().expect_err("expected invalid LISTEN_PORT to fail");
    assert!(
        err.to_string().contains("LISTEN_PORT"),
        "unexpected error: {err}"
    );

    unsafe {
        env::remove_var("LISTEN_PORT");
    }
    set_env("DB_CONN_MAX_LIFETIME", "not-a-duration");
    let err = Config::from_env().expect_err("expected invalid DB_CONN_MAX_LIFETIME to fail");
    assert!(
        err.to_string().contains("DB_CONN_MAX_LIFETIME"),
        "unexpected error: {err}"
    );
}

#[test]
fn database_url_uses_explicit_value_when_present() {
    let _lock = GLOBAL_ENV_LOCK.lock().expect("poisoned env lock");
    let _cwd = CwdGuard::enter_temp_dir();
    let _env = EnvGuard::clear(TRACKED_VARS);

    set_env(
        "DATABASE_URL",
        "postgres://explicit:pw@db.example:5432/main",
    );
    set_env("DATABASE_USER", "builder");
    set_env("DATABASE_PASSWORD", "builderpw");
    set_env("DATABASE_HOST", "localhost");
    set_env("DATABASE_PORT", "5432");
    set_env("DATABASE_NAME", "ignored_db");

    let config = Config::from_env().expect("config should parse");
    assert_eq!(
        config.database_url.as_deref(),
        Some("postgres://explicit:pw@db.example:5432/main")
    );
}

#[test]
fn database_url_builds_from_parts_when_explicit_missing() {
    let _lock = GLOBAL_ENV_LOCK.lock().expect("poisoned env lock");
    let _cwd = CwdGuard::enter_temp_dir();
    let _env = EnvGuard::clear(TRACKED_VARS);

    set_env("DATABASE_USER", "myuser");
    set_env("DATABASE_PASSWORD", "mypass");
    set_env("DATABASE_HOST", "127.0.0.1");
    set_env("DATABASE_PORT", "5433");
    set_env("DATABASE_NAME", "mydb");

    let config = Config::from_env().expect("config should parse");
    assert_eq!(
        config.database_url.as_deref(),
        Some("postgres://myuser:mypass@127.0.0.1:5433/mydb")
    );
}

#[test]
fn oidc_requires_all_required_fields_when_enabled() {
    let _lock = GLOBAL_ENV_LOCK.lock().expect("poisoned env lock");
    let _cwd = CwdGuard::enter_temp_dir();
    let _env = EnvGuard::clear(TRACKED_VARS);

    set_env("OIDC_ENABLED", "true");
    let err = Config::from_env().expect_err("expected missing OIDC vars to fail");
    assert!(
        err.to_string()
            .contains("required OIDC_* variables are missing"),
        "unexpected error: {err}"
    );
}

#[test]
fn oidc_enabled_succeeds_when_all_required_fields_are_set() {
    let _lock = GLOBAL_ENV_LOCK.lock().expect("poisoned env lock");
    let _cwd = CwdGuard::enter_temp_dir();
    let _env = EnvGuard::clear(TRACKED_VARS);

    set_env("OIDC_ENABLED", "true");
    set_env("OIDC_ISSUER_URL", "https://issuer.example.com");
    set_env("OIDC_CLIENT_ID", "my-client-id");
    set_env("OIDC_REDIRECT_URL", "http://localhost:3030/callback");

    let config = Config::from_env().expect("expected valid OIDC config");
    assert!(config.oidc_enabled);
    assert_eq!(
        config.oidc_issuer_url.as_deref(),
        Some("https://issuer.example.com")
    );
    assert_eq!(config.oidc_client_id.as_deref(), Some("my-client-id"));
    assert_eq!(
        config.oidc_redirect_url.as_deref(),
        Some("http://localhost:3030/callback")
    );
}
