use crate::llm_proxy::LlmProxyError;

use super::redis_error;

pub async fn write(connection: &mut redis::aio::ConnectionManager, key: String, value: String, ttl_seconds: u64) -> Result<(), LlmProxyError> {
    let _: () = command(key, value, ttl_seconds).query_async(connection).await.map_err(redis_error)?;
    Ok(())
}

fn command(key: String, value: String, ttl_seconds: u64) -> redis::Cmd {
    let mut command = redis::cmd("SET");
    command.arg(key).arg(value);
    if ttl_seconds > 0 {
        command.arg("EX").arg(ttl_seconds);
    }
    command
}

#[cfg(test)]
mod tests {
    use super::command;

    #[test]
    fn scheduling_snapshot_write_uses_plain_set_when_ttl_is_disabled() {
        let command = command("snapshot-key".into(), "snapshot-json".into(), 0);

        let packed = String::from_utf8(command.get_packed_command()).unwrap();

        assert_eq!(packed, "*3\r\n$3\r\nSET\r\n$12\r\nsnapshot-key\r\n$13\r\nsnapshot-json\r\n");
    }

    #[test]
    fn scheduling_snapshot_write_sets_expiration_when_ttl_is_positive() {
        let command = command("snapshot-key".into(), "snapshot-json".into(), 3600);

        let packed = String::from_utf8(command.get_packed_command()).unwrap();

        assert_eq!(
            packed,
            "*5\r\n$3\r\nSET\r\n$12\r\nsnapshot-key\r\n$13\r\nsnapshot-json\r\n$2\r\nEX\r\n$4\r\n3600\r\n"
        );
    }
}
