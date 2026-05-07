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
    pub secret_env: String,
    pub access_token_ttl_seconds: u64,
    pub refresh_token_ttl_seconds: u64,
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
    #[error("jwt secret environment variable {0} is not set")]
    MissingJwtSecret(String),
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
        env::var(&self.jwt.secret_env).map_err(|_| SettingsError::MissingJwtSecret(self.jwt.secret_env.clone()))
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

#[cfg(test)]
mod tests {
    use super::{DatabaseSettings, JwtSettings, MODULE_CONFIG_PATH, ROOT_CONFIG_PATH, ServerSettings, Settings, SettingsError, explicit_config_path};
    use std::{ffi::OsString, path::PathBuf};

    #[test]
    fn database_url_prefers_explicit_url() {
        let settings = settings_with_database(DatabaseSettings {
            url: Some("postgres://user:pass@remote:5432/app".into()),
            password: Some("ignored".into()),
            ..database_parts()
        });

        let url = settings.database_url().unwrap();

        assert_eq!(url, "postgres://user:pass@remote:5432/app");
    }

    #[test]
    fn database_url_uses_parts_when_url_is_missing() {
        let settings = settings_with_database(database_parts());

        let url = settings.database_url().unwrap();

        assert_eq!(url, "postgres://postgres:123456@localhost:5433/postgres");
    }

    #[test]
    fn database_url_uses_parts_when_url_is_blank() {
        let settings = settings_with_database(DatabaseSettings {
            url: Some("  ".into()),
            ..database_parts()
        });

        let url = settings.database_url().unwrap();

        assert_eq!(url, "postgres://postgres:123456@localhost:5433/postgres");
    }

    #[test]
    fn database_url_errors_without_password_when_url_is_missing() {
        let settings = settings_with_database(DatabaseSettings {
            password: None,
            ..database_parts()
        });

        let result = settings.database_url();

        assert!(matches!(result, Err(SettingsError::MissingDatabasePassword)));
    }

    #[test]
    fn explicit_config_path_reads_path_after_config_arg() {
        let args = vec![OsString::from("backend"), OsString::from("--config"), OsString::from("custom.yaml")];

        let path = explicit_config_path(&args).unwrap();

        assert_eq!(path, Some(PathBuf::from("custom.yaml")));
    }

    #[test]
    fn explicit_config_path_errors_without_value() {
        let args = vec![OsString::from("backend"), OsString::from("--config")];

        let result = explicit_config_path(&args);

        assert!(matches!(result, Err(SettingsError::MissingConfigArgument)));
    }

    #[test]
    fn default_config_paths_are_ordered() {
        assert_eq!(MODULE_CONFIG_PATH, "config/config.yaml");
        assert_eq!(ROOT_CONFIG_PATH, "config.yaml");
    }

    fn settings_with_database(database: DatabaseSettings) -> Settings {
        Settings {
            server: ServerSettings {
                host: "127.0.0.1".into(),
                port: 3000,
            },
            database,
            jwt: jwt_settings(),
        }
    }

    fn database_parts() -> DatabaseSettings {
        DatabaseSettings {
            url: None,
            scheme: "postgres".into(),
            host: "localhost".into(),
            port: 5433,
            username: "postgres".into(),
            password: Some("123456".into()),
            name: "postgres".into(),
        }
    }

    fn jwt_settings() -> JwtSettings {
        JwtSettings {
            secret_env: "HOOK_JWT_SECRET".into(),
            access_token_ttl_seconds: 900,
            refresh_token_ttl_seconds: 604800,
        }
    }
}
