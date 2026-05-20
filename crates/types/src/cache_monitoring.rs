use serde::{Deserialize, Serialize};

const DEFAULT_PAGE_NUMBER: u64 = 1;
const DEFAULT_PAGE_SIZE: u64 = 20;

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct CacheAffinityListRequest {
    #[serde(default = "default_page_number")]
    pub page: u64,
    #[serde(default = "default_page_size")]
    pub page_size: u64,
    #[serde(default)]
    pub search: Option<String>,
}

impl Default for CacheAffinityListRequest {
    fn default() -> Self {
        Self {
            page: default_page_number(),
            page_size: default_page_size(),
            search: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CacheAffinityItem {
    pub affinity_key: String,
    pub user_id: Option<String>,
    pub username: Option<String>,
    pub user_email: Option<String>,
    pub token_name: Option<String>,
    pub token_prefix: Option<String>,
    pub provider_id: String,
    pub provider_name: Option<String>,
    pub endpoint_id: String,
    pub endpoint_base_url: Option<String>,
    pub provider_key_id: String,
    pub provider_key_name: Option<String>,
    pub model_id: String,
    pub model_name: Option<String>,
    pub api_format: String,
    pub ttl_seconds: i64,
    pub request_count: i64,
}

fn default_page_number() -> u64 {
    DEFAULT_PAGE_NUMBER
}

fn default_page_size() -> u64 {
    DEFAULT_PAGE_SIZE
}
