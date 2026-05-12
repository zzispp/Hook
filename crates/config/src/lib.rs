use config_rs::{Config, File};
use serde::Deserialize;
use std::{
    env,
    path::{Path, PathBuf},
};
use thiserror::Error;

const CONFIG_ARG: &str = "--config";
const MODULE_CONFIG_PATH: &str = "config/config.yaml";
const ROOT_CONFIG_PATH: &str = "config.yaml";

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct Settings {
    pub server: ServerSettings,
    pub database: DatabaseSettings,
    pub jwt: JwtSettings,
    pub admin: AdminSettings,
    pub auth: AuthSettings,
    pub security: SecuritySettings,
    pub redis: RedisSettings,
    pub tracing: TracingSettings,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct ServerSettings {
    pub host: String,
    pub port: u16,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct DatabaseSettings {
    pub url: Option<String>,
    pub scheme: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: Option<String>,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct JwtSettings {
    pub secret: String,
    pub access_token_ttl_seconds: u64,
    pub refresh_token_ttl_seconds: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct AdminSettings {
    pub id: String,
    pub username: String,
    pub email: String,
    pub role: String,
    pub is_active: bool,
    pub password_hash: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct AuthSettings {
    pub whitelist: Vec<AuthWhitelistRule>,
    pub authenticated: Vec<AuthWhitelistRule>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct AuthWhitelistRule {
    pub methods: Vec<String>,
    pub path_pattern: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct SecuritySettings {
    pub provider_key_secret: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct RedisSettings {
    pub url: Option<String>,
    pub scheme: String,
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
    pub database: Option<u16>,
    pub protocol: Option<String>,
    pub key_prefix: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct TracingSettings {
    pub log_level: String,
}

#[derive(Debug, Error)]
pub enum SettingsError {
    #[error("configuration error: {0}")]
    Config(#[from] config_rs::ConfigError),
    #[error("database.password is required when database.url is not set")]
    MissingDatabasePassword,
    #[error("configuration file not found")]
    MissingConfigFile,
    #[error("--config requires a file path")]
    MissingConfigArgument,
    #[error("{0} cannot be blank")]
    BlankConfigValue(&'static str),
}

impl Settings {
    pub fn load() -> Result<Self, SettingsError> {
        Self::load_from_args(env::args_os())
    }

    pub fn load_from_args<I, S>(args: I) -> Result<Self, SettingsError>
    where
        I: IntoIterator<Item = S>,
        S: Into<std::ffi::OsString>,
    {
        let path = resolve_config_path(args)?;
        Config::builder()
            .add_source(File::from(path))
            .build()?
            .try_deserialize()
            .map_err(SettingsError::from)
    }

    pub fn bind_addr(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }

    pub fn database_url(&self) -> Result<String, SettingsError> {
        if let Some(url) = non_empty_database_url(self.database.url.as_deref()) {
            return Ok(url.to_owned());
        }

        let password = self.database.password.as_ref().ok_or(SettingsError::MissingDatabasePassword)?;
        Ok(format!(
            "{}://{}:{}@{}:{}/{}",
            self.database.scheme, self.database.username, password, self.database.host, self.database.port, self.database.name
        ))
    }

    pub fn jwt_secret(&self) -> Result<String, SettingsError> {
        required_config_value("jwt.secret", &self.jwt.secret)
    }

    pub fn admin_password_hash(&self) -> Result<String, SettingsError> {
        required_config_value("admin.password_hash", &self.admin.password_hash)
    }

    pub fn provider_key_secret(&self) -> Result<String, SettingsError> {
        required_config_value("security.provider_key_secret", &self.security.provider_key_secret)
    }

    pub fn redis_url(&self) -> Result<String, SettingsError> {
        if let Some(url) = non_empty_config_url(self.redis.url.as_deref()) {
            return Ok(url.to_owned());
        }

        Ok(format!(
            "{}://{}{}{}{}",
            self.redis.scheme,
            redis_auth(&self.redis),
            self.redis.host,
            redis_port(self.redis.port),
            redis_query(&self.redis)
        ))
    }

    pub fn tracing_log_level(&self) -> Result<String, SettingsError> {
        required_config_value("tracing.log_level", &self.tracing.log_level)
    }
}

fn resolve_config_path<I, S>(args: I) -> Result<PathBuf, SettingsError>
where
    I: IntoIterator<Item = S>,
    S: Into<std::ffi::OsString>,
{
    let args = args.into_iter().map(Into::into).collect::<Vec<std::ffi::OsString>>();
    if let Some(path) = explicit_config_path(&args)? {
        return Ok(path);
    }

    first_existing_path([MODULE_CONFIG_PATH, ROOT_CONFIG_PATH])
}

fn explicit_config_path(args: &[std::ffi::OsString]) -> Result<Option<PathBuf>, SettingsError> {
    let Some(index) = args.iter().position(|arg| arg == CONFIG_ARG) else {
        return Ok(None);
    };

    args.get(index + 1).map(PathBuf::from).map(Some).ok_or(SettingsError::MissingConfigArgument)
}

fn first_existing_path(paths: [&str; 2]) -> Result<PathBuf, SettingsError> {
    paths
        .into_iter()
        .map(PathBuf::from)
        .find(|path| Path::new(path).is_file())
        .ok_or(SettingsError::MissingConfigFile)
}

fn non_empty_database_url(url: Option<&str>) -> Option<&str> {
    match url {
        Some(value) if !value.trim().is_empty() => Some(value),
        _ => None,
    }
}

fn non_empty_config_url(url: Option<&str>) -> Option<&str> {
    match url {
        Some(value) if !value.trim().is_empty() => Some(value.trim()),
        _ => None,
    }
}

fn redis_auth(settings: &RedisSettings) -> String {
    let Some(username) = settings.username.as_deref().map(str::trim).filter(|value| !value.is_empty()) else {
        return String::new();
    };

    match settings.password.as_deref() {
        Some(password) => format!("{username}:{password}@"),
        None => format!("{username}@"),
    }
}

fn redis_port(port: u16) -> String {
    format!(":{port}")
}

fn redis_query(settings: &RedisSettings) -> String {
    let path = settings.database.map(|database| format!("/{database}")).unwrap_or_default();
    let query = settings
        .protocol
        .as_deref()
        .map(str::trim)
        .filter(|protocol| !protocol.is_empty())
        .map(|protocol| format!("?protocol={protocol}"))
        .unwrap_or_default();
    format!("{path}{query}")
}

fn required_config_value(key: &'static str, value: &str) -> Result<String, SettingsError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(SettingsError::BlankConfigValue(key));
    }

    Ok(trimmed.to_owned())
}

#[cfg(test)]
mod tests;
