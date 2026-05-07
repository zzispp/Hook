use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct PageRequest {
    pub page: u64,
    pub page_size: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PageSliceRequest {
    pub offset: u64,
    pub limit: u64,
    pub page: u64,
    pub page_size: u64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
}
