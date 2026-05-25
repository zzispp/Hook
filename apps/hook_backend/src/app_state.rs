use std::sync::Arc;

use rbac::{
    application::{AuthorizationConfig, RbacService},
    infra::{RedisRbacCache, StorageRbacRepository},
};
use types::api_token::ApiTokenOwnerResponse;

use crate::{llm_proxy::LlmProxyState, performance_monitoring_os::PerformanceOsCollector};

pub(crate) struct AppState {
    pub(crate) database: storage::Database,
    pub(crate) users: Arc<dyn user::application::UserUseCase>,
    pub(crate) tokens: user::api::TokenService,
    pub(crate) rbac: Arc<RbacService<StorageRbacRepository, RedisRbacCache>>,
    pub(crate) models: Arc<dyn model::application::ModelUseCase>,
    pub(crate) providers: Arc<dyn provider::application::ProviderUseCase>,
    pub(crate) dashboard: Arc<dyn dashboard::application::DashboardUseCase>,
    pub(crate) wallets: Arc<dyn wallet::application::WalletUseCase>,
    pub(crate) card_codes: Arc<dyn card_code::application::CardCodeUseCase>,
    pub(crate) recharges: Arc<dyn recharge::application::RechargeUseCase>,
    pub(crate) system_settings: Arc<dyn setting::application::SettingUseCase>,
    pub(crate) groups: Arc<dyn group::application::GroupUseCase>,
    pub(crate) i18n: Arc<dyn i18n::application::I18nUseCase>,
    pub(crate) api_tokens: Arc<dyn api_token::application::ApiTokenUseCase>,
    pub(crate) cache_monitoring_system_owner: Option<(String, ApiTokenOwnerResponse)>,
    pub(crate) operations: Arc<dyn operations::application::OperationsUseCase>,
    pub(crate) captcha: Arc<dyn captcha::application::CaptchaUseCase>,
    pub(crate) llm_proxy: LlmProxyState,
    pub(crate) performance_os_collector: Arc<PerformanceOsCollector>,
    pub(crate) scheduler: Arc<dyn scheduler::runtime::SchedulerUseCase>,
    pub(crate) authorization: AuthorizationConfig,
}
