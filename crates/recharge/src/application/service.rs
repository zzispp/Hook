use async_trait::async_trait;
use rust_decimal::Decimal;
use types::{
    pagination::PageRequest,
    recharge::{
        PaymentChannelUpdatePayload, RECHARGE_PACKAGE_STATUS_ACTIVE, RechargeOrder, RechargeOrderCreatePayload, RechargeOrderListFilters,
        RechargeOrderListResponse, RechargePackage, RechargePackageCreatePayload, RechargePackageListFilters, RechargePackageListResponse,
        RechargePackageUpdatePayload, UserRechargePackageListResponse, UserRechargePackageResponse,
    },
    system_setting::SystemSettings,
};

use crate::application::{
    PaymentChannelRegistry, RechargeError, RechargeOrderCreateRecord, RechargeRepository, RechargeResult, RechargeUseCase,
    validation::{sanitize_create, sanitize_update, validate_create, validate_order_filters, validate_package_filters, validate_page, validate_update},
};

pub struct RechargeService<R> {
    repository: R,
}

impl<R> RechargeService<R>
where
    R: RechargeRepository,
{
    pub async fn new(repository: R, registry: PaymentChannelRegistry) -> RechargeResult<Self> {
        repository.sync_payment_channels(registry.registrations()).await?;
        Ok(Self { repository })
    }
}

#[async_trait]
impl<R> RechargeUseCase for RechargeService<R>
where
    R: RechargeRepository,
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

    async fn create_user_order(&self, user_id: &str, input: RechargeOrderCreatePayload) -> RechargeResult<RechargeOrder> {
        validate_user_id(user_id)?;
        let package_id = validated_package_id(input.package_id)?;
        let settings = self.repository.get_system_settings().await?;
        let package = self.repository.find_package(&package_id).await?.ok_or(RechargeError::NotFound)?;
        validate_order_package(&package, &settings)?;
        self.repository.create_order(order_record(user_id, &package, &settings)).await
    }

    async fn list_payment_channels(&self) -> RechargeResult<Vec<types::recharge::PaymentChannel>> {
        self.repository.list_payment_channels().await
    }

    async fn update_payment_channel(&self, code: &str, input: PaymentChannelUpdatePayload) -> RechargeResult<types::recharge::PaymentChannel> {
        self.repository.update_payment_channel(code, input).await
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

fn validated_package_id(value: String) -> RechargeResult<String> {
    let package_id = value.trim().to_owned();
    if package_id.is_empty() {
        return Err(RechargeError::InvalidInput("package_id is required".into()));
    }
    Ok(package_id)
}

fn validate_order_package(package: &RechargePackage, settings: &SystemSettings) -> RechargeResult<()> {
    if !settings.recharge_enabled {
        return Err(RechargeError::InvalidInput("recharge is disabled".into()));
    }
    if package.status != RECHARGE_PACKAGE_STATUS_ACTIVE {
        return Err(RechargeError::InvalidInput("recharge package is disabled".into()));
    }
    validate_amount_limits(package.recharge_amount, settings)
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

fn order_record(user_id: &str, package: &RechargePackage, settings: &SystemSettings) -> RechargeOrderCreateRecord {
    let now = time::OffsetDateTime::now_utc();
    RechargeOrderCreateRecord {
        user_id: user_id.to_owned(),
        package_id: Some(package.id.clone()),
        package_name: package.name.clone(),
        recharge_amount: package.recharge_amount,
        gift_amount: package.gift_amount,
        total_arrival_amount: package.recharge_amount + package.gift_amount,
        payable_amount: package.recharge_amount * settings.recharge_arrival_ratio,
        expires_at: now + time::Duration::minutes(settings.recharge_order_expire_minutes),
    }
}

#[cfg(test)]
#[path = "service_tests.rs"]
mod tests;
