use async_trait::async_trait;
use rand::{Rng, rngs::OsRng};
use rust_decimal::Decimal;
use types::{
    card_code::{
        CARD_CODE_BALANCE_TYPE_GIFT, CARD_CODE_BALANCE_TYPE_RECHARGE, CARD_CODE_STATUS_ACTIVE,
        CardCodeBatchStatusPayload, CardCodeBatchStatusResponse, CardCodeCreateRecord,
        CardCodeGeneratePayload, CardCodeGenerateResponse, CardCodeListFilters,
        CardCodeListResponse, CardCodeRedeemInput, CardCodeRedeemPayload, CardCodeRedeemResponse,
        CardCodeType, CardCodeTypeCreatePayload, CardCodeTypeListFilters,
        CardCodeTypeListResponse, CardCodeTypeUpdatePayload,
    },
    pagination::PageRequest,
};
use uuid::Uuid;

use crate::application::{
    CardCodeError, CardCodeOperator, CardCodeRedeemer, CardCodeRepository, CardCodeResult,
    CardCodeUseCase,
};

use super::validation::{
    validate_batch_status, validate_generate, validate_page, validate_type_create, validate_type_update,
};

const CODE_ALPHABET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
const MAX_CODE_ATTEMPTS: u8 = 20;

pub struct CardCodeService<R> {
    repository: R,
}

impl<R> CardCodeService<R>
where
    R: CardCodeRepository,
{
    pub const fn new(repository: R) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<R> CardCodeUseCase for CardCodeService<R>
where
    R: CardCodeRepository,
{
    async fn list_types(
        &self,
        page: PageRequest,
        filters: CardCodeTypeListFilters,
    ) -> CardCodeResult<CardCodeTypeListResponse> {
        validate_page(page)?;
        self.repository.list_types(page, filters).await.map(Into::into)
    }

    async fn create_type(&self, input: CardCodeTypeCreatePayload) -> CardCodeResult<CardCodeType> {
        let input = sanitize_type_create(input);
        validate_type_create(&input)?;
        self.repository.create_type(input).await
    }

    async fn update_type(
        &self,
        id: &str,
        input: CardCodeTypeUpdatePayload,
    ) -> CardCodeResult<CardCodeType> {
        let input = sanitize_type_update(input);
        validate_type_update(&input)?;
        self.repository.update_type(id, input).await
    }

    async fn list_codes(
        &self,
        page: PageRequest,
        filters: CardCodeListFilters,
    ) -> CardCodeResult<CardCodeListResponse> {
        validate_page(page)?;
        self.repository.list_codes(page, filters).await.map(Into::into)
    }

    async fn generate_codes(
        &self,
        input: CardCodeGeneratePayload,
        operator: CardCodeOperator,
    ) -> CardCodeResult<CardCodeGenerateResponse> {
        let input = sanitize_generate(input);
        validate_generate(&input)?;
        let card_type = self.active_type(&input.type_id).await?;
        let amounts = generation_amounts(&input, &card_type)?;
        let batch_no = batch_no();
        let records = self.generated_records(input, operator, card_type, amounts, &batch_no).await?;
        let items = self.repository.create_codes(records).await?;
        Ok(CardCodeGenerateResponse {
            total: items.len() as u64,
            batch_no: Some(batch_no),
            items: items.into_iter().map(Into::into).collect(),
        })
    }

    async fn batch_update_code_status(
        &self,
        input: CardCodeBatchStatusPayload,
    ) -> CardCodeResult<CardCodeBatchStatusResponse> {
        validate_batch_status(&input)?;
        let updated_count = self.repository.batch_update_code_status(&input.ids, &input.status).await?;
        Ok(CardCodeBatchStatusResponse { updated_count })
    }

    async fn redeem(
        &self,
        input: CardCodeRedeemPayload,
        user: CardCodeRedeemer,
    ) -> CardCodeResult<CardCodeRedeemResponse> {
        let code = normalize_code(&input.code)?;
        self.repository
            .redeem(CardCodeRedeemInput {
                code,
                user_id: user.user_id,
                username: user.username,
                client_ip: user.client_ip,
            })
            .await
    }
}

impl<R> CardCodeService<R>
where
    R: CardCodeRepository,
{
    async fn active_type(&self, type_id: &str) -> CardCodeResult<CardCodeType> {
        let card_type = self.repository.find_type(type_id).await?.ok_or(CardCodeError::NotFound)?;
        if card_type.status != CARD_CODE_STATUS_ACTIVE {
            return Err(CardCodeError::Conflict("card code type is disabled".into()));
        }
        Ok(card_type)
    }

    async fn generated_records(
        &self,
        input: CardCodeGeneratePayload,
        operator: CardCodeOperator,
        card_type: CardCodeType,
        amounts: CardCodeAmounts,
        batch_no: &str,
    ) -> CardCodeResult<Vec<CardCodeCreateRecord>> {
        let mut records = Vec::with_capacity(input.quantity as usize);
        for _ in 0..input.quantity {
            records.push(CardCodeCreateRecord {
                code: self.unique_code(input.code_length).await?,
                batch_no: batch_no.into(),
                type_id: card_type.id.clone(),
                type_name: card_type.name.clone(),
                recharge_amount: amounts.recharge,
                gift_amount: amounts.gift,
                status: input.status.clone().unwrap_or_else(|| CARD_CODE_STATUS_ACTIVE.into()),
                remark: input.remark.clone(),
                expires_at: input.expires_at.clone(),
                created_by_user_id: operator.user_id.clone(),
                created_by_username: operator.username.clone(),
                created_ip: operator.client_ip.clone(),
            });
        }
        Ok(records)
    }

    async fn unique_code(&self, length: u8) -> CardCodeResult<String> {
        for _ in 0..MAX_CODE_ATTEMPTS {
            let code = build_code(length);
            if !self.repository.code_exists(&code).await? {
                return Ok(code);
            }
        }
        Err(CardCodeError::Conflict("failed to generate unique card code".into()))
    }
}

#[derive(Clone, Copy)]
struct CardCodeAmounts {
    recharge: Decimal,
    gift: Decimal,
}

fn generation_amounts(
    input: &CardCodeGeneratePayload,
    card_type: &CardCodeType,
) -> CardCodeResult<CardCodeAmounts> {
    match card_type.balance_type.as_str() {
        CARD_CODE_BALANCE_TYPE_RECHARGE => Ok(CardCodeAmounts {
            recharge: input.amount,
            gift: Decimal::ZERO,
        }),
        CARD_CODE_BALANCE_TYPE_GIFT => Ok(CardCodeAmounts {
            recharge: Decimal::ZERO,
            gift: input.amount,
        }),
        _ => Err(CardCodeError::InvalidInput("balance_type must be recharge or gift".into())),
    }
}

fn sanitize_type_create(input: CardCodeTypeCreatePayload) -> CardCodeTypeCreatePayload {
    CardCodeTypeCreatePayload {
        name: input.name.trim().into(),
        balance_type: input.balance_type.trim().into(),
        status: Some(input.status.unwrap_or_else(|| CARD_CODE_STATUS_ACTIVE.into())),
        remark: trimmed_optional(input.remark),
    }
}

fn sanitize_type_update(input: CardCodeTypeUpdatePayload) -> CardCodeTypeUpdatePayload {
    CardCodeTypeUpdatePayload {
        name: input.name.trim().into(),
        balance_type: input.balance_type.trim().into(),
        status: input.status.trim().into(),
        remark: trimmed_optional(input.remark),
    }
}

fn sanitize_generate(input: CardCodeGeneratePayload) -> CardCodeGeneratePayload {
    CardCodeGeneratePayload {
        type_id: input.type_id.trim().into(),
        status: input.status.map(|value| value.trim().to_owned()),
        remark: trimmed_optional(input.remark),
        expires_at: trimmed_optional(input.expires_at),
        ..input
    }
}

fn trimmed_optional(value: Option<String>) -> Option<String> {
    value.map(|item| item.trim().to_owned()).filter(|item| !item.is_empty())
}

fn normalize_code(raw: &str) -> CardCodeResult<String> {
    let code: String = raw.chars().filter(|ch| ch.is_ascii_alphanumeric()).collect();
    let code = code.to_ascii_uppercase();
    if code.is_empty() {
        return Err(CardCodeError::InvalidInput("code is required".into()));
    }
    Ok(code)
}

fn build_code(length: u8) -> String {
    let mut rng = OsRng;
    (0..length)
        .map(|_| CODE_ALPHABET[rng.gen_range(0..CODE_ALPHABET.len())] as char)
        .collect()
}

fn batch_no() -> String {
    format!("ccb_{}", Uuid::now_v7().simple())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generation_amounts_puts_recharge_type_amount_into_recharge_balance() {
        let amounts = generation_amounts(&generate_payload(), &card_type(CARD_CODE_BALANCE_TYPE_RECHARGE)).unwrap();

        assert_eq!(amounts.recharge, Decimal::new(1000, 2));
        assert_eq!(amounts.gift, Decimal::ZERO);
    }

    #[test]
    fn generation_amounts_puts_gift_type_amount_into_gift_balance() {
        let amounts = generation_amounts(&generate_payload(), &card_type(CARD_CODE_BALANCE_TYPE_GIFT)).unwrap();

        assert_eq!(amounts.recharge, Decimal::ZERO);
        assert_eq!(amounts.gift, Decimal::new(1000, 2));
    }

    fn generate_payload() -> CardCodeGeneratePayload {
        CardCodeGeneratePayload {
            type_id: "type_1".into(),
            quantity: 1,
            code_length: 12,
            status: None,
            remark: None,
            expires_at: None,
            amount: Decimal::new(1000, 2),
        }
    }

    fn card_type(balance_type: &str) -> CardCodeType {
        CardCodeType {
            id: "type_1".into(),
            name: "type".into(),
            balance_type: balance_type.into(),
            status: CARD_CODE_STATUS_ACTIVE.into(),
            remark: None,
            created_at: "2026-05-14T00:00:00Z".into(),
            updated_at: "2026-05-14T00:00:00Z".into(),
        }
    }
}
