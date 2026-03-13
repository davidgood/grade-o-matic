use anyhow::{Context, Result, anyhow, bail};
use regex::Regex;
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::{env, path::Path, time::Duration};
use tokio::time::sleep;

/// Config is a struct that holds the configuration for the application.
#[derive(Clone, Debug)]
pub struct Config {
    pub log: String,
    pub debug: bool,
    pub jwt_secret: String,
    pub database_name: Option<String>,
    pub database_user: Option<String>,
    pub database_password: Option<String>,
    pub database_host: Option<String>,
    pub database_port: Option<u16>,
    pub database_max_connections: Option<u32>,
    pub database_min_connections: Option<u32>,
    pub database_conn_max_lifetime: Option<Duration>,
    pub database_url: Option<String>,

    pub oidc_enabled: bool,
    pub oidc_issuer_url: Option<String>,
    pub oidc_client_id: Option<String>,
    pub oidc_redirect_url: Option<String>,

    pub service_host: String,
    pub service_port: u16,

    pub assets_public_path: String,
    pub assets_public_url: String,

    pub assets_private_path: String,
    pub assets_private_url: String,

    pub asset_allowed_extensions_pattern: Regex,
    pub asset_max_size: usize,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();

        let ext_val = env_var_or_default("ASSET_ALLOWED_EXTENSIONS", "jpg|jpeg|png|gif|webp");
        let jwt_secret = env_var_or_default("JWT_SECRET", "");
        let database_name = env_var_optional("DATABASE_NAME");
        let database_user = env_var_optional("DATABASE_USER");
        let database_password = env_var_optional("DATABASE_PASSWORD");
        let database_host = env_var_optional("DATABASE_HOST");
        let database_port = parse_optional::<u16>("DATABASE_PORT")?;
        let database_url = build_database_url(
            env_var_optional("DATABASE_URL"),
            database_user.as_deref(),
            database_password.as_deref(),
            database_host.as_deref(),
            database_port,
            database_name.as_deref(),
        );

        let oidc_enabled = parse_bool_with_default("OIDC_ENABLED", false)?;
        let oidc_issuer_url = env_var_optional("OIDC_ISSUER_URL");
        let oidc_client_id = env_var_optional("OIDC_CLIENT_ID");
        let oidc_redirect_url = env_var_optional("OIDC_REDIRECT_URL");

        if oidc_enabled
            && (oidc_issuer_url.is_none()
                || oidc_client_id.is_none()
                || oidc_redirect_url.is_none())
        {
            bail!("OIDC is enabled, but one or more required OIDC_* variables are missing");
        }

        Ok(Self {
            log: env_var_or_default("RUST_LOG", "info"),
            debug: parse_bool_with_default("DEBUG", false)?,
            jwt_secret,
            service_host: env_var_or_default("LISTEN_ADDRESS", "0.0.0.0"),
            service_port: parse_with_default("LISTEN_PORT", 3030_u16)?,
            database_name,
            database_user,
            database_password,
            database_host,
            database_port,
            database_max_connections: parse_optional::<u32>("DB_MAX_OPEN_CONNS")?,
            database_min_connections: parse_optional::<u32>("DB_MAX_IDLE_CONNS")?,
            database_conn_max_lifetime: parse_duration_optional("DB_CONN_MAX_LIFETIME")?,

            database_url,

            oidc_enabled,
            oidc_issuer_url,
            oidc_client_id,
            oidc_redirect_url,

            assets_public_path: resolve_asset_path(&env_var_or_default(
                "ASSETS_PUBLIC_PATH",
                "apps/web/assets/public",
            )),
            assets_public_url: env_var_or_default("ASSETS_PUBLIC_URL", "/assets/public"),

            assets_private_path: resolve_asset_path(&env_var_or_default(
                "ASSETS_PRIVATE_PATH",
                "apps/web/assets/private",
            )),
            assets_private_url: env_var_or_default("ASSETS_PRIVATE_URL", "/assets/private"),

            asset_allowed_extensions_pattern: Regex::new(&format!(r"(?i)^.*\.({})$", ext_val))
                .unwrap_or_else(|_| {
                    eprintln!(
                        "Invalid ASSET_ALLOWED_EXTENSIONS regex pattern: {}",
                        ext_val
                    );
                    Regex::new(r"(?i)^.*\.(jpg|jpeg|png|gif|webp)$").unwrap()
                }),

            asset_max_size: parse_with_default("ASSET_MAX_SIZE", 50 * 1024 * 1024usize)?, // Default to 50MB
        })
    }
}

fn build_database_url(
    explicit_url: Option<String>,
    user: Option<&str>,
    password: Option<&str>,
    host: Option<&str>,
    port: Option<u16>,
    name: Option<&str>,
) -> Option<String> {
    if explicit_url.is_some() {
        return explicit_url;
    }

    match (user, password, host, port, name) {
        (Some(user), Some(password), Some(host), Some(port), Some(name)) => {
            Some(format!("postgres://{user}:{password}@{host}:{port}/{name}"))
        }
        _ => None,
    }
}

fn env_var_or_default(name: &str, default: &str) -> String {
    env::var(name).unwrap_or_else(|_| default.to_string())
}

fn env_var_optional(name: &str) -> Option<String> {
    env::var(name).ok().and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

fn resolve_asset_path(raw: &str) -> String {
    let configured_path = Path::new(raw);
    if configured_path.exists() {
        return raw.to_string();
    }

    let repo_relative_path = Path::new("apps/web").join(raw);
    if repo_relative_path.exists() {
        return repo_relative_path.to_string_lossy().into_owned();
    }

    raw.to_string()
}

fn parse_bool_with_default(name: &str, default: bool) -> Result<bool> {
    match env::var(name) {
        Ok(raw) => raw
            .parse::<bool>()
            .with_context(|| format!("{name} must be true or false")),
        Err(_) => Ok(default),
    }
}
fn parse_with_default<T>(name: &str, default: T) -> Result<T>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    match env::var(name) {
        Ok(raw) => raw
            .parse::<T>()
            .map_err(|err| anyhow!("{name} has an invalid value: {raw} ({err})")),
        Err(_) => Ok(default),
    }
}
fn parse_optional<T>(name: &str) -> Result<Option<T>>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    match env_var_optional(name) {
        Some(raw) => raw
            .parse::<T>()
            .map(Some)
            .map_err(|err| anyhow!("{name} has an invalid value: {raw} ({err})")),
        None => Ok(None),
    }
}

fn parse_duration_optional(name: &str) -> Result<Option<Duration>> {
    let Some(raw) = env_var_optional(name) else {
        return Ok(None);
    };

    let raw = raw.trim().to_lowercase();
    if raw.is_empty() {
        return Ok(None);
    }

    let (value, multiplier): (&str, u64) = if let Some(v) = raw.strip_suffix('s') {
        (v, 1)
    } else if let Some(v) = raw.strip_suffix('m') {
        (v, 60)
    } else if let Some(v) = raw.strip_suffix('h') {
        (v, 60 * 60)
    } else {
        (raw.as_str(), 1)
    };

    let seconds = value
        .parse::<u64>()
        .map_err(|err| anyhow!("{name} has an invalid duration value: {raw} ({err})"))?;

    Ok(Some(Duration::from_secs(seconds * multiplier)))
}

/// setup_database initializes the database connection pool.
pub async fn setup_database(config: &Config) -> Result<PgPool> {
    let database_url = config
        .database_url
        .as_deref()
        .context("Missing database configuration. Set DATABASE_URL or DATABASE_* parts")?;

    // Attempt to connect repeatedly, with a small delay, until success (or a max number of tries)
    let mut attempts = 0;
    let pool = loop {
        attempts += 1;
        match PgPoolOptions::new()
            .max_connections(config.database_max_connections.unwrap_or(5))
            .min_connections(config.database_min_connections.unwrap_or(1))
            .connect(database_url)
            .await
        {
            Ok(pool) => break pool,
            Err(err) => {
                if attempts >= 3 {
                    return Err(err.into());
                }
                eprintln!(
                    "Postgres not ready yet ({:?}), retrying in 1s… (attempt {}/{})",
                    err, attempts, 3
                );
                sleep(Duration::from_secs(1)).await;
            }
        }
    };

    Ok(pool)
}
