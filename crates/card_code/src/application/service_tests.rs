use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use rust_decimal::Decimal;
use types::{
    card_code::{
        CARD_CODE_BALANCE_TYPE_GIFT, CARD_CODE_BALANCE_TYPE_RECHARGE, CARD_CODE_STATUS_ACTIVE, CardCode, CardCodeCreateRecord, CardCodeGeneratePayload,
        CardCodeListFilters, CardCodeRedeemInput, CardCodeRedeemPayload, CardCodeRedeemResponse, CardCodeType, CardCodeTypeCreatePayload,
        CardCodeTypeListFilters, CardCodeTypeUpdatePayload,
    },
    pagination::{Page, PageRequest},
    wallet::{WalletId, WalletTransaction},
};

use super::*;
use crate::application::{CardCodeCurrencyProvider, CardCodeRepository};

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

#[tokio::test]
async fn generate_codes_uses_default_wallet_currency() {
    let service = CardCodeService::new(MockRepository, MockCurrencyProvider);
    let response = service.generate_codes(generate_payload(), operator()).await.unwrap();

    assert_eq!(response.items[0].currency, currency::DEFAULT_WALLET_CURRENCY);
}

#[tokio::test]
async fn redeem_uses_default_wallet_currency_not_display_currency() {
    let repository = RedeemRepository::default();
    let service = CardCodeService::new(repository.clone(), CnyCurrencyProvider);

    service.redeem(CardCodeRedeemPayload { code: "card-code".into() }, redeemer()).await.unwrap();

    let input = repository.redeem_input().unwrap();
    assert_eq!(input.target_currency, currency::DEFAULT_WALLET_CURRENCY);
    assert_eq!(input.usd_cny_rate, Some(Decimal::new(7, 0)));
}

#[test]
fn exchange_rate_is_required_only_for_redeemable_currency_mismatch() {
    let mut code = card_code("CNY", CARD_CODE_STATUS_ACTIVE);

    assert!(requires_exchange_rate(&code, "USD"));

    code.status = "used".into();
    assert!(!requires_exchange_rate(&code, "USD"));
    assert!(!requires_exchange_rate(&card_code("USD", CARD_CODE_STATUS_ACTIVE), "USD"));
}

struct MockRepository;

#[async_trait]
impl CardCodeRepository for MockRepository {
    async fn list_types(&self, _page: PageRequest, _filters: CardCodeTypeListFilters) -> CardCodeResult<Page<CardCodeType>> {
        unreachable!()
    }

    async fn create_type(&self, _input: CardCodeTypeCreatePayload) -> CardCodeResult<CardCodeType> {
        unreachable!()
    }

    async fn update_type(&self, _id: &str, _input: CardCodeTypeUpdatePayload) -> CardCodeResult<CardCodeType> {
        unreachable!()
    }

    async fn find_type(&self, _id: &str) -> CardCodeResult<Option<CardCodeType>> {
        Ok(Some(card_type(CARD_CODE_BALANCE_TYPE_GIFT)))
    }

    async fn find_code(&self, _code: &str) -> CardCodeResult<Option<CardCode>> {
        Ok(None)
    }

    async fn user_wallet_currency(&self, _user_id: &str) -> CardCodeResult<Option<String>> {
        Ok(None)
    }

    async fn code_exists(&self, _code: &str) -> CardCodeResult<bool> {
        Ok(false)
    }

    async fn create_codes(&self, inputs: Vec<CardCodeCreateRecord>) -> CardCodeResult<Vec<CardCode>> {
        Ok(inputs.into_iter().map(card_code_from_record).collect())
    }

    async fn list_codes(&self, _page: PageRequest, _filters: CardCodeListFilters) -> CardCodeResult<Page<CardCode>> {
        unreachable!()
    }

    async fn batch_update_code_status(&self, _ids: &[String], _status: &str) -> CardCodeResult<u64> {
        unreachable!()
    }

    async fn redeem(&self, _input: CardCodeRedeemInput) -> CardCodeResult<CardCodeRedeemResponse> {
        unreachable!()
    }
}

struct MockCurrencyProvider;

#[async_trait]
impl CardCodeCurrencyProvider for MockCurrencyProvider {
    async fn usd_cny_rate(&self) -> CardCodeResult<Decimal> {
        Ok(Decimal::new(7, 0))
    }
}

#[derive(Clone, Default)]
struct RedeemRepository {
    input: Arc<Mutex<Option<CardCodeRedeemInput>>>,
}

impl RedeemRepository {
    fn redeem_input(&self) -> Option<CardCodeRedeemInput> {
        self.input.lock().unwrap().clone()
    }
}

#[async_trait]
impl CardCodeRepository for RedeemRepository {
    async fn list_types(&self, _page: PageRequest, _filters: CardCodeTypeListFilters) -> CardCodeResult<Page<CardCodeType>> {
        unreachable!()
    }

    async fn create_type(&self, _input: CardCodeTypeCreatePayload) -> CardCodeResult<CardCodeType> {
        unreachable!()
    }

    async fn update_type(&self, _id: &str, _input: CardCodeTypeUpdatePayload) -> CardCodeResult<CardCodeType> {
        unreachable!()
    }

    async fn find_type(&self, _id: &str) -> CardCodeResult<Option<CardCodeType>> {
        unreachable!()
    }

    async fn find_code(&self, _code: &str) -> CardCodeResult<Option<CardCode>> {
        Ok(Some(card_code("CNY", CARD_CODE_STATUS_ACTIVE)))
    }

    async fn user_wallet_currency(&self, _user_id: &str) -> CardCodeResult<Option<String>> {
        Ok(None)
    }

    async fn code_exists(&self, _code: &str) -> CardCodeResult<bool> {
        unreachable!()
    }

    async fn create_codes(&self, _inputs: Vec<CardCodeCreateRecord>) -> CardCodeResult<Vec<CardCode>> {
        unreachable!()
    }

    async fn list_codes(&self, _page: PageRequest, _filters: CardCodeListFilters) -> CardCodeResult<Page<CardCode>> {
        unreachable!()
    }

    async fn batch_update_code_status(&self, _ids: &[String], _status: &str) -> CardCodeResult<u64> {
        unreachable!()
    }

    async fn redeem(&self, input: CardCodeRedeemInput) -> CardCodeResult<CardCodeRedeemResponse> {
        *self.input.lock().unwrap() = Some(input);
        Ok(CardCodeRedeemResponse {
            card_code: card_code("CNY", CARD_CODE_STATUS_ACTIVE).into(),
            transaction: wallet_transaction().into(),
        })
    }
}

struct CnyCurrencyProvider;

#[async_trait]
impl CardCodeCurrencyProvider for CnyCurrencyProvider {
    async fn usd_cny_rate(&self) -> CardCodeResult<Decimal> {
        Ok(Decimal::new(7, 0))
    }
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

fn redeemer() -> CardCodeRedeemer {
    CardCodeRedeemer {
        user_id: "user_1".into(),
        username: "alice".into(),
        client_ip: Some("127.0.0.1".into()),
    }
}

fn operator() -> CardCodeOperator {
    CardCodeOperator {
        user_id: Some("user_1".into()),
        username: Some("admin".into()),
        client_ip: Some("127.0.0.1".into()),
    }
}

fn card_type(balance_type: &str) -> CardCodeType {
    CardCodeType {
        id: "type_1".into(),
        name: "type".into(),
        balance_type: balance_type.into(),
        status: CARD_CODE_STATUS_ACTIVE.into(),
        remark: None,
        created_at: timestamp(),
        updated_at: timestamp(),
    }
}

fn card_code(currency: &str, status: &str) -> CardCode {
    CardCode {
        id: "code_1".into(),
        code: "CARD-CODE".into(),
        batch_no: "batch_1".into(),
        type_id: "type_1".into(),
        type_name: "type".into(),
        recharge_amount: Decimal::new(1000, 2),
        gift_amount: Decimal::ZERO,
        currency: currency.into(),
        status: status.into(),
        remark: None,
        expires_at: None,
        created_at: timestamp(),
        updated_at: timestamp(),
        created_by_user_id: None,
        created_by_username: None,
        created_ip: None,
        used_by_user_id: None,
        used_by_username: None,
        used_ip: None,
        used_at: None,
        wallet_id: None,
        wallet_transaction_id: None,
    }
}

fn card_code_from_record(input: CardCodeCreateRecord) -> CardCode {
    CardCode {
        id: "code_id".into(),
        code: input.code,
        batch_no: input.batch_no,
        type_id: input.type_id,
        type_name: input.type_name,
        recharge_amount: input.recharge_amount,
        gift_amount: input.gift_amount,
        currency: input.currency,
        status: input.status,
        remark: input.remark,
        expires_at: input.expires_at,
        created_at: timestamp(),
        updated_at: timestamp(),
        created_by_user_id: input.created_by_user_id,
        created_by_username: input.created_by_username,
        created_ip: input.created_ip,
        used_by_user_id: None,
        used_by_username: None,
        used_ip: None,
        used_at: None,
        wallet_id: None,
        wallet_transaction_id: None,
    }
}

fn wallet_transaction() -> WalletTransaction {
    WalletTransaction {
        id: "tx_1".into(),
        wallet_id: WalletId("wallet_1".into()).0,
        category: "recharge".into(),
        reason_code: "topup_card_code".into(),
        amount: Decimal::new(1000, 2),
        balance_before: Decimal::ZERO,
        balance_after: Decimal::new(1000, 2),
        recharge_balance_before: Decimal::ZERO,
        recharge_balance_after: Decimal::new(1000, 2),
        gift_balance_before: Decimal::ZERO,
        gift_balance_after: Decimal::ZERO,
        link_type: None,
        link_id: None,
        operator_id: None,
        description: None,
        created_at: timestamp(),
    }
}

fn timestamp() -> String {
    "2026-05-14T00:00:00Z".into()
}
