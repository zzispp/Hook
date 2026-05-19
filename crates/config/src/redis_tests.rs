use config_rs::{Config, File, FileFormat};

use super::RedisSettings;

#[test]
fn scheduling_snapshot_ttl_defaults_to_disabled() {
    let settings: RedisSettings = Config::builder()
        .add_source(File::from_str(redis_yaml_without_snapshot_ttl(), FileFormat::Yaml))
        .build()
        .unwrap()
        .try_deserialize()
        .unwrap();

    assert_eq!(settings.scheduling_snapshot_ttl_seconds, 0);
}

fn redis_yaml_without_snapshot_ttl() -> &'static str {
    r#"
url: "redis://default:@localhost:6380?protocol=resp3"
scheme: "redis"
host: "localhost"
port: 6380
username: "default"
password: ""
database:
protocol: "resp3"
key_prefix: "hook"
"#
}
