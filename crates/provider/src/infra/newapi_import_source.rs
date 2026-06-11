use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use req::ReqwestClient;
use types::provider::{NewApiQuickImportConfig, ProviderQuickImportSourceConfig, ProviderQuickImportSourceKind};

use crate::application::{
    ProviderError, ProviderResult, UpstreamImportData, UpstreamImportModel, UpstreamImportToken, UpstreamProviderImportSource, UpstreamSyncSnapshot,
    UpstreamSyncToken,
};
use crate::infra::newapi_import_types::{
    GroupMap, GroupsEnvelope, ModelsEnvelope, NewApiTokenRecord, TokenKeyEnvelope, TokenListEnvelope, client_error, decode_envelope, model_response,
    newapi_url, normalize_newapi_key, response_text, token_group_ratio, url_error,
};

const FETCH_TIMEOUT_SECONDS: u64 = 30;
const TOKEN_PAGE_SIZE: u64 = 100;

#[derive(Clone)]
pub struct NewApiImportSource {
    http: ReqwestClient,
    upstream_lock: Arc<tokio::sync::Mutex<()>>,
}

impl NewApiImportSource {
    pub fn new() -> ProviderResult<Self> {
        let http = ReqwestClient::from_builder(req::builder().timeout(Duration::from_secs(FETCH_TIMEOUT_SECONDS))).map_err(client_error)?;
        Ok(Self {
            http,
            upstream_lock: Arc::new(tokio::sync::Mutex::new(())),
        })
    }
}

#[async_trait]
impl UpstreamProviderImportSource for NewApiImportSource {
    async fn fetch_import_data(&self, source: &ProviderQuickImportSourceConfig) -> ProviderResult<UpstreamImportData> {
        let _guard = self.upstream_lock.lock().await;
        let ProviderQuickImportSourceConfig::Newapi(config) = source;
        let groups = self.fetch_groups(config).await?;
        let tokens = self.fetch_tokens(config, &groups).await?;
        Ok(UpstreamImportData {
            source_kind: ProviderQuickImportSourceKind::Newapi,
            tokens,
        })
    }

    async fn fetch_sync_snapshot(&self, source: &ProviderQuickImportSourceConfig) -> ProviderResult<UpstreamSyncSnapshot> {
        let _guard = self.upstream_lock.lock().await;
        let ProviderQuickImportSourceConfig::Newapi(config) = source;
        let groups = self.fetch_groups(config).await?;
        let records = self.fetch_token_records(config).await?;
        Ok(UpstreamSyncSnapshot {
            source_kind: ProviderQuickImportSourceKind::Newapi,
            groups: groups.into_iter().map(|(name, group)| (name, group.ratio())).collect(),
            tokens: records
                .into_iter()
                .map(|record| UpstreamSyncToken {
                    id: record.id.to_string(),
                    name: record.name,
                    masked_key: record.key,
                    status: record.status,
                    group: record.group,
                })
                .collect(),
        })
    }

    async fn fetch_sync_token_models(&self, source: &ProviderQuickImportSourceConfig, upstream_token_id: &str) -> ProviderResult<Vec<UpstreamImportModel>> {
        let _guard = self.upstream_lock.lock().await;
        let ProviderQuickImportSourceConfig::Newapi(config) = source;
        let token_id = upstream_token_id
            .parse::<i64>()
            .map_err(|error| ProviderError::InvalidInput(format!("invalid newapi token id: {error}")))?;
        let key = self.fetch_token_key(config, token_id).await?;
        self.fetch_models(config, &key).await
    }
}

impl NewApiImportSource {
    async fn fetch_tokens(&self, config: &NewApiQuickImportConfig, groups: &GroupMap) -> ProviderResult<Vec<UpstreamImportToken>> {
        let records = self.fetch_token_records(config).await?;
        let mut tokens = Vec::with_capacity(records.len());
        for record in records {
            tokens.push(self.enrich_token(config, groups, record).await?);
        }
        Ok(tokens)
    }

    async fn fetch_token_records(&self, config: &NewApiQuickImportConfig) -> ProviderResult<Vec<NewApiTokenRecord>> {
        let mut page = 1;
        let first = self.fetch_token_page(config, page).await?;
        let total = first.data.total;
        let mut items = first.data.items;
        while (items.len() as u64) < total {
            page += 1;
            let next = self.fetch_token_page(config, page).await?;
            if next.data.items.is_empty() {
                return Err(ProviderError::Infrastructure("newapi token list ended before reported total".into()));
            }
            items.extend(next.data.items);
        }
        Ok(items)
    }

    async fn enrich_token(&self, config: &NewApiQuickImportConfig, groups: &GroupMap, record: NewApiTokenRecord) -> ProviderResult<UpstreamImportToken> {
        let group_ratio = token_group_ratio(groups, record.group.as_deref())?;
        let (api_key, models) = if record.status == 1 {
            let key = self.fetch_token_key(config, record.id).await?;
            let models = self.fetch_models(config, &key).await?;
            (Some(key), models)
        } else {
            (None, Vec::new())
        };
        Ok(UpstreamImportToken {
            id: record.id.to_string(),
            name: record.name,
            masked_key: record.key,
            status: record.status,
            group: record.group,
            group_ratio,
            api_key,
            models,
        })
    }

    async fn fetch_token_page(&self, config: &NewApiQuickImportConfig, page: u64) -> ProviderResult<TokenListEnvelope> {
        let url = newapi_url(&config.base_url, &format!("/api/token/?p={page}&size={TOKEN_PAGE_SIZE}"))?;
        let text = self.get_text(config, &url).await?;
        decode_envelope(&text)
    }

    async fn fetch_groups(&self, config: &NewApiQuickImportConfig) -> ProviderResult<GroupMap> {
        let url = newapi_url(&config.base_url, "/api/user/self/groups")?;
        let text = self.get_text(config, &url).await?;
        let envelope: GroupsEnvelope = decode_envelope(&text)?;
        Ok(envelope.data)
    }

    async fn fetch_token_key(&self, config: &NewApiQuickImportConfig, token_id: i64) -> ProviderResult<String> {
        let url = newapi_url(&config.base_url, &format!("/api/token/{token_id}/key"))?;
        let text = self.post_text(config, url).await?;
        let envelope: TokenKeyEnvelope = decode_envelope(&text)?;
        Ok(normalize_newapi_key(&envelope.data.key))
    }

    async fn fetch_models(&self, config: &NewApiQuickImportConfig, api_key: &str) -> ProviderResult<Vec<UpstreamImportModel>> {
        let url = newapi_url(&config.base_url, "/v1/models")?;
        let request = self.http.get(req::Url::parse(&url).map_err(url_error)?).bearer_auth(api_key);
        let response = self
            .http
            .execute(self.http.build_request(request).map_err(client_error)?)
            .await
            .map_err(client_error)?;
        let text = response_text(response).await?;
        let envelope: ModelsEnvelope = decode_envelope(&text)?;
        Ok(envelope.data.into_iter().map(model_response).collect())
    }

    async fn get_text(&self, config: &NewApiQuickImportConfig, url: &str) -> ProviderResult<String> {
        let request = self.apply_admin_headers(self.http.get(req::Url::parse(url).map_err(url_error)?), config);
        let response = self
            .http
            .execute(self.http.build_request(request).map_err(client_error)?)
            .await
            .map_err(client_error)?;
        response_text(response).await
    }

    async fn post_text(&self, config: &NewApiQuickImportConfig, url: String) -> ProviderResult<String> {
        let request = self.apply_admin_headers(self.http.post(url), config);
        let response = self
            .http
            .execute(self.http.build_request(request).map_err(client_error)?)
            .await
            .map_err(client_error)?;
        response_text(response).await
    }

    fn apply_admin_headers(&self, request: req::RequestBuilder, config: &NewApiQuickImportConfig) -> req::RequestBuilder {
        request
            .bearer_auth(config.system_access_token.trim())
            .header("New-Api-User", config.user_id.trim())
    }
}
