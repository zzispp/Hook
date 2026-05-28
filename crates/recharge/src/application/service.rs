use async_trait::async_trait;
use payment::{PaymentChannelConfig, PaymentChannelRegistry};
use rust_decimal::Decimal;
use types::{
    pagination::PageRequest,
    recharge::{
        PaymentCallbackListFilters, PaymentCallbackRecordListResponse, PaymentChannelUpdatePayload, PublicPaymentChannelResponse,
        RECHARGE_PACKAGE_STATUS_ACTIVE, RechargeOrderCreatePayload, RechargeOrderCreateResponse, RechargeOrderListFilters, RechargeOrderListResponse,
        RechargePackage, RechargePackageCreatePayload, RechargePackageListFilters, RechargePackageListResponse, RechargePackageUpdatePayload,
        UserRechargePackageListResponse, UserRechargePackageResponse,
    },
    system_setting::{SystemSettings, public_base_url_is_valid},
};

const CUSTOM_RECHARGE_PACKAGE_NAME: &str = "Custom recharge";

use crate::application::{
    NoRechargeSecretCipher, PaymentChannelUpdateRecord, RechargeError, RechargePaymentCallbackResult, RechargeRepository, RechargeResult, RechargeSecretCipher,
    RechargeUseCase,
    validation::{
        sanitize_create, sanitize_update, validate_callback_filters, validate_create, validate_order_filters, validate_package_filters, validate_page,
        validate_update,
    },
};

pub struct RechargeService<R, C = NoRechargeSecretCipher> {
    pub(super) repository: R,
    pub(super) registry: PaymentChannelRegistry,
    cipher: C,
}

impl<R> RechargeService<R, NoRechargeSecretCipher>
where
    R: RechargeRepository,
{
    pub async fn new(repository: R, registry: PaymentChannelRegistry) -> RechargeResult<Self> {
        Self::with_secret_cipher(repository, registry, NoRechargeSecretCipher).await
    }
}

impl<R, C> RechargeService<R, C>
where
    R: RechargeRepository,
    C: RechargeSecretCipher,
{
    pub async fn with_secret_cipher(repository: R, registry: PaymentChannelRegistry, cipher: C) -> RechargeResult<Self> {
        repository.sync_payment_channels(&registry.registrations()).await?;
        Ok(Self { repository, registry, cipher })
    }
}

#[async_trait]
impl<R, C> RechargeUseCase for RechargeService<R, C>
where
    R: RechargeRepository,
    C: RechargeSecretCipher,
{
    async fn list_packages(&self, page: PageRequest, filters: RechargePackageListFilters) -> RechargeResult<RechargePackageListResponse> {
        validate_page(page)?;
        validate_package_filters(&filters)?;
        self.repository.list_packages(page, filters).await.map(Into::into)
    }

    async fn list_user_packages(&self, page: PageRequest) -> RechargeResult<UserRechargePackageListResponse> {
        validate_page(page)?;
        let settings = self.repository.get_system_settings().await?;
        let packages = self.repository.list_active_packages(page).await?;
        Ok(user_package_list_response(packages, settings))
    }

    async fn create_package(&self, input: RechargePackageCreatePayload) -> RechargeResult<RechargePackage> {
        let input = sanitize_create(input);
        validate_create(&input)?;
        self.repository.create_package(input).await
    }

    async fn update_package(&self, id: &str, input: RechargePackageUpdatePayload) -> RechargeResult<RechargePackage> {
        let input = sanitize_update(input);
        validate_update(&input)?;
        self.repository.update_package(id, input).await
    }

    async fn list_orders(&self, page: PageRequest, filters: RechargeOrderListFilters) -> RechargeResult<RechargeOrderListResponse> {
        validate_page(page)?;
        validate_order_filters(&filters)?;
        self.repository.list_orders(page, filters).await.map(Into::into)
    }

    async fn list_user_orders(&self, user_id: &str, page: PageRequest) -> RechargeResult<RechargeOrderListResponse> {
        validate_user_id(user_id)?;
        validate_page(page)?;
        self.repository.list_user_orders(user_id, page).await.map(Into::into)
    }

    async fn list_payment_callbacks(&self, page: PageRequest, filters: PaymentCallbackListFilters) -> RechargeResult<PaymentCallbackRecordListResponse> {
        validate_page(page)?;
        validate_callback_filters(&filters)?;
        self.repository.list_payment_callbacks(page, filters).await.map(Into::into)
    }

    async fn create_user_order(&self, user_id: &str, input: RechargeOrderCreatePayload) -> RechargeResult<RechargeOrderCreateResponse> {
        validate_user_id(user_id)?;
        let order_item = self.order_item(input.package_id, input.recharge_amount).await?;
        let payment_channel_code = validated_required(input.payment_channel_code, "payment_channel_code")?;
        let payment_method = validated_required(input.payment_method, "payment_method")?;
        let settings = self.repository.get_system_settings().await?;
        let channel = self.payment_channel(&payment_channel_code).await?;
        validate_order_item(&order_item, &settings)?;
        validate_payment_settings(&settings)?;
        self.create_payment_order(user_id, &payment_channel_code, &payment_method, &order_item, &settings, channel)
            .await
    }

    async fn list_payment_channels(&self) -> RechargeResult<Vec<types::recharge::PaymentChannel>> {
        let mut channels = self.repository.list_payment_channels().await?;
        for channel in &mut channels {
            channel.config_schema = self.provider_schema(&channel.code);
        }
        Ok(channels)
    }

    async fn list_user_payment_channels(&self) -> RechargeResult<Vec<PublicPaymentChannelResponse>> {
        let channels = self.repository.list_payment_channels().await?;
        channels
            .into_iter()
            .filter(|channel| channel.enabled)
            .map(|channel| self.public_payment_channel(channel))
            .collect()
    }

    async fn update_payment_channel(&self, code: &str, input: PaymentChannelUpdatePayload) -> RechargeResult<types::recharge::PaymentChannel> {
        if input.enabled {
            validate_payment_channel_public_base_url(&self.repository.get_system_settings().await?)?;
        }
        let encrypted_secret = input
            .api_key
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .map(|value| self.cipher.encrypt_secret(value.trim()))
            .transpose()?;
        let mut channel = self
            .repository
            .update_payment_channel(
                code,
                PaymentChannelUpdateRecord {
                    enabled: input.enabled,
                    config: input.config,
                    encrypted_secret,
                },
            )
            .await?;
        channel.config_schema = self.provider_schema(&channel.code);
        Ok(channel)
    }

    async fn handle_payment_callback(&self, request: crate::application::RechargePaymentCallbackRequest) -> RechargeResult<RechargePaymentCallbackResult> {
        self.handle_logged_provider_callback(request).await
    }

    async fn poll_pending_payment_orders(&self, limit: u64) -> RechargeResult<crate::application::RechargePaymentPollResult> {
        self.poll_pending_orders(limit).await
    }
}

impl<R, C> RechargeService<R, C>
where
    R: RechargeRepository,
    C: RechargeSecretCipher,
{
    async fn payment_channel(&self, code: &str) -> RechargeResult<super::payment_flow::PaymentChannelContext> {
        let channel = self.repository.find_payment_channel(code).await?.ok_or(RechargeError::NotFound)?;
        if !channel.enabled {
            return Err(RechargeError::InvalidInput("payment channel is disabled".into()));
        }
        Ok(super::payment_flow::PaymentChannelContext {
            code: channel.code.clone(),
            name: channel.name,
            config: self.payment_config(&channel.code).await?,
        })
    }

    async fn order_item(&self, package_id: Option<String>, recharge_amount: Option<Decimal>) -> RechargeResult<RechargeOrderItem> {
        match (normalized_optional(package_id), recharge_amount) {
            (Some(id), None) => self.package_order_item(&id).await,
            (None, Some(amount)) => Ok(custom_order_item(amount)),
            (Some(_), Some(_)) => Err(RechargeError::InvalidInput("package_id and recharge_amount cannot be used together".into())),
            (None, None) => Err(RechargeError::InvalidInput("package_id or recharge_amount is required".into())),
        }
    }

    async fn package_order_item(&self, package_id: &str) -> RechargeResult<RechargeOrderItem> {
        let package = self.repository.find_package(package_id).await?.ok_or(RechargeError::NotFound)?;
        Ok(RechargeOrderItem {
            package_id: Some(package.id),
            package_name: package.name,
            package_status: Some(package.status),
            recharge_amount: package.recharge_amount,
            gift_amount: package.gift_amount,
        })
    }

    pub(super) async fn payment_config(&self, code: &str) -> RechargeResult<payment::PaymentChannelConfig> {
        let stored = self.repository.payment_channel_config(code).await?;
        let secret = stored.encrypted_secret.as_deref().map(|value| self.cipher.decrypt_secret(value)).transpose()?;
        Ok(PaymentChannelConfig { config: stored.config, secret })
    }

    fn provider_schema(&self, code: &str) -> Option<serde_json::Value> {
        self.registry
            .provider(code)
            .and_then(|provider| serde_json::to_value(provider.config_schema()).ok())
    }

    fn public_payment_channel(&self, channel: types::recharge::PaymentChannel) -> RechargeResult<PublicPaymentChannelResponse> {
        let provider = self
            .registry
            .provider(&channel.code)
            .ok_or_else(|| RechargeError::Payment(format!("payment provider is not registered: {}", channel.code)))?;
        Ok(PublicPaymentChannelResponse {
            code: channel.code,
            name: channel.name,
            methods: provider.config_schema().methods,
        })
    }
}

fn user_package_list_response(page: types::pagination::Page<RechargePackage>, settings: SystemSettings) -> UserRechargePackageListResponse {
    UserRechargePackageListResponse {
        recharge_enabled: settings.recharge_enabled,
        arrival_ratio: settings.recharge_arrival_ratio,
        min_amount: settings.recharge_min_amount,
        max_amount: settings.recharge_max_amount,
        items: page
            .items
            .into_iter()
            .map(|package| user_package_response(package, settings.recharge_arrival_ratio))
            .collect(),
        total: page.total,
        page: page.page,
        page_size: page.page_size,
    }
}

fn user_package_response(package: RechargePackage, ratio: Decimal) -> UserRechargePackageResponse {
    let total_arrival_amount = package.recharge_amount + package.gift_amount;
    UserRechargePackageResponse {
        id: package.id,
        name: package.name,
        description: package.description,
        recharge_amount: package.recharge_amount,
        gift_amount: package.gift_amount,
        total_arrival_amount,
        estimated_payable_amount: package.recharge_amount * ratio,
        sort_order: package.sort_order,
        updated_at: package.updated_at,
    }
}

fn validate_user_id(value: &str) -> RechargeResult<()> {
    if value.trim().is_empty() {
        return Err(RechargeError::Forbidden);
    }
    Ok(())
}

fn normalized_optional(value: Option<String>) -> Option<String> {
    value.map(|item| item.trim().to_owned()).filter(|item| !item.is_empty())
}

fn validated_required(value: String, field: &str) -> RechargeResult<String> {
    let value = value.trim().to_owned();
    if value.is_empty() {
        return Err(RechargeError::InvalidInput(format!("{field} is required")));
    }
    Ok(value)
}

fn validate_order_item(item: &RechargeOrderItem, settings: &SystemSettings) -> RechargeResult<()> {
    if !settings.recharge_enabled {
        return Err(RechargeError::InvalidInput("recharge is disabled".into()));
    }
    if item.package_status.as_deref().is_some_and(|status| status != RECHARGE_PACKAGE_STATUS_ACTIVE) {
        return Err(RechargeError::InvalidInput("recharge package is disabled".into()));
    }
    validate_custom_amount(item.recharge_amount)?;
    validate_amount_limits(item.recharge_amount, settings)
}

fn validate_payment_settings(settings: &SystemSettings) -> RechargeResult<()> {
    if settings.recharge_max_unpaid_orders <= 0 {
        return Err(RechargeError::InvalidInput("recharge_max_unpaid_orders must be greater than 0".into()));
    }
    if settings.public_base_url.trim().is_empty() {
        return Err(RechargeError::InvalidInput("public_base_url is required for online recharge".into()));
    }
    validate_public_base_url_value(&settings.public_base_url)
}

fn validate_payment_channel_public_base_url(settings: &SystemSettings) -> RechargeResult<()> {
    if settings.public_base_url.trim().is_empty() {
        return Err(RechargeError::InvalidInput(
            "public_base_url is required before enabling payment channel".into(),
        ));
    }
    validate_public_base_url_value(&settings.public_base_url)
}

fn validate_public_base_url_value(value: &str) -> RechargeResult<()> {
    let is_valid =
        public_base_url_is_valid(value.trim()).map_err(|error| RechargeError::Infrastructure(format!("invalid public_base_url validation regex: {error}")))?;
    if !is_valid {
        return Err(RechargeError::InvalidInput("public_base_url must be a valid HTTP or HTTPS URL".into()));
    }
    Ok(())
}

fn validate_amount_limits(amount: Decimal, settings: &SystemSettings) -> RechargeResult<()> {
    if amount < settings.recharge_min_amount {
        return Err(RechargeError::InvalidInput("recharge amount is below minimum".into()));
    }
    if amount > settings.recharge_max_amount {
        return Err(RechargeError::InvalidInput("recharge amount exceeds maximum".into()));
    }
    Ok(())
}

fn validate_custom_amount(amount: Decimal) -> RechargeResult<()> {
    if amount <= Decimal::ZERO {
        return Err(RechargeError::InvalidInput("recharge_amount must be greater than 0".into()));
    }
    Ok(())
}

fn custom_order_item(recharge_amount: Decimal) -> RechargeOrderItem {
    RechargeOrderItem {
        package_id: None,
        package_name: CUSTOM_RECHARGE_PACKAGE_NAME.into(),
        package_status: None,
        recharge_amount,
        gift_amount: Decimal::ZERO,
    }
}

pub(super) struct RechargeOrderItem {
    pub package_id: Option<String>,
    pub package_name: String,
    pub package_status: Option<String>,
    pub recharge_amount: Decimal,
    pub gift_amount: Decimal,
}

#[cfg(test)]
#[path = "service_tests.rs"]
mod tests;
