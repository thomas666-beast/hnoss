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
/// let a = Money::new(500_i128, usd.clone());
/// let b = Money::new(250_i128, usd.clone());
///
/// assert_eq!(a.add(&b).unwrap().as_string(), "750 USD");
/// assert_eq!(a.subtract(&b).unwrap().as_string(), "250 USD");
/// assert_eq!(a.negate().unwrap().as_string(), "-500 USD");
/// assert_eq!(a.abs().unwrap().as_string(), "500 USD");
/// ```
///
/// # Construction
///
/// The canonical constructor is [`Money::new`], which takes the amount in
/// minor units and a `Currency` object. For convenience, [`Money::from_code`]
/// wraps a currency-code string.
///
/// # Arithmetic
///
/// All arithmetic takes `&self` and returns a new `Money`; the receiver is
/// never modified. Every operation is fallible: `add` / `subtract` fail on
/// currency mismatch or `i128` overflow, and `negate` / `abs` fail on
/// `i128::MIN` (whose negation overflows).
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

    // --- arithmetic ---------------------------------------------------------

    /// Returns `self + other`.
    ///
    /// Both must be in the same currency. The receiver is not modified; a
    /// new `Money` is returned.
    ///
    /// # Errors
    ///
    /// * [`MoneyError::CurrencyMismatch`] if `self.currency != other.currency`.
    /// * [`MoneyError::Overflow`] if the result does not fit in `i128`.
    ///
    /// # Example
    ///
    /// ```
    /// use hnoss::Money;
    ///
    /// let a = Money::from_code(500_i128, "USD").unwrap();
    /// let b = Money::from_code(250_i128, "USD").unwrap();
    /// assert_eq!(a.add(&b).unwrap().as_string(), "750 USD");
    ///
    /// let eur = Money::from_code(500_i128, "EUR").unwrap();
    /// assert!(a.add(&eur).is_err());
    /// ```
    pub fn add(&self, other: &Money) -> Result<Money, MoneyError> {
        self.check_same_currency(other)?;
        let amount = self
            .amount
            .checked_add(other.amount)
            .ok_or(MoneyError::Overflow)?;
        Ok(Money::new(amount, self.currency.clone()))
    }

    /// Returns `self - other`.
    ///
    /// Both must be in the same currency. The receiver is not modified; a
    /// new `Money` is returned. The result may be negative.
    ///
    /// # Errors
    ///
    /// * [`MoneyError::CurrencyMismatch`] if `self.currency != other.currency`.
    /// * [`MoneyError::Overflow`] if the result does not fit in `i128`.
    ///
    /// # Example
    ///
    /// ```
    /// use hnoss::Money;
    ///
    /// let a = Money::from_code(500_i128, "USD").unwrap();
    /// let b = Money::from_code(250_i128, "USD").unwrap();
    /// assert_eq!(a.subtract(&b).unwrap().as_string(), "250 USD");
    /// assert_eq!(b.subtract(&a).unwrap().as_string(), "-250 USD");
    /// ```
    pub fn subtract(&self, other: &Money) -> Result<Money, MoneyError> {
        self.check_same_currency(other)?;
        let amount = self
            .amount
            .checked_sub(other.amount)
            .ok_or(MoneyError::Overflow)?;
        Ok(Money::new(amount, self.currency.clone()))
    }

    /// Returns `-self`.
    ///
    /// # Errors
    ///
    /// [`MoneyError::Overflow`] if `self.amount == i128::MIN`, whose negation
    /// does not fit in `i128`. This is the only failure mode.
    ///
    /// # Example
    ///
    /// ```
    /// use hnoss::Money;
    ///
    /// let a = Money::from_code(500_i128, "USD").unwrap();
    /// assert_eq!(a.negate().unwrap().as_string(), "-500 USD");
    ///
    /// let b = Money::from_code(-500_i128, "USD").unwrap();
    /// assert_eq!(b.negate().unwrap().as_string(), "500 USD");
    /// ```
    pub fn negate(&self) -> Result<Money, MoneyError> {
        let amount = self.amount.checked_neg().ok_or(MoneyError::Overflow)?;
        Ok(Money::new(amount, self.currency.clone()))
    }

    /// Returns `|self|`.
    ///
    /// # Errors
    ///
    /// [`MoneyError::Overflow`] if `self.amount == i128::MIN`, whose absolute
    /// value does not fit in `i128`. This is the only failure mode.
    ///
    /// # Example
    ///
    /// ```
    /// use hnoss::Money;
    ///
    /// let a = Money::from_code(-500_i128, "USD").unwrap();
    /// assert_eq!(a.abs().unwrap().as_string(), "500 USD");
    ///
    /// let b = Money::from_code(500_i128, "USD").unwrap();
    /// assert_eq!(b.abs().unwrap().as_string(), "500 USD");
    /// ```
    pub fn abs(&self) -> Result<Money, MoneyError> {
        let amount = self.amount.checked_abs().ok_or(MoneyError::Overflow)?;
        Ok(Money::new(amount, self.currency.clone()))
    }

    // --- internals ----------------------------------------------------------

    fn check_same_currency(&self, other: &Money) -> Result<(), MoneyError> {
        if self.currency != other.currency {
            return Err(MoneyError::CurrencyMismatch {
                left: self.currency.code().to_string(),
                right: other.currency.code().to_string(),
            });
        }
        Ok(())
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

    // --- construction -------------------------------------------------------

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

    // --- display / stringification ------------------------------------------

    #[test]
    fn as_string_format() {
        assert_eq!(Money::new(500, usd()).as_string(), "500 USD");
        assert_eq!(Money::new(-5, usd()).as_string(), "-5 USD");
        assert_eq!(Money::new(0, usd()).as_string(), "0 USD");
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

    // --- equality ------------------------------------------------------------

    #[test]
    fn equality_same_currency() {
        assert_eq!(Money::new(500, usd()), Money::new(500, usd()));
    }

    #[test]
    fn inequality_same_currency() {
        assert_ne!(Money::new(500, usd()), Money::new(501, usd()));
    }

    #[test]
    fn inequality_different_currency() {
        assert_ne!(Money::new(500, usd()), Money::new(500, eur()));
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
        s.insert(Money::new(500, usd()));
        s.insert(Money::new(500, eur()));
        assert_eq!(s.len(), 2);
    }

    #[test]
    fn extreme_amounts_are_representable() {
        let max = Money::new(i128::MAX, usd());
        let min = Money::new(i128::MIN, usd());
        assert_eq!(max.amount(), i128::MAX);
        assert_eq!(min.amount(), i128::MIN);
    }

    // --- add -----------------------------------------------------------------

    #[test]
    fn add_returns_new_money() {
        let a = Money::new(500, usd());
        let b = Money::new(250, usd());
        let sum = a.add(&b).unwrap();
        assert_eq!(sum.as_string(), "750 USD");
        // Originals unchanged
        assert_eq!(a.as_string(), "500 USD");
        assert_eq!(b.as_string(), "250 USD");
    }

    #[test]
    fn add_zero() {
        let a = Money::new(500, usd());
        let z = Money::new(0, usd());
        assert_eq!(a.add(&z).unwrap(), a);
    }

    #[test]
    fn add_negative() {
        let a = Money::new(500, usd());
        let b = Money::new(-200, usd());
        assert_eq!(a.add(&b).unwrap().as_string(), "300 USD");
    }

    #[test]
    fn add_across_currencies_errors() {
        let a = Money::new(500, usd());
        let b = Money::new(500, eur());
        let err = a.add(&b).unwrap_err();
        match err {
            MoneyError::CurrencyMismatch { left, right } => {
                assert_eq!(left, "USD");
                assert_eq!(right, "EUR");
            }
            other => panic!("expected CurrencyMismatch, got {other:?}"),
        }
    }

    #[test]
    fn add_overflow() {
        let a = Money::new(i128::MAX, usd());
        let b = Money::new(1, usd());
        assert!(matches!(a.add(&b), Err(MoneyError::Overflow)));
    }

    // --- subtract ------------------------------------------------------------

    #[test]
    fn subtract_returns_new_money() {
        let a = Money::new(500, usd());
        let b = Money::new(250, usd());
        let diff = a.subtract(&b).unwrap();
        assert_eq!(diff.as_string(), "250 USD");
        assert_eq!(a.as_string(), "500 USD");
        assert_eq!(b.as_string(), "250 USD");
    }

    #[test]
    fn subtract_to_negative() {
        let a = Money::new(250, usd());
        let b = Money::new(500, usd());
        assert_eq!(a.subtract(&b).unwrap().as_string(), "-250 USD");
    }

    #[test]
    fn subtract_across_currencies_errors() {
        let a = Money::new(500, usd());
        let b = Money::new(500, eur());
        match a.subtract(&b).unwrap_err() {
            MoneyError::CurrencyMismatch { left, right } => {
                assert_eq!(left, "USD");
                assert_eq!(right, "EUR");
            }
            other => panic!("expected CurrencyMismatch, got {other:?}"),
        }
    }

    #[test]
    fn subtract_overflow() {
        let a = Money::new(i128::MIN, usd());
        let b = Money::new(1, usd());
        assert!(matches!(a.subtract(&b), Err(MoneyError::Overflow)));
    }

    // --- negate --------------------------------------------------------------

    #[test]
    fn negate_positive() {
        let a = Money::new(500, usd());
        assert_eq!(a.negate().unwrap().as_string(), "-500 USD");
    }

    #[test]
    fn negate_negative() {
        let a = Money::new(-500, usd());
        assert_eq!(a.negate().unwrap().as_string(), "500 USD");
    }

    #[test]
    fn negate_zero() {
        let a = Money::new(0, usd());
        assert_eq!(a.negate().unwrap().as_string(), "0 USD");
    }

    #[test]
    fn negate_i128_min_overflows() {
        let a = Money::new(i128::MIN, usd());
        assert!(matches!(a.negate(), Err(MoneyError::Overflow)));
    }

    // --- abs -----------------------------------------------------------------

    #[test]
    fn abs_positive() {
        let a = Money::new(500, usd());
        assert_eq!(a.abs().unwrap().as_string(), "500 USD");
    }

    #[test]
    fn abs_negative() {
        let a = Money::new(-500, usd());
        assert_eq!(a.abs().unwrap().as_string(), "500 USD");
    }

    #[test]
    fn abs_zero() {
        let a = Money::new(0, usd());
        assert_eq!(a.abs().unwrap().as_string(), "0 USD");
    }

    #[test]
    fn abs_i128_min_overflows() {
        let a = Money::new(i128::MIN, usd());
        assert!(matches!(a.abs(), Err(MoneyError::Overflow)));
    }
}
