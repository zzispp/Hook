use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use req::ReqwestClient;
use serde_json::json;
use time::{Duration as TimeDuration, OffsetDateTime, format_description::well_known::Rfc3339};
use types::provider::{
    ProviderQuickImportSourceConfig, ProviderQuickImportSourceKind, Sub2ApiPasswordQuickImportConfig, Sub2ApiQuickImportConfig, Sub2ApiTokenQuickImportConfig,
};

use crate::application::{
    ProviderError, ProviderResult, UpstreamGroupRatio, UpstreamImportData, UpstreamImportModel, UpstreamImportToken, UpstreamProviderImportSource,
    UpstreamSyncSnapshot,
};
use crate::infra::sub2api_import_types::{
    PaginatedKeys, Sub2ApiGroupRecord, Sub2ApiKeyRecord, TokenRefreshResponse, UserGroupRates, client_error, decode_envelope, decode_models, group_ratio,
    key_is_active, key_status, masked_key, response_text, sub2api_url, sync_token, url_error,
};
use crate::infra::sub2api_token_filter::skip_inactive_group_tokens;

const FETCH_TIMEOUT_SECONDS: u64 = 30;
const TOKEN_PAGE_SIZE: u64 = 100;
const REFRESH_THRESHOLD_MINUTES: i64 = 30;

#[derive(Clone)]
pub struct Sub2ApiImportSource {
    http: ReqwestClient,
    upstream_lock: Arc<tokio::sync::Mutex<()>>,
}

#[derive(Clone)]
struct ActiveToken {
    access_token: String,
    refresh_token: String,
    token_expires_at: String,
}

impl Sub2ApiImportSource {
    pub fn new() -> ProviderResult<Self> {
        let http = ReqwestClient::from_builder(req::builder().timeout(Duration::from_secs(FETCH_TIMEOUT_SECONDS))).map_err(client_error)?;
        Ok(Self {
            http,
            upstream_lock: Arc::new(tokio::sync::Mutex::new(())),
        })
    }
}

#[async_trait]
impl UpstreamProviderImportSource for Sub2ApiImportSource {
    async fn fetch_import_data(&self, source: &ProviderQuickImportSourceConfig) -> ProviderResult<UpstreamImportData> {
        let _guard = self.upstream_lock.lock().await;
        let ProviderQuickImportSourceConfig::Sub2api(config) = source else {
            return Err(ProviderError::InvalidInput("sub2api importer received non-sub2api source".into()));
        };
        let token = self.token_for_config(config).await?;
        let user_group_rates = self.fetch_user_group_rates(config, &token.access_token).await?;
        let records = skip_inactive_group_tokens(self.fetch_keys(config, &token.access_token).await?);
        let mut tokens = Vec::with_capacity(records.len());
        for record in records {
            tokens.push(self.enrich_token(config, &user_group_rates, record).await?);
        }
        Ok(UpstreamImportData {
            source_kind: ProviderQuickImportSourceKind::Sub2api,
            tokens,
        })
    }

    async fn fetch_sync_snapshot(&self, source: &ProviderQuickImportSourceConfig) -> ProviderResult<UpstreamSyncSnapshot> {
        let _guard = self.upstream_lock.lock().await;
        let ProviderQuickImportSourceConfig::Sub2api(config) = source else {
            return Err(ProviderError::InvalidInput("sub2api importer received non-sub2api source".into()));
        };
        let token = self.token_for_config(config).await?;
        let groups = self.fetch_available_groups(config, &token.access_token).await?;
        let user_group_rates = self.fetch_user_group_rates(config, &token.access_token).await?;
        let tokens = self
            .fetch_keys(config, &token.access_token)
            .await?
            .into_iter()
            .map(sync_token)
            .collect::<ProviderResult<Vec<_>>>()?;
        let mut group_ratios = std::collections::BTreeMap::new();
        for group in groups {
            group_ratios.insert(group.name.clone(), UpstreamGroupRatio::Fixed(group_decimal(&group, &user_group_rates)?));
        }
        Ok(UpstreamSyncSnapshot {
            source_kind: ProviderQuickImportSourceKind::Sub2api,
            groups: group_ratios,
            tokens,
        })
    }

    async fn fetch_sync_token_models(&self, source: &ProviderQuickImportSourceConfig, upstream_token_id: &str) -> ProviderResult<Vec<UpstreamImportModel>> {
        let _guard = self.upstream_lock.lock().await;
        let ProviderQuickImportSourceConfig::Sub2api(config) = source else {
            return Err(ProviderError::InvalidInput("sub2api importer received non-sub2api source".into()));
        };
        let token = self.token_for_config(config).await?;
        let token_id = upstream_token_id
            .parse::<i64>()
            .map_err(|error| ProviderError::InvalidInput(format!("invalid sub2api token id: {error}")))?;
        let record = self
            .fetch_keys(config, &token.access_token)
            .await?
            .into_iter()
            .find(|item| item.id == token_id)
            .ok_or_else(|| ProviderError::InvalidInput(format!("sub2api token does not exist: {upstream_token_id}")))?;
        self.fetch_models(config, &record.key).await
    }

    async fn refreshed_source_config(&self, source: &ProviderQuickImportSourceConfig) -> ProviderResult<Option<ProviderQuickImportSourceConfig>> {
        let ProviderQuickImportSourceConfig::Sub2api(config) = source else {
            return Ok(Some(source.clone()));
        };
        match config {
            Sub2ApiQuickImportConfig::Password(_) => Ok(Some(source.clone())),
            Sub2ApiQuickImportConfig::Token(config) => {
                let token = self.active_token(config, REFRESH_THRESHOLD_MINUTES).await?;
                Ok(Some(ProviderQuickImportSourceConfig::Sub2api(Sub2ApiQuickImportConfig::Token(
                    Sub2ApiTokenQuickImportConfig {
                        base_url: config.base_url.clone(),
                        auth_token: token.access_token,
                        refresh_token: token.refresh_token,
                        token_expires_at: token.token_expires_at,
                    },
                ))))
            }
        }
    }

    async fn refreshed_source_config_with_threshold(
        &self,
        source: &ProviderQuickImportSourceConfig,
        refresh_threshold_minutes: i64,
    ) -> ProviderResult<Option<ProviderQuickImportSourceConfig>> {
        let ProviderQuickImportSourceConfig::Sub2api(config) = source else {
            return Ok(Some(source.clone()));
        };
        match config {
            Sub2ApiQuickImportConfig::Password(_) => Ok(Some(source.clone())),
            Sub2ApiQuickImportConfig::Token(config) => {
                let token = self.active_token(config, refresh_threshold_minutes).await?;
                Ok(Some(ProviderQuickImportSourceConfig::Sub2api(Sub2ApiQuickImportConfig::Token(
                    Sub2ApiTokenQuickImportConfig {
                        base_url: config.base_url.clone(),
                        auth_token: token.access_token,
                        refresh_token: token.refresh_token,
                        token_expires_at: token.token_expires_at,
                    },
                ))))
            }
        }
    }
}

impl Sub2ApiImportSource {
    async fn active_token(&self, config: &Sub2ApiTokenQuickImportConfig, refresh_threshold_minutes: i64) -> ProviderResult<ActiveToken> {
        let expires_at = parse_token_expires_at(&config.token_expires_at)?;
        if expires_at - OffsetDateTime::now_utc() > TimeDuration::minutes(refresh_threshold_minutes) {
            return Ok(ActiveToken {
                access_token: config.auth_token.trim().to_owned(),
                refresh_token: config.refresh_token.trim().to_owned(),
                token_expires_at: config.token_expires_at.trim().to_owned(),
            });
        }
        let refreshed = self.refresh_token(config).await?;
        Ok(ActiveToken {
            access_token: refreshed.access_token,
            refresh_token: refreshed.refresh_token,
            token_expires_at: refreshed.token_expires_at,
        })
    }

    async fn refresh_token(&self, config: &Sub2ApiTokenQuickImportConfig) -> ProviderResult<TokenRefreshResponse> {
        let url = sub2api_url(&config.base_url, "/api/v1/auth/refresh")?;
        let request = self.http.post(url).json(&json!({
            "refresh_token": config.refresh_token.trim(),
        }));
        let response = self.execute(request).await?;
        let text = response_text(response).await?;
        decode_envelope(&text)
    }

    async fn password_login(&self, config: &Sub2ApiPasswordQuickImportConfig) -> ProviderResult<ActiveToken> {
        let url = sub2api_url(&config.base_url, "/api/v1/auth/login")?;
        let request = self.http.post(url).json(&json!({
            "email": config.email.trim(),
            "password": config.password.trim(),
        }));
        let response = self.execute(request).await?;
        let text = response_text(response).await?;
        let login: TokenRefreshResponse = decode_envelope(&text)?;
        Ok(ActiveToken {
            access_token: login.access_token,
            refresh_token: login.refresh_token,
            token_expires_at: login.token_expires_at,
        })
    }

    async fn token_for_config(&self, config: &Sub2ApiQuickImportConfig) -> ProviderResult<ActiveToken> {
        match config {
            Sub2ApiQuickImportConfig::Password(config) => self.password_login(config).await,
            Sub2ApiQuickImportConfig::Token(config) => self.active_token(config, REFRESH_THRESHOLD_MINUTES).await,
        }
    }

    async fn fetch_keys(&self, config: &Sub2ApiQuickImportConfig, access_token: &str) -> ProviderResult<Vec<Sub2ApiKeyRecord>> {
        let mut page = 1_u64;
        let first = self.fetch_key_page(config, access_token, page).await?;
        let total = first.total;
        let mut items = first.items;
        while (items.len() as u64) < total {
            page += 1;
            let next = self.fetch_key_page(config, access_token, page).await?;
            if next.items.is_empty() {
                return Err(ProviderError::Infrastructure("sub2api key list ended before reported total".into()));
            }
            items.extend(next.items);
        }
        Ok(items)
    }

    async fn fetch_key_page(&self, config: &Sub2ApiQuickImportConfig, access_token: &str, page: u64) -> ProviderResult<PaginatedKeys> {
        let url = sub2api_url(base_url(config), &format!("/api/v1/keys?page={page}&page_size={TOKEN_PAGE_SIZE}"))?;
        let text = self.get_text(access_token, &url).await?;
        decode_envelope(&text)
    }

    async fn fetch_available_groups(&self, config: &Sub2ApiQuickImportConfig, access_token: &str) -> ProviderResult<Vec<Sub2ApiGroupRecord>> {
        let url = sub2api_url(base_url(config), "/api/v1/groups/available")?;
        let text = self.get_text(access_token, &url).await?;
        decode_envelope(&text)
    }

    async fn fetch_user_group_rates(&self, config: &Sub2ApiQuickImportConfig, access_token: &str) -> ProviderResult<UserGroupRates> {
        let url = sub2api_url(base_url(config), "/api/v1/groups/rates")?;
        let text = self.get_text(access_token, &url).await?;
        decode_envelope(&text)
    }

    async fn enrich_token(
        &self,
        config: &Sub2ApiQuickImportConfig,
        user_group_rates: &UserGroupRates,
        record: Sub2ApiKeyRecord,
    ) -> ProviderResult<UpstreamImportToken> {
        let group = record.group.as_ref().map(|group| group.name.clone());
        let group_ratio = group_ratio(&record, user_group_rates)?.unwrap_or(rust_decimal::Decimal::ONE);
        let status = key_status(&record)?;
        let is_active = key_is_active(&record)?;
        let models = if is_active && group.is_some() {
            self.fetch_models(config, &record.key).await?
        } else {
            Vec::new()
        };
        Ok(UpstreamImportToken {
            id: record.id.to_string(),
            name: record.name,
            masked_key: masked_key(&record.key),
            status,
            is_active,
            group,
            group_ratio,
            api_key: is_active.then_some(record.key),
            models,
        })
    }

    async fn fetch_models(&self, config: &Sub2ApiQuickImportConfig, api_key: &str) -> ProviderResult<Vec<UpstreamImportModel>> {
        let url = sub2api_url(base_url(config), "/v1/models")?;
        let request = self.http.get(req::Url::parse(&url).map_err(url_error)?).bearer_auth(api_key.trim());
        let response = self.execute(request).await?;
        let text = response_text(response).await?;
        decode_models(&text)
    }

    async fn get_text(&self, access_token: &str, url: &str) -> ProviderResult<String> {
        let request = self.http.get(req::Url::parse(url).map_err(url_error)?).bearer_auth(access_token.trim());
        let response = self.execute(request).await?;
        response_text(response).await
    }

    async fn execute(&self, request: req::RequestBuilder) -> ProviderResult<req::Response> {
        let request = self.http.build_request(request).map_err(client_error)?;
        self.http.execute(request).await.map_err(client_error)
    }
}

fn base_url(config: &Sub2ApiQuickImportConfig) -> &str {
    match config {
        Sub2ApiQuickImportConfig::Password(config) => &config.base_url,
        Sub2ApiQuickImportConfig::Token(config) => &config.base_url,
    }
}

fn parse_token_expires_at(value: &str) -> ProviderResult<OffsetDateTime> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(ProviderError::InvalidInput("token_expires_at cannot be blank".into()));
    }
    if let Ok(milliseconds) = trimmed.parse::<i128>() {
        let seconds = milliseconds.div_euclid(1000) as i64;
        let nanos = (milliseconds.rem_euclid(1000) as i64) * 1_000_000;
        return OffsetDateTime::from_unix_timestamp(seconds)
            .map(|value| value + TimeDuration::nanoseconds(nanos))
            .map_err(|error| ProviderError::InvalidInput(format!("invalid token_expires_at milliseconds: {error}")));
    }
    OffsetDateTime::parse(trimmed, &Rfc3339).map_err(|error| ProviderError::InvalidInput(format!("invalid token_expires_at: {error}")))
}

fn group_decimal(group: &Sub2ApiGroupRecord, user_group_rates: &UserGroupRates) -> ProviderResult<rust_decimal::Decimal> {
    let ratio = user_group_rates.get(&group.id.to_string()).copied().unwrap_or(group.rate_multiplier);
    rust_decimal::Decimal::from_str_exact(&ratio.to_string())
        .map_err(|error| ProviderError::Infrastructure(format!("invalid sub2api group ratio for {}: {error}", group.name)))
}
