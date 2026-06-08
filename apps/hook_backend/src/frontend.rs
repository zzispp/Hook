use axum::{
    Router,
    http::{Method, StatusCode, Uri, header},
    response::{IntoResponse, Response},
};
use rust_embed::{Embed, EmbeddedFile};

const INDEX_FILE: &str = "index.html";
const NOT_FOUND_FILE: &str = "404.html";
const METHOD_NOT_ALLOWED_BODY: &str = "Method Not Allowed";
const NOT_FOUND_BODY: &str = "404 Not Found";
const RESERVED_BACKEND_PREFIXES: [&str; 3] = ["/api", "/v1", "/v1beta"];

#[derive(Embed)]
#[folder = "../hook_frontend/out"]
#[allow_missing = true]
struct FrontendAssets;

pub fn create_router() -> Router {
    Router::new().fallback(static_handler)
}

pub fn ensure_assets() -> Result<(), &'static str> {
    if FrontendAssets::get(INDEX_FILE).is_none() {
        return Err("embedded frontend index asset is missing; run `pnpm build:frontend:embedded` before starting or packaging the backend");
    }

    if FrontendAssets::get(NOT_FOUND_FILE).is_none() {
        return Err("embedded frontend 404 asset is missing; run `pnpm build:frontend:embedded` before starting or packaging the backend");
    }

    Ok(())
}

async fn static_handler(method: Method, uri: Uri) -> Response {
    if method != Method::GET && method != Method::HEAD {
        return (StatusCode::METHOD_NOT_ALLOWED, METHOD_NOT_ALLOWED_BODY).into_response();
    }

    if is_reserved_backend_path(uri.path()) {
        return not_found();
    }

    let Some(path) = normalized_asset_path(uri.path()) else {
        return not_found();
    };

    match find_asset(&path) {
        Some(file) => asset_response(file),
        None if should_serve_frontend_not_found(&path) => frontend_not_found(),
        None => not_found(),
    }
}

fn normalized_asset_path(path: &str) -> Option<String> {
    if path.contains('\\') {
        return None;
    }

    let trimmed = path.trim_start_matches('/').trim_end_matches('/');
    if trimmed.split('/').any(|segment| segment == "..") {
        return None;
    }

    Some(if trimmed.is_empty() { INDEX_FILE.into() } else { trimmed.into() })
}

fn find_asset(path: &str) -> Option<EmbeddedFile> {
    if let Some(file) = FrontendAssets::get(path) {
        return Some(file);
    }

    if has_file_extension(path) {
        return None;
    }

    FrontendAssets::get(&format!("{path}/{INDEX_FILE}"))
}

fn has_file_extension(path: &str) -> bool {
    path.rsplit('/').next().is_some_and(|segment| segment.contains('.'))
}

fn should_serve_frontend_not_found(path: &str) -> bool {
    !has_file_extension(path)
}

fn is_reserved_backend_path(path: &str) -> bool {
    RESERVED_BACKEND_PREFIXES
        .iter()
        .any(|prefix| path == *prefix || path.strip_prefix(*prefix).is_some_and(|rest| rest.starts_with('/')))
}

fn asset_response(file: EmbeddedFile) -> Response {
    ([(header::CONTENT_TYPE, file.metadata.mimetype())], file.data).into_response()
}

fn not_found() -> Response {
    (StatusCode::NOT_FOUND, NOT_FOUND_BODY).into_response()
}

fn frontend_not_found() -> Response {
    let file = FrontendAssets::get(NOT_FOUND_FILE)
        .expect("embedded frontend 404 asset is missing; run `pnpm build:frontend:embedded` before starting or packaging the backend");

    (StatusCode::NOT_FOUND, [(header::CONTENT_TYPE, file.metadata.mimetype())], file.data).into_response()
}

#[cfg(test)]
mod tests {
    use super::{INDEX_FILE, has_file_extension, is_reserved_backend_path, normalized_asset_path, should_serve_frontend_not_found};

    #[test]
    fn normalizes_static_paths() {
        assert_eq!(normalized_asset_path("/").as_deref(), Some(INDEX_FILE));
        assert_eq!(normalized_asset_path("/dashboard/").as_deref(), Some("dashboard"));
        assert_eq!(normalized_asset_path("/_next/static/app.js").as_deref(), Some("_next/static/app.js"));
    }

    #[test]
    fn rejects_path_traversal() {
        assert_eq!(normalized_asset_path("/../secret"), None);
        assert_eq!(normalized_asset_path("/dashboard/../secret"), None);
        assert_eq!(normalized_asset_path("\\secret"), None);
    }

    #[test]
    fn reserves_backend_prefixes() {
        assert!(is_reserved_backend_path("/api"));
        assert!(is_reserved_backend_path("/api/users"));
        assert!(is_reserved_backend_path("/v1/chat/completions"));
        assert!(is_reserved_backend_path("/v1beta/models/gemini:generateContent"));
        assert!(!is_reserved_backend_path("/v10"));
        assert!(!is_reserved_backend_path("/dashboard"));
    }

    #[test]
    fn detects_file_extensions_in_last_segment() {
        assert!(has_file_extension("_next/static/app.js"));
        assert!(!has_file_extension("dashboard/admin"));
        assert!(!has_file_extension("release.v1/dashboard"));
    }

    #[test]
    fn serves_frontend_404_for_missing_page_routes_only() {
        assert!(should_serve_frontend_not_found("dashboard/unknown"));
        assert!(should_serve_frontend_not_found("release.v1/dashboard"));
        assert!(!should_serve_frontend_not_found("_next/static/missing.js"));
        assert!(!should_serve_frontend_not_found("favicon.ico"));
    }
}
