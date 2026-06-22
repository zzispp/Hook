use std::sync::Arc;

use scheduler::runtime::{ScheduleTaskContext, ScheduledTaskLifecycle, SchedulerError, SchedulerResult, TaskConfigValue, TaskResult};
use storage::scheduler::task_definition;

use super::{integer_config, integer_fields, validate_empty_config, validate_positive_integer};

const RECHARGE_ORDER_EXPIRE_INTERVAL_SECONDS: i64 = 60;
const RECHARGE_PAYMENT_POLL_INTERVAL_SECONDS: i64 = 60;
const RECHARGE_PAYMENT_POLL_LIMIT: i64 = 50;

#[derive(Clone)]
pub(super) struct RechargeOrderExpireTask {
    pub(super) recharge_service: Arc<dyn ::recharge::application::RechargeUseCase>,
}

#[derive(Clone)]
pub(super) struct RechargePaymentPollTask {
    pub(super) recharge_service: Arc<dyn ::recharge::application::RechargeUseCase>,
}

#[async_trait::async_trait]
impl ScheduledTaskLifecycle for RechargeOrderExpireTask {
    fn definition(&self) -> types::scheduler::ScheduledTaskDefinition {
        task_definition(
            "recharge_order_expire",
            "scheduledTasks.definitions.rechargeOrderExpire.name",
            "scheduledTasks.definitions.rechargeOrderExpire.description",
            RECHARGE_ORDER_EXPIRE_INTERVAL_SECONDS,
            RECHARGE_ORDER_EXPIRE_INTERVAL_SECONDS,
            serde_json::json!({}),
            Vec::new(),
        )
    }

    fn validate_config(&self, config: &TaskConfigValue) -> SchedulerResult<()> {
        validate_empty_config(config)
    }

    async fn run(&self, _ctx: ScheduleTaskContext, _config: TaskConfigValue) -> TaskResult {
        let expired = self.recharge_service.expire_pending_orders().await.map_err(recharge_error)?;
        Ok(Some(format!("expired={expired}")))
    }
}

#[async_trait::async_trait]
impl ScheduledTaskLifecycle for RechargePaymentPollTask {
    fn definition(&self) -> types::scheduler::ScheduledTaskDefinition {
        task_definition(
            "recharge_payment_poll",
            "scheduledTasks.definitions.rechargePaymentPoll.name",
            "scheduledTasks.definitions.rechargePaymentPoll.description",
            RECHARGE_PAYMENT_POLL_INTERVAL_SECONDS,
            RECHARGE_PAYMENT_POLL_INTERVAL_SECONDS,
            serde_json::json!({
                "limit": RECHARGE_PAYMENT_POLL_LIMIT
            }),
            integer_fields(&[("limit", "scheduledTasks.config.rechargePaymentPoll.limit", 1)]),
        )
    }

    fn validate_config(&self, config: &TaskConfigValue) -> SchedulerResult<()> {
        validate_positive_integer(config, "limit", 1)
    }

    async fn run(&self, _ctx: ScheduleTaskContext, config: TaskConfigValue) -> TaskResult {
        let limit = integer_config(&config, "limit")?;
        let limit = u64::try_from(limit).map_err(|_| SchedulerError::InvalidInput("limit must be greater than 0".into()))?;
        let result = self.recharge_service.poll_pending_payment_orders(limit).await.map_err(recharge_error)?;
        Ok(Some(format!(
            "checked={}, paid={}, unsupported={}",
            result.checked, result.paid, result.unsupported
        )))
    }
}

fn recharge_error(error: ::recharge::application::RechargeError) -> SchedulerError {
    SchedulerError::Infrastructure(error.to_string())
}
