use super::{
    AdminSettings, AuthSettings, DatabaseSettings, JwtSettings, MODULE_CONFIG_PATH, ROOT_CONFIG_PATH, RedisSettings, ServerSettings, Settings, SettingsError,
    explicit_config_path,
};
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
fn jwt_secret_trims_config_secret() {
    let settings = settings_with_jwt(JwtSettings {
        secret: "  secret-from-config  ".into(),
        ..jwt_settings()
    });

    let secret = settings.jwt_secret().unwrap();

    assert_eq!(secret, "secret-from-config");
}

#[test]
fn jwt_secret_errors_when_blank() {
    let settings = settings_with_jwt(JwtSettings {
        secret: "  ".into(),
        ..jwt_settings()
    });

    let result = settings.jwt_secret();

    assert!(matches!(result, Err(SettingsError::BlankConfigValue("jwt.secret"))));
}

#[test]
fn admin_password_hash_trims_config_value() {
    let settings = settings_with_admin(AdminSettings {
        password_hash: "  hash-from-config  ".into(),
        ..admin_settings()
    });

    let password_hash = settings.admin_password_hash().unwrap();

    assert_eq!(password_hash, "hash-from-config");
}

#[test]
fn redis_url_trims_config_value() {
    let settings = settings_with_redis(RedisSettings {
        url: Some("  redis://localhost:6379/0  ".into()),
        ..redis_settings()
    });

    let url = settings.redis_url().unwrap();

    assert_eq!(url, "redis://localhost:6379/0");
}

#[test]
fn redis_url_uses_parts_when_url_is_missing() {
    let settings = settings_with_redis(RedisSettings { url: None, ..redis_settings() });

    let url = settings.redis_url().unwrap();

    assert_eq!(url, "redis://default:@localhost:6380?protocol=resp3");
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
        admin: admin_settings(),
        auth: AuthSettings { whitelist: vec![] },
        redis: redis_settings(),
    }
}

fn settings_with_jwt(jwt: JwtSettings) -> Settings {
    Settings {
        jwt,
        ..settings_with_database(database_parts())
    }
}

fn settings_with_admin(admin: AdminSettings) -> Settings {
    Settings {
        admin,
        ..settings_with_database(database_parts())
    }
}

fn settings_with_redis(redis: RedisSettings) -> Settings {
    Settings {
        redis,
        ..settings_with_database(database_parts())
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
        secret: "jwt-secret-from-config".into(),
        access_token_ttl_seconds: 900,
        refresh_token_ttl_seconds: 604800,
    }
}

fn admin_settings() -> AdminSettings {
    AdminSettings {
        id: "00000000-0000-7000-8000-000000000000".into(),
        username: "admin".into(),
        email: "admin@example.com".into(),
        role: "admin".into(),
        is_active: true,
        password_hash: "admin-password-hash-from-config".into(),
    }
}

fn redis_settings() -> RedisSettings {
    RedisSettings {
        url: Some("redis://default:@localhost:6380?protocol=resp3".into()),
        scheme: "redis".into(),
        host: "localhost".into(),
        port: 6380,
        username: Some("default".into()),
        password: Some(String::new()),
        database: None,
        protocol: Some("resp3".into()),
        key_prefix: "hook".into(),
    }
}
