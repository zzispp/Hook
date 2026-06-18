use std::{
    net::TcpListener,
    path::PathBuf,
    process::{Child, Command, Stdio},
    sync::{Mutex, OnceLock},
    time::Duration,
};

use uuid::Uuid;

use super::store::{CodexChatHistoryStore, DEFAULT_HISTORY_TTL_SECONDS};

static REDIS: OnceLock<Mutex<TestRedis>> = OnceLock::new();

pub(crate) async fn test_history() -> CodexChatHistoryStore {
    test_history_with_ttl(DEFAULT_HISTORY_TTL_SECONDS).await
}

pub(crate) async fn test_history_with_ttl(ttl_seconds: u64) -> CodexChatHistoryStore {
    let url = redis_url();
    let client = redis::Client::open(url.as_str()).expect("test redis url must be valid");
    let connection = client.get_connection_manager().await.expect("test redis must accept connection");
    CodexChatHistoryStore::with_ttl_seconds(connection, format!("test-{}", Uuid::now_v7()), ttl_seconds)
}

fn redis_url() -> String {
    let redis = REDIS.get_or_init(|| Mutex::new(TestRedis::start()));
    redis.lock().expect("test redis mutex must not be poisoned").url.clone()
}

struct TestRedis {
    url: String,
    _child: Child,
    _dir: PathBuf,
}

impl TestRedis {
    fn start() -> Self {
        let port = available_port();
        let dir = std::env::temp_dir().join(format!("hook-codex-history-redis-{port}"));
        std::fs::create_dir_all(&dir).expect("test redis dir must be created");
        let child = Command::new("redis-server")
            .arg("--port")
            .arg(port.to_string())
            .arg("--save")
            .arg("")
            .arg("--appendonly")
            .arg("no")
            .arg("--dir")
            .arg(&dir)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("redis-server must be installed for Redis-backed history tests");
        let url = format!("redis://127.0.0.1:{port}/");
        wait_until_ready(&url);
        Self { url, _child: child, _dir: dir }
    }
}

impl Drop for TestRedis {
    fn drop(&mut self) {
        let _ = self._child.kill();
        let _ = self._child.wait();
        let _ = std::fs::remove_dir_all(&self._dir);
    }
}

fn available_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .expect("ephemeral redis test port must be available")
        .local_addr()
        .expect("ephemeral redis test listener must have local addr")
        .port()
}

fn wait_until_ready(url: &str) {
    for _ in 0..50 {
        if redis_ping(url) {
            return;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    panic!("test redis did not become ready");
}

fn redis_ping(url: &str) -> bool {
    let Ok(client) = redis::Client::open(url) else {
        return false;
    };
    let Ok(mut connection) = client.get_connection() else {
        return false;
    };
    redis::cmd("PING").query::<String>(&mut connection).is_ok()
}
