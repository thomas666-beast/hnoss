//! Monetary value object.

use std::cmp::Ordering;
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
///
/// assert!(a.compare(&b).unwrap().is_gt());
/// assert!(a.is_positive());
/// assert!(!a.is_zero());
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
/// `i128::MIN`.
///
/// # Comparison
///
/// [`Money::equals`] and [`Money::compare`] are the same-currency comparison
/// methods. Equality (`==`, [`Money::equals`]) never fails and is total;
/// ordering (`compare`) fails on currency mismatch. The `Ord` and `PartialOrd`
/// traits are deliberately not implemented, because `cmp` cannot fail and
/// panicking on currency mismatch would be a footgun.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Money {
    amount: i128,
    currency: Currency,
}

impl Money {
    /// Constructs a `Money` from an amount in the currency's smallest unit.
    pub fn new(amount: i128, currency: Currency) -> Self {
        Money { amount, currency }
    }

    /// Constructs a `Money` from an amount and a currency code string.
    ///
    /// # Errors
    ///
    /// Returns [`MoneyError::InvalidCurrency`] if the code is not 3 to 8
    /// ASCII letters.
    pub fn from_code(amount: i128, code: impl AsRef<str>) -> Result<Self, MoneyError> {
        let currency = Currency::new(code)?;
        Ok(Money::new(amount, currency))
    }

    /// Returns the amount in the currency's smallest unit.
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
    /// a `Money` of 500 USD cents renders as `"500 USD"`, not `"$5.00"`.
    pub fn as_string(&self) -> String {
        format!("{} {}", self.amount, self.currency.code())
    }

    // --- arithmetic ---------------------------------------------------------

    /// Returns `self + other`.
    ///
    /// # Errors
    ///
    /// * [`MoneyError::CurrencyMismatch`] if the currencies differ.
    /// * [`MoneyError::Overflow`] if the result does not fit in `i128`.
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
    /// # Errors
    ///
    /// * [`MoneyError::CurrencyMismatch`] if the currencies differ.
    /// * [`MoneyError::Overflow`] if the result does not fit in `i128`.
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
    /// [`MoneyError::Overflow`] if `self.amount == i128::MIN`.
    pub fn negate(&self) -> Result<Money, MoneyError> {
        let amount = self.amount.checked_neg().ok_or(MoneyError::Overflow)?;
        Ok(Money::new(amount, self.currency.clone()))
    }

    /// Returns `|self|`.
    ///
    /// # Errors
    ///
    /// [`MoneyError::Overflow`] if `self.amount == i128::MIN`.
    pub fn abs(&self) -> Result<Money, MoneyError> {
        let amount = self.amount.checked_abs().ok_or(MoneyError::Overflow)?;
        Ok(Money::new(amount, self.currency.clone()))
    }

    // --- comparison ---------------------------------------------------------

    /// Compares two `Money` in the same currency.
    ///
    /// Returns [`Ordering::Less`] if `self < other`, [`Ordering::Equal`] if
    /// equal, [`Ordering::Greater`] if greater.
    ///
    /// # Errors
    ///
    /// [`MoneyError::CurrencyMismatch`] if the currencies differ.
    ///
    /// # Example
    ///
    /// ```
    /// use hnoss::Money;
    /// use std::cmp::Ordering;
    ///
    /// let a = Money::from_code(500_i128, "USD").unwrap();
    /// let b = Money::from_code(250_i128, "USD").unwrap();
    /// assert_eq!(a.compare(&b).unwrap(), Ordering::Greater);
    /// assert_eq!(b.compare(&a).unwrap(), Ordering::Less);
    /// assert_eq!(a.compare(&a).unwrap(), Ordering::Equal);
    /// ```
    pub fn compare(&self, other: &Money) -> Result<Ordering, MoneyError> {
        self.check_same_currency(other)?;
        Ok(self.amount.cmp(&other.amount))
    }

    /// Returns true if `self` and `other` are equal.
    ///
    /// This is the same as `==` (derived `PartialEq`); it's provided as a
    /// named method mirroring the Perl API and to make the intent explicit
    /// at call sites. `equals` never fails: two `Money` in different
    /// currencies are unequal, not an error.
    ///
    /// Use [`Money::compare`] if you want a currency mismatch to be an
    /// error.
    pub fn equals(&self, other: &Money) -> bool {
        self == other
    }

    /// Returns true if the amount is exactly zero.
    pub fn is_zero(&self) -> bool {
        self.amount == 0
    }

    /// Returns true if the amount is strictly greater than zero.
    ///
    /// `is_positive` is false for zero: a zero amount is neither positive
    /// nor negative. If you need "non-negative", use `!money.is_negative()`.
    pub fn is_positive(&self) -> bool {
        self.amount > 0
    }

    /// Returns true if the amount is strictly less than zero.
    pub fn is_negative(&self) -> bool {
        self.amount < 0
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
    fn from_code_rejects_bad_code() {
        assert!(matches!(
            Money::from_code(500, "U5D"),
            Err(MoneyError::InvalidCurrency(_))
        ));
    }

    // --- display -------------------------------------------------------------

    #[test]
    fn as_string_format() {
        assert_eq!(Money::new(500, usd()).as_string(), "500 USD");
        assert_eq!(Money::new(-5, usd()).as_string(), "-5 USD");
    }

    #[test]
    fn display_matches_as_string() {
        let m = Money::new(500, usd());
        assert_eq!(format!("{m}"), m.as_string());
    }

    // --- equality ------------------------------------------------------------

    #[test]
    fn equality_same_currency() {
        assert_eq!(Money::new(500, usd()), Money::new(500, usd()));
    }

    #[test]
    fn inequality_different_amount() {
        assert_ne!(Money::new(500, usd()), Money::new(501, usd()));
    }

    #[test]
    fn inequality_different_currency() {
        assert_ne!(Money::new(500, usd()), Money::new(500, eur()));
    }

    #[test]
    fn equals_matches_partial_eq() {
        let cases = [
            (Money::new(500, usd()), Money::new(500, usd()), true),
            (Money::new(500, usd()), Money::new(501, usd()), false),
            (Money::new(500, usd()), Money::new(500, eur()), false),
            (Money::new(0, usd()), Money::new(0, usd()), true),
        ];
        for (a, b, expected) in cases {
            assert_eq!(a == b, expected, "{a} == {b}");
            assert_eq!(a.equals(&b), expected, "{a}.equals({b})");
            assert_eq!(
                a == b,
                a.equals(&b),
                "== and equals must agree for {a} vs {b}"
            );
        }
    }

    // --- compare -------------------------------------------------------------

    #[test]
    fn compare_ordering() {
        let a = Money::new(500, usd());
        let b = Money::new(250, usd());
        let c = Money::new(500, usd());

        assert_eq!(a.compare(&b).unwrap(), Ordering::Greater);
        assert_eq!(b.compare(&a).unwrap(), Ordering::Less);
        assert_eq!(a.compare(&c).unwrap(), Ordering::Equal);
    }

    #[test]
    fn compare_zero() {
        let z = Money::new(0, usd());
        let p = Money::new(1, usd());
        let n = Money::new(-1, usd());
        assert_eq!(z.compare(&z).unwrap(), Ordering::Equal);
        assert_eq!(z.compare(&p).unwrap(), Ordering::Less);
        assert_eq!(z.compare(&n).unwrap(), Ordering::Greater);
    }

    #[test]
    fn compare_across_currencies_errors() {
        let a = Money::new(500, usd());
        let b = Money::new(500, eur());
        match a.compare(&b).unwrap_err() {
            MoneyError::CurrencyMismatch { left, right } => {
                assert_eq!(left, "USD");
                assert_eq!(right, "EUR");
            }
            other => panic!("expected CurrencyMismatch, got {other:?}"),
        }
    }

    // --- predicates ----------------------------------------------------------

    #[test]
    fn is_zero() {
        assert!(Money::new(0, usd()).is_zero());
        assert!(!Money::new(1, usd()).is_zero());
        assert!(!Money::new(-1, usd()).is_zero());
    }

    #[test]
    fn is_positive() {
        assert!(Money::new(1, usd()).is_positive());
        assert!(!Money::new(0, usd()).is_positive());
        assert!(!Money::new(-1, usd()).is_positive());
    }

    #[test]
    fn is_negative() {
        assert!(Money::new(-1, usd()).is_negative());
        assert!(!Money::new(0, usd()).is_negative());
        assert!(!Money::new(1, usd()).is_negative());
    }

    #[test]
    fn predicates_are_mutually_consistent() {
        for amount in [-100_i128, -1, 0, 1, 100] {
            let m = Money::new(amount, usd());
            let signs = [m.is_zero(), m.is_positive(), m.is_negative()];
            let true_count = signs.iter().filter(|b| **b).count();
            assert_eq!(
                true_count, 1,
                "amount {amount}: exactly one of zero/positive/negative must hold"
            );
        }
    }
}
