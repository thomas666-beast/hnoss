//! Monetary value object.

use std::fmt;

use crate::currency::Currency;
use crate::error::MoneyError;

/// An immutable amount of money in a given currency.
///
/// The amount is stored as an `i128` in the currency's smallest unit (e.g.
/// cents for USD, satoshi for BTC). No floating point is used anywhere.
///
/// # Example
///
/// ```
/// use hnoss::{Currency, Money};
///
/// let usd = Currency::new("USD").unwrap();
/// let price = Money::new(500_i128, usd.clone());
///
/// assert_eq!(price.amount(), 500);
/// assert_eq!(price.currency(), &usd);
/// assert_eq!(price.as_string(), "500 USD");
/// assert_eq!(format!("{price}"), "500 USD");
/// ```
///
/// # Construction
///
/// The canonical constructor is [`Money::new`], which takes the amount in
/// minor units and a `Currency` object. For convenience, [`Money::from_code`]
/// wraps a currency-code string.
///
/// Arithmetic (add, subtract, ...) is not implemented yet; it arrives in a
/// later release.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Money {
    amount: i128,
    currency: Currency,
}

impl Money {
    /// Constructs a `Money` from an amount in the currency's smallest unit.
    ///
    /// # Example
    ///
    /// ```
    /// use hnoss::{Currency, Money};
    ///
    /// let usd = Currency::new("USD").unwrap();
    /// let m = Money::new(500_i128, usd);
    /// assert_eq!(m.amount(), 500);
    /// ```
    pub fn new(amount: i128, currency: Currency) -> Self {
        Money { amount, currency }
    }

    /// Constructs a `Money` from an amount and a currency code string.
    ///
    /// This is a convenience over [`Money::new`] that wraps the code in a
    /// [`Currency`] first.
    ///
    /// # Errors
    ///
    /// Returns [`MoneyError::InvalidCurrency`] if the code is not 3 to 8
    /// ASCII letters.
    ///
    /// # Example
    ///
    /// ```
    /// use hnoss::Money;
    ///
    /// let m = Money::from_code(500_i128, "USD").unwrap();
    /// assert_eq!(m.as_string(), "500 USD");
    ///
    /// assert!(Money::from_code(500_i128, "U5D").is_err());
    /// ```
    pub fn from_code(amount: i128, code: impl AsRef<str>) -> Result<Self, MoneyError> {
        let currency = Currency::new(code)?;
        Ok(Money::new(amount, currency))
    }

    /// Returns the amount in the currency's smallest unit.
    ///
    /// For USD, this is cents. For BTC, this is satoshi. The value is
    /// already an integer; there is nothing to convert or round.
    pub fn amount(&self) -> i128 {
        self.amount
    }

    /// Returns a reference to the currency.
    pub fn currency(&self) -> &Currency {
        &self.currency
    }

    /// Returns the canonical, unambiguous string form: `"<amount> <code>"`.
    ///
    /// This is not the same as a human-facing display format. For example,
    /// a `Money` of 500 USD cents renders as `"500 USD"`, not `"$5.00"`. Use
    /// a formatting method (coming in a later release) for display.
    ///
    /// # Example
    ///
    /// ```
    /// use hnoss::Money;
    ///
    /// let m = Money::from_code(500_i128, "USD").unwrap();
    /// assert_eq!(m.as_string(), "500 USD");
    /// ```
    pub fn as_string(&self) -> String {
        format!("{} {}", self.amount, self.currency.code())
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.amount, self.currency)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn usd() -> Currency {
        Currency::new("USD").unwrap()
    }

    fn eur() -> Currency {
        Currency::new("EUR").unwrap()
    }

    #[test]
    fn new_stores_fields() {
        let m = Money::new(500, usd());
        assert_eq!(m.amount(), 500);
        assert_eq!(m.currency().code(), "USD");
    }

    #[test]
    fn from_code_wraps_currency() {
        let m = Money::from_code(500, "USD").unwrap();
        assert_eq!(m.amount(), 500);
        assert_eq!(m.currency().code(), "USD");
    }

    #[test]
    fn from_code_normalizes_case() {
        let m = Money::from_code(500, "usd").unwrap();
        assert_eq!(m.currency().code(), "USD");
    }

    #[test]
    fn from_code_rejects_bad_code() {
        assert!(matches!(
            Money::from_code(500, "U5D"),
            Err(MoneyError::InvalidCurrency(_))
        ));
        assert!(matches!(
            Money::from_code(500, "US"),
            Err(MoneyError::InvalidCurrency(_))
        ));
    }

    #[test]
    fn as_string_format() {
        let m = Money::new(500, usd());
        assert_eq!(m.as_string(), "500 USD");

        let n = Money::new(-5, usd());
        assert_eq!(n.as_string(), "-5 USD");

        let z = Money::new(0, usd());
        assert_eq!(z.as_string(), "0 USD");
    }

    #[test]
    fn display_matches_as_string() {
        let m = Money::new(500, usd());
        assert_eq!(format!("{m}"), m.as_string());
    }

    #[test]
    fn debug_is_derived() {
        let m = Money::new(500, usd());
        let s = format!("{m:?}");
        assert!(s.contains("Money"));
        assert!(s.contains("500"));
        assert!(s.contains("USD"));
    }

    #[test]
    fn equality_same_currency() {
        let a = Money::new(500, usd());
        let b = Money::new(500, usd());
        assert_eq!(a, b);
    }

    #[test]
    fn inequality_same_currency() {
        let a = Money::new(500, usd());
        let b = Money::new(501, usd());
        assert_ne!(a, b);
    }

    #[test]
    fn inequality_different_currency() {
        let a = Money::new(500, usd());
        let b = Money::new(500, eur());
        assert_ne!(a, b);
    }

    #[test]
    fn equality_uses_normalized_currency() {
        let a = Money::new(500, Currency::new("USD").unwrap());
        let b = Money::new(500, Currency::new("usd").unwrap());
        assert_eq!(a, b);
    }

    #[test]
    fn can_be_used_as_hash_key() {
        use std::collections::HashSet;
        let mut s = HashSet::new();
        s.insert(Money::new(500, usd()));
        s.insert(Money::new(500, usd())); // duplicate
        s.insert(Money::new(500, eur()));
        assert_eq!(s.len(), 2);
    }

    #[test]
    fn extreme_amounts_are_representable() {
        // i128 range in minor units: about 1.7e38. That's a lot of cents.
        let max = Money::new(i128::MAX, usd());
        let min = Money::new(i128::MIN, usd());
        assert_eq!(max.amount(), i128::MAX);
        assert_eq!(min.amount(), i128::MIN);
    }
}
