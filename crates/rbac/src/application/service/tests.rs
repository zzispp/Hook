use types::{
    pagination::PageRequest,
    rbac::{RbacListFilters, RbacListRequest, Role},
};

use super::{
    test_fixtures::{api_input, api_permission, menu_item, menu_item_input, menu_section, permission_snapshot, rbac_id, role_input},
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
        authenticated: vec![],
    };

    service.authorize_api(&config, api_request("GET", "/health", "user")).await.unwrap();
}

#[tokio::test]
async fn authorize_api_allows_whitelisted_prefix_without_matching_sibling_prefixes() {
    let service = test_service();
    let config = AuthorizationConfig {
        whitelist: vec![AuthWhitelistRule {
            methods: vec!["POST".into()],
            path_pattern: "/v1/*".into(),
        }],
        authenticated: vec![],
    };

    service
        .authorize_api(&config, api_request("POST", "/v1/chat/completions", "user"))
        .await
        .unwrap();
    service
        .authorize_api(&config, api_request("POST", "/v1/chat/completions/", "user"))
        .await
        .unwrap();

    assert!(!service.is_whitelisted(&config, "POST", "/v10/chat/completions").unwrap());
}

#[tokio::test]
async fn authorize_api_allows_authenticated_base_path_without_cache() {
    let service = test_service();
    let config = AuthorizationConfig {
        whitelist: vec![],
        authenticated: vec![AuthWhitelistRule {
            methods: vec!["GET".into()],
            path_pattern: "/api/navbar".into(),
        }],
    };

    service.authorize_api(&config, api_request("GET", "/api/navbar", "user")).await.unwrap();
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
    let repository = MemoryRbacRepository::with_menu_state(
        vec![menu_section(
            1,
            types::rbac::MenuSectionInput {
                code: "system".into(),
                subheader: "System".into(),
                sort_order: 0,
                enabled: true,
            },
        )],
        vec![],
    );
    let cache = MemoryRbacCache::default();
    let service = RbacService::new(repository, cache.clone());

    service.create_menu_item(menu_item_input("users")).await.unwrap();

    assert_eq!(cache.write_count(), 1);
}

#[tokio::test]
async fn page_apis_returns_repository_page() {
    let repository = MemoryRbacRepository::with_apis(vec![api_permission(1, api_input("users_read")), api_permission(2, api_input("users_write"))]);
    let service = RbacService::new(repository, MemoryRbacCache::default());

    let page = service.page_apis(test_list_request(1)).await.unwrap();

    assert_eq!(page.items[0].code, "users_read");
    assert_eq!(page.total, 2);
}

#[tokio::test]
async fn page_unbound_apis_excludes_menu_bound_permissions() {
    let bound_api = rbac_id(1);
    let unbound_api = rbac_id(2);
    let repository = MemoryRbacRepository::with_apis_and_menu_api_bindings(
        vec![api_permission(1, api_input("bound")), api_permission(2, api_input("unbound"))],
        rbac_id(3),
        vec![bound_api],
    );
    let service = RbacService::new(repository, MemoryRbacCache::default());

    let page = service.page_unbound_apis(test_list_request(10)).await.unwrap();

    assert_eq!(page.items.iter().map(|api| api.id.as_str()).collect::<Vec<_>>(), vec![unbound_api]);
    assert_eq!(page.total, 1);
}

#[tokio::test]
async fn role_permission_reads_return_current_ids() {
    let repository = MemoryRbacRepository::with_role_bindings("admin", vec!["menu-1".into()]);
    let service = RbacService::new(repository, MemoryRbacCache::default());

    let permissions = service.role_permission_bindings("admin").await.unwrap();

    assert_eq!(permissions.menu_item_ids, vec!["menu-1"]);
    assert!(permissions.api_permission_ids.is_empty());
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

#[tokio::test]
async fn ensure_system_role_marks_existing_role_as_system() {
    let repository = MemoryRbacRepository::with_role(Role {
        code: "admin".into(),
        name: "Old Admin".into(),
        description: String::new(),
        enabled: true,
        system: false,
        sort_order: 99,
    });
    let service = RbacService::new(repository, MemoryRbacCache::default());

    let role = service.ensure_system_role(role_input("admin")).await.unwrap();

    assert!(role.system);
    assert_eq!(role.sort_order, 0);
}

#[tokio::test]
async fn delete_api_rejects_bound_permission() {
    let api_id = rbac_id(1);
    let repository = MemoryRbacRepository::with_menu_api_bindings(rbac_id(2), vec![api_id.clone()]);
    let service = RbacService::new(repository, MemoryRbacCache::default());

    let result = service.delete_api(&api_id).await;

    assert!(matches!(result, Err(RbacError::Conflict(_))));
}

#[tokio::test]
async fn delete_api_rejects_role_bound_permission() {
    let api_id = rbac_id(1);
    let repository = MemoryRbacRepository::with_role_api_bindings("admin", vec![api_id.clone()]);
    let service = RbacService::new(repository, MemoryRbacCache::default());

    let result = service.delete_api(&api_id).await;

    assert!(matches!(result, Err(RbacError::Conflict(_))));
}

#[tokio::test]
async fn delete_menu_item_rejects_role_bound_item() {
    let item_id = rbac_id(1);
    let repository = MemoryRbacRepository::with_role_bindings("admin", vec![item_id.clone()]);
    let service = RbacService::new(repository, MemoryRbacCache::default());

    let result = service.delete_menu_item(&item_id).await;

    assert!(matches!(result, Err(RbacError::Conflict(_))));
}

#[tokio::test]
async fn delete_menu_section_rejects_non_empty_section() {
    let section = menu_section(
        1,
        types::rbac::MenuSectionInput {
            code: "system".into(),
            subheader: "System".into(),
            sort_order: 0,
            enabled: true,
        },
    );
    let item = menu_item(1, menu_item_input("users"));
    let repository = MemoryRbacRepository::with_menu_state(vec![section.clone()], vec![item]);
    let service = RbacService::new(repository, MemoryRbacCache::default());

    let result = service.delete_menu_section(&section.id).await;

    assert!(matches!(result, Err(RbacError::Conflict(_))));
}

#[tokio::test]
async fn replace_menu_apis_rejects_unknown_api_id() {
    let menu_id = rbac_id(1);
    let repository = MemoryRbacRepository::with_menu_state(vec![], vec![menu_item(1, menu_item_input("users"))]);
    let service = RbacService::new(repository, MemoryRbacCache::default());

    let result = service
        .replace_menu_apis(
            &menu_id,
            types::rbac::MenuApiBindingInput {
                api_permission_ids: vec![rbac_id(404)],
            },
        )
        .await;

    assert!(matches!(result, Err(RbacError::InvalidInput(_))));
}

#[tokio::test]
async fn create_menu_item_rejects_unknown_section() {
    let service = test_service();

    let result = service.create_menu_item(menu_item_input("users")).await;

    assert!(matches!(result, Err(RbacError::InvalidInput(_))));
}

#[tokio::test]
async fn replace_menu_item_rejects_self_parent() {
    let mut input = menu_item_input("users");
    input.parent_id = Some(rbac_id(1));
    let existing = menu_item(1, menu_item_input("users"));
    let repository = MemoryRbacRepository::with_menu_state(
        vec![menu_section(
            1,
            types::rbac::MenuSectionInput {
                code: "system".into(),
                subheader: "System".into(),
                sort_order: 0,
                enabled: true,
            },
        )],
        vec![existing.clone()],
    );
    let service = RbacService::new(repository, MemoryRbacCache::default());

    let result = service.replace_menu_item(&existing.id, input).await;

    assert!(matches!(result, Err(RbacError::InvalidInput(_))));
}

fn test_service() -> RbacService<MemoryRbacRepository, MemoryRbacCache> {
    RbacService::new(MemoryRbacRepository::default(), MemoryRbacCache::default())
}

fn empty_config() -> AuthorizationConfig {
    AuthorizationConfig {
        whitelist: vec![],
        authenticated: vec![],
    }
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

fn test_list_request(page_size: u64) -> RbacListRequest {
    RbacListRequest {
        page: PageRequest { page: 1, page_size },
        filters: RbacListFilters::default(),
    }
}
