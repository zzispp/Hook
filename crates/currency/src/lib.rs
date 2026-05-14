use rust_decimal::Decimal;
use thiserror::Error;

pub const USD: &str = "USD";
pub const CNY: &str = "CNY";
pub const ACCOUNTING_CURRENCY: &str = USD;
pub const DEFAULT_WALLET_CURRENCY: &str = CNY;

const MONEY_SCALE: u32 = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CurrencyCode {
    Usd,
    Cny,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CurrencyError {
    #[error("unsupported currency: {0}")]
    UnsupportedCurrency(String),
    #[error("unsupported currency conversion: {from} -> {to}")]
    UnsupportedConversion { from: String, to: String },
    #[error("USD/CNY exchange rate is required")]
    MissingUsdCnyRate,
    #[error("USD/CNY exchange rate must be greater than 0")]
    InvalidUsdCnyRate,
}

pub fn convert_amount(value: Decimal, source: &str, target: &str, usd_cny_rate: Option<Decimal>) -> Result<Decimal, CurrencyError> {
    let source = CurrencyCode::try_from(source)?;
    let target = CurrencyCode::try_from(target)?;
    if source == target {
        return Ok(value);
    }
    let rate = valid_usd_cny_rate(usd_cny_rate)?;
    match (source, target) {
        (CurrencyCode::Usd, CurrencyCode::Cny) => Ok((value * rate).round_dp(MONEY_SCALE)),
        (CurrencyCode::Cny, CurrencyCode::Usd) => Ok((value / rate).round_dp(MONEY_SCALE)),
        _ => Err(CurrencyError::UnsupportedConversion {
            from: source.as_str().into(),
            to: target.as_str().into(),
        }),
    }
}

impl CurrencyCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Usd => USD,
            Self::Cny => CNY,
        }
    }
}

impl TryFrom<&str> for CurrencyCode {
    type Error = CurrencyError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            USD => Ok(Self::Usd),
            CNY => Ok(Self::Cny),
            _ => Err(CurrencyError::UnsupportedCurrency(value.into())),
        }
    }
}

fn valid_usd_cny_rate(rate: Option<Decimal>) -> Result<Decimal, CurrencyError> {
    let rate = rate.ok_or(CurrencyError::MissingUsdCnyRate)?;
    if rate <= Decimal::ZERO {
        return Err(CurrencyError::InvalidUsdCnyRate);
    }
    Ok(rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_usd_to_cny() {
        let amount = convert_amount(Decimal::new(10, 0), USD, CNY, Some(Decimal::new(72, 1))).unwrap();
        assert_eq!(amount, Decimal::new(720000000, 7));
    }

    #[test]
    fn converts_cny_to_usd() {
        let amount = convert_amount(Decimal::new(72, 0), CNY, USD, Some(Decimal::new(72, 1))).unwrap();
        assert_eq!(amount, Decimal::new(100000000, 7));
    }

    #[test]
    fn same_currency_does_not_require_rate() {
        let amount = convert_amount(Decimal::new(1234, 2), USD, USD, None).unwrap();
        assert_eq!(amount, Decimal::new(1234, 2));
    }

    #[test]
    fn missing_rate_is_visible() {
        let error = convert_amount(Decimal::ONE, USD, CNY, None).unwrap_err();
        assert_eq!(error, CurrencyError::MissingUsdCnyRate);
    }

    #[test]
    fn unsupported_currency_is_visible() {
        let error = convert_amount(Decimal::ONE, "EUR", CNY, Some(Decimal::ONE)).unwrap_err();
        assert_eq!(error, CurrencyError::UnsupportedCurrency("EUR".into()));
    }
}
