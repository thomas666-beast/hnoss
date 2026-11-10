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
///
/// assert_eq!(a.multiply(3).unwrap().as_string(), "1500 USD");
/// assert_eq!(a.divide(2).unwrap().as_string(), "250 USD");
///
/// let (q, r) = a.divide_with_remainder(3).unwrap();
/// assert_eq!(q.as_string(), "166 USD");
/// assert_eq!(r.as_string(), "2 USD");
/// ```
///
/// # Arithmetic
///
/// All arithmetic takes `&self` and returns a new `Money`; the receiver is
/// never modified. Every operation is fallible:
///
/// * `add` / `subtract` fail on currency mismatch or `i128` overflow.
/// * `multiply` fails on `i128` overflow.
/// * `divide` / `divide_with_remainder` fail on zero divisor, `i128::MIN / -1`,
///   or `i128` overflow of the quotient.
/// * `negate` / `abs` fail on `i128::MIN`.
///
/// # Division uses floor semantics
///
/// `divide` and `divide_with_remainder` round toward negative infinity, like
/// Python's `//` and Perl's `int()` in integer contexts. For positive
/// amounts this matches truncation. For negative amounts it differs:
/// `-500 / 3` returns quotient `-167`, remainder `1`.
///
/// The invariant `q * divisor + r == original` holds exactly, with `r`'s
/// sign matching the divisor's.
///
/// # Comparison
///
/// [`Money::equals`] and [`Money::compare`] are the same-currency comparison
/// methods. Equality never fails and is total; ordering fails on currency
/// mismatch. The `Ord` and `PartialOrd` traits are deliberately not
/// implemented.
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
    /// # Errors
    ///
    /// [`MoneyError::CurrencyMismatch`] if the currencies differ.
    pub fn compare(&self, other: &Money) -> Result<Ordering, MoneyError> {
        self.check_same_currency(other)?;
        Ok(self.amount.cmp(&other.amount))
    }

    /// Returns true if `self` and `other` are equal.
    ///
    /// Same semantics as `==`. Never fails.
    pub fn equals(&self, other: &Money) -> bool {
        self == other
    }

    /// Returns true if the amount is exactly zero.
    pub fn is_zero(&self) -> bool {
        self.amount == 0
    }

    /// Returns true if the amount is strictly greater than zero.
    pub fn is_positive(&self) -> bool {
        self.amount > 0
    }

    /// Returns true if the amount is strictly less than zero.
    pub fn is_negative(&self) -> bool {
        self.amount < 0
    }

    // --- multiplication and division ----------------------------------------

    /// Returns `self * factor`.
    ///
    /// `factor` must be an integer. Multiplication is exact; there is no
    /// rounding.
    ///
    /// # Errors
    ///
    /// [`MoneyError::Overflow`] if the result does not fit in `i128`.
    ///
    /// # Example
    ///
    /// ```
    /// use hnoss::Money;
    ///
    /// let a = Money::from_code(500_i128, "USD").unwrap();
    /// assert_eq!(a.multiply(3).unwrap().as_string(), "1500 USD");
    /// assert_eq!(a.multiply(0).unwrap().as_string(), "0 USD");
    /// assert_eq!(a.multiply(-1).unwrap().as_string(), "-500 USD");
    /// ```
    pub fn multiply(&self, factor: i128) -> Result<Money, MoneyError> {
        let amount = self
            .amount
            .checked_mul(factor)
            .ok_or(MoneyError::Overflow)?;
        Ok(Money::new(amount, self.currency.clone()))
    }

    /// Returns `self / divisor` using **floor** division.
    ///
    /// For positive amounts, this matches truncation. For negative amounts,
    /// the quotient rounds toward negative infinity:
    ///
    /// * ` 500 / 3 =  166` (remainder  2, discarded)
    /// * `-500 / 3 = -167` (remainder  1, discarded)
    ///
    /// Use [`Money::divide_with_remainder`] to get the remainder.
    ///
    /// # Errors
    ///
    /// * [`MoneyError::DivisionByZero`] if `divisor == 0`.
    /// * [`MoneyError::Overflow`] if `self.amount == i128::MIN` and
    ///   `divisor == -1`.
    ///
    /// # Example
    ///
    /// ```
    /// use hnoss::Money;
    ///
    /// let a = Money::from_code(500_i128, "USD").unwrap();
    /// assert_eq!(a.divide(2).unwrap().as_string(), "250 USD");
    /// assert_eq!(a.divide(3).unwrap().as_string(), "166 USD");
    ///
    /// let neg = Money::from_code(-500_i128, "USD").unwrap();
    /// assert_eq!(neg.divide(3).unwrap().as_string(), "-167 USD");
    /// ```
    pub fn divide(&self, divisor: i128) -> Result<Money, MoneyError> {
        let (q, _r) = self.divide_with_remainder(divisor)?;
        Ok(q)
    }

    /// Returns `(quotient, remainder)` from floor-dividing by `divisor`.
    ///
    /// The invariant `quotient * divisor + remainder == self` holds exactly,
    /// with `remainder`'s sign matching the divisor's (or zero).
    ///
    /// # Errors
    ///
    /// * [`MoneyError::DivisionByZero`] if `divisor == 0`.
    /// * [`MoneyError::Overflow`] if `self.amount == i128::MIN` and
    ///   `divisor == -1`.
    ///
    /// # Example
    ///
    /// ```
    /// use hnoss::Money;
    ///
    /// let a = Money::from_code(500_i128, "USD").unwrap();
    /// let (q, r) = a.divide_with_remainder(3).unwrap();
    /// assert_eq!(q.as_string(), "166 USD");
    /// assert_eq!(r.as_string(), "2 USD");
    /// // 166 * 3 + 2 == 500
    ///
    /// let neg = Money::from_code(-500_i128, "USD").unwrap();
    /// let (q, r) = neg.divide_with_remainder(3).unwrap();
    /// assert_eq!(q.as_string(), "-167 USD");
    /// assert_eq!(r.as_string(), "1 USD");
    /// // -167 * 3 + 1 == -500
    /// ```
    pub fn divide_with_remainder(
        &self,
        divisor: i128,
    ) -> Result<(Money, Money), MoneyError> {
        if divisor == 0 {
            return Err(MoneyError::DivisionByZero);
        }
        if self.amount == i128::MIN && divisor == -1 {
            // -i128::MIN overflows i128. Report it as overflow, not division.
            return Err(MoneyError::Overflow);
        }

        let q_trunc = self.amount / divisor;
        let r_trunc = self.amount % divisor;

        // Adjust toward floor when the truncated remainder's sign disagrees
        // with the divisor's sign.
        let (q, r) = if r_trunc != 0 && (r_trunc < 0) != (divisor < 0) {
            (q_trunc - 1, r_trunc + divisor)
        } else {
            (q_trunc, r_trunc)
        };

        Ok((
            Money::new(q, self.currency.clone()),
            Money::new(r, self.currency.clone()),
        ))
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

    // --- display -------------------------------------------------------------

    #[test]
    fn as_string_format() {
        assert_eq!(Money::new(500, usd()).as_string(), "500 USD");
        assert_eq!(Money::new(-5, usd()).as_string(), "-5 USD");
    }

    // --- equality ------------------------------------------------------------

    #[test]
    fn equality_basics() {
        assert_eq!(Money::new(500, usd()), Money::new(500, usd()));
        assert_ne!(Money::new(500, usd()), Money::new(501, usd()));
        assert_ne!(Money::new(500, usd()), Money::new(500, eur()));
    }

    // --- arithmetic ---------------------------------------------------------

    #[test]
    fn add_and_subtract() {
        let a = Money::new(500, usd());
        let b = Money::new(250, usd());
        assert_eq!(a.add(&b).unwrap().as_string(), "750 USD");
        assert_eq!(a.subtract(&b).unwrap().as_string(), "250 USD");
        assert_eq!(a.as_string(), "500 USD"); // unchanged
    }

    #[test]
    fn negate_and_abs() {
        let a = Money::new(500, usd());
        let b = Money::new(-500, usd());
        assert_eq!(a.negate().unwrap().as_string(), "-500 USD");
        assert_eq!(b.negate().unwrap().as_string(), "500 USD");
        assert_eq!(b.abs().unwrap().as_string(), "500 USD");
    }

    // --- comparison ---------------------------------------------------------

    #[test]
    fn compare_basics() {
        let a = Money::new(500, usd());
        let b = Money::new(250, usd());
        assert_eq!(a.compare(&b).unwrap(), Ordering::Greater);
        assert_eq!(b.compare(&a).unwrap(), Ordering::Less);
        assert_eq!(a.compare(&a).unwrap(), Ordering::Equal);
    }

    // --- multiply ------------------------------------------------------------

    #[test]
    fn multiply_by_int() {
        let a = Money::new(500, usd());
        assert_eq!(a.multiply(2).unwrap().as_string(), "1000 USD");
        assert_eq!(a.multiply(0).unwrap().as_string(), "0 USD");
        assert_eq!(a.multiply(-1).unwrap().as_string(), "-500 USD");
    }

    #[test]
    fn multiply_does_not_touch_receiver() {
        let a = Money::new(500, usd());
        let _ = a.multiply(3).unwrap();
        assert_eq!(a.as_string(), "500 USD");
    }

    #[test]
    fn multiply_overflow_errors() {
        let a = Money::new(i128::MAX, usd());
        assert!(matches!(a.multiply(2), Err(MoneyError::Overflow)));
    }

    #[test]
    fn multiply_min_by_minus_one_overflows() {
        let a = Money::new(i128::MIN, usd());
        assert!(matches!(a.multiply(-1), Err(MoneyError::Overflow)));
    }

    // --- divide --------------------------------------------------------------

    #[test]
    fn divide_exact() {
        let a = Money::new(500, usd());
        assert_eq!(a.divide(2).unwrap().as_string(), "250 USD");
        assert_eq!(a.divide(5).unwrap().as_string(), "100 USD");
        assert_eq!(a.divide(1).unwrap().as_string(), "500 USD");
    }

    #[test]
    fn divide_floors_positive() {
        let a = Money::new(500, usd());
        // 500 / 3 = 166.66 -> floor 166
        assert_eq!(a.divide(3).unwrap().as_string(), "166 USD");
    }

    #[test]
    fn divide_floors_negative() {
        let a = Money::new(-500, usd());
        // -500 / 3 = -166.66 -> floor -167
        assert_eq!(a.divide(3).unwrap().as_string(), "-167 USD");
    }

    #[test]
    fn divide_by_negative() {
        let a = Money::new(500, usd());
        // 500 / -3 = -166.66 -> floor -167
        assert_eq!(a.divide(-3).unwrap().as_string(), "-167 USD");
    }

    #[test]
    fn divide_negative_by_negative() {
        let a = Money::new(-500, usd());
        // -500 / -3 = 166.66 -> floor 166
        assert_eq!(a.divide(-3).unwrap().as_string(), "166 USD");
    }

    #[test]
    fn divide_by_zero_errors() {
        let a = Money::new(500, usd());
        assert!(matches!(a.divide(0), Err(MoneyError::DivisionByZero)));
    }

    #[test]
    fn divide_min_by_minus_one_overflows() {
        let a = Money::new(i128::MIN, usd());
        assert!(matches!(a.divide(-1), Err(MoneyError::Overflow)));
    }

    // --- divide_with_remainder ----------------------------------------------

    #[test]
    fn divide_with_remainder_positive() {
        let a = Money::new(500, usd());
        let (q, r) = a.divide_with_remainder(3).unwrap();
        assert_eq!(q.as_string(), "166 USD");
        assert_eq!(r.as_string(), "2 USD");
    }

    #[test]
    fn divide_with_remainder_exact() {
        let a = Money::new(500, usd());
        let (q, r) = a.divide_with_remainder(5).unwrap();
        assert_eq!(q.as_string(), "100 USD");
        assert_eq!(r.as_string(), "0 USD");
    }

    #[test]
    fn divide_with_remainder_negative_dividend() {
        let a = Money::new(-500, usd());
        let (q, r) = a.divide_with_remainder(3).unwrap();
        assert_eq!(q.as_string(), "-167 USD");
        assert_eq!(r.as_string(), "1 USD"); // sign of divisor
    }

    #[test]
    fn divide_with_remainder_negative_divisor() {
        let a = Money::new(500, usd());
        let (q, r) = a.divide_with_remainder(-3).unwrap();
        assert_eq!(q.as_string(), "-167 USD");
        assert_eq!(r.as_string(), "-1 USD"); // sign of divisor
    }

    #[test]
    fn divide_with_remainder_both_negative() {
        let a = Money::new(-500, usd());
        let (q, r) = a.divide_with_remainder(-3).unwrap();
        assert_eq!(q.as_string(), "166 USD");
        assert_eq!(r.as_string(), "-2 USD");
    }

    #[test]
    fn divide_with_remainder_invariant() {
        // q * d + r == original, for a range of dividend/divisor pairs.
        for dividend in [-1000_i128, -999, -1, 0, 1, 999, 1000] {
            for divisor in [-7_i128, -3, -1, 1, 3, 7] {
                let m = Money::new(dividend, usd());
                let (q, r) = m.divide_with_remainder(divisor).unwrap();
                let reconstructed = q.multiply(divisor).unwrap().add(&r).unwrap();
                assert_eq!(
                    reconstructed.amount(),
                    dividend,
                    "invariant failed for {dividend} / {divisor}: q={}, r={}",
                    q.amount(),
                    r.amount()
                );
                // remainder's sign matches divisor's (or zero)
                assert!(
                    r.amount() == 0 || (r.amount() < 0) == (divisor < 0),
                    "remainder sign for {dividend} / {divisor}: r={}",
                    r.amount()
                );
                // |r| < |divisor|
                assert!(
                    r.amount().unsigned_abs() < divisor.unsigned_abs(),
                    "|r| >= |divisor| for {dividend} / {divisor}: r={}",
                    r.amount()
                );
            }
        }
    }

    #[test]
    fn divide_with_remainder_by_zero_errors() {
        let a = Money::new(500, usd());
        assert!(matches!(
            a.divide_with_remainder(0),
            Err(MoneyError::DivisionByZero)
        ));
    }
}
