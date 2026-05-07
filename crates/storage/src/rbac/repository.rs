use types::pagination::{Page, PageSliceRequest};

use crate::Database;

#[derive(Clone)]
pub struct RbacStore {
    pub(super) database: Database,
}

impl RbacStore {
    pub fn new(database: Database) -> Self {
        Self { database }
    }
}

pub(super) fn rbac_page<T>(items: Vec<T>, total: u64, request: PageSliceRequest) -> Page<T> {
    Page {
        items,
        total,
        page: request.page,
        page_size: request.page_size,
    }
}
