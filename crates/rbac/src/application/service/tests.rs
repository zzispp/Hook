use types::{pagination::PageRequest, rbac::Role};

use super::{
    test_fixtures::{api_input, api_permission, menu_item_input, permission_snapshot, role_input},
    test_support::{MemoryRbacCache, MemoryRbacRepository},
};
use crate::application::{ApiCheckRequest, AuthWhitelistRule, AuthorizationConfig, RbacError, RbacService};

#[tokio::test]
async fn authorize_api_allows_whitelisted_path_without_cache() {
    let service = test_service();
    let config = AuthorizationConfig {
        whitelist: vec![AuthWhitelistRule {
            methods: vec!["GET".into()],
            path_pattern: "/health".into(),
        }],
    };

    service.authorize_api(&config, api_request("GET", "/health", "user")).await.unwrap();
}

#[tokio::test]
async fn authorize_api_uses_cached_permission_snapshot() {
    let cache = MemoryRbacCache::with_snapshot(permission_snapshot());
    let service = RbacService::new(MemoryRbacRepository::default(), cache);

    service
        .authorize_api(&empty_config(), api_request("PUT", "/api/users/7", "admin"))
        .await
        .unwrap();
}

#[tokio::test]
async fn authorize_api_rejects_unbound_role() {
    let cache = MemoryRbacCache::with_snapshot(permission_snapshot());
    let service = RbacService::new(MemoryRbacRepository::default(), cache);

    let result = service.authorize_api(&empty_config(), api_request("PUT", "/api/users/7", "user")).await;

    assert!(matches!(result, Err(RbacError::Forbidden)));
}

#[tokio::test]
async fn authorize_api_allows_system_user_without_cache() {
    let service = test_service();

    service
        .authorize_api(&empty_config(), system_api_request("DELETE", "/api/rbac/apis/1"))
        .await
        .unwrap();
}

#[tokio::test]
async fn navbar_reads_role_menu_from_cache() {
    let cache = MemoryRbacCache::with_snapshot(permission_snapshot());
    let service = RbacService::new(MemoryRbacRepository::default(), cache);

    let nav = service.navbar("admin").await.unwrap();

    assert_eq!(nav.nav_items[0].subheader, "Management");
    assert_eq!(nav.nav_items[0].items[0].title, "Users");
}

#[tokio::test]
async fn mutating_role_rebuilds_cache() {
    let repository = MemoryRbacRepository::default();
    let cache = MemoryRbacCache::default();
    let service = RbacService::new(repository, cache.clone());

    service.create_role(role_input("manager")).await.unwrap();

    assert_eq!(cache.write_count(), 1);
}

#[tokio::test]
async fn mutating_menu_item_rebuilds_cache() {
    let repository = MemoryRbacRepository::default();
    let cache = MemoryRbacCache::default();
    let service = RbacService::new(repository, cache.clone());

    service.create_menu_item(menu_item_input("users")).await.unwrap();

    assert_eq!(cache.write_count(), 1);
}

#[tokio::test]
async fn page_apis_returns_repository_page() {
    let repository = MemoryRbacRepository::with_apis(vec![api_permission(1, api_input("users_read")), api_permission(2, api_input("users_write"))]);
    let service = RbacService::new(repository, MemoryRbacCache::default());

    let page = service.page_apis(PageRequest { page: 1, page_size: 1 }).await.unwrap();

    assert_eq!(page.items[0].code, "users_read");
    assert_eq!(page.total, 2);
}

#[tokio::test]
async fn system_role_cannot_be_deleted() {
    let repository = MemoryRbacRepository::with_role(Role {
        code: "admin".into(),
        name: "Admin".into(),
        description: String::new(),
        enabled: true,
        system: true,
        sort_order: 0,
    });
    let service = RbacService::new(repository, MemoryRbacCache::default());

    let result = service.delete_role("admin").await;

    assert!(matches!(result, Err(RbacError::Conflict(_))));
}

fn test_service() -> RbacService<MemoryRbacRepository, MemoryRbacCache> {
    RbacService::new(MemoryRbacRepository::default(), MemoryRbacCache::default())
}

fn empty_config() -> AuthorizationConfig {
    AuthorizationConfig { whitelist: vec![] }
}

fn api_request(method: &str, path: &str, role_code: &str) -> ApiCheckRequest {
    ApiCheckRequest {
        method: method.into(),
        path: path.into(),
        role_code: role_code.into(),
        system: false,
    }
}

fn system_api_request(method: &str, path: &str) -> ApiCheckRequest {
    ApiCheckRequest {
        method: method.into(),
        path: path.into(),
        role_code: "admin".into(),
        system: true,
    }
}
