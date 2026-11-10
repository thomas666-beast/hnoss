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
///
/// * `format` and `format_with` render the amount using the currency's
///   exponent. `format` uses default options and cannot fail; `format_with`
///   accepts custom options and fails on invalid `group_size`.
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

    /// Formats the amount in a human-readable form, using the currency's
    /// exponent (2 for USD, 0 for JPY, 3 for BHD, 8 for BTC, ...) and the
    /// default [`FormatOptions`](crate::FormatOptions).
    ///
    /// This method never fails, because the default options are always
    /// valid. For control over separators, grouping, or currency
    /// inclusion, use [`Money::format_with`].
    ///
    /// # Example
    ///
    /// ```
    /// use hnoss::Money;
    ///
    /// assert_eq!(
    ///     Money::from_code(500_i128, "USD").unwrap().format(),
    ///     "5.00 USD"
    /// );
    /// assert_eq!(
    ///     Money::from_code(500_i128, "JPY").unwrap().format(),
    ///     "500 JPY"
    /// );
    /// assert_eq!(
    ///     Money::from_code(500_i128, "BHD").unwrap().format(),
    ///     "0.500 BHD"
    /// );
    /// ```
    pub fn format(&self) -> String {
        crate::format::render_options(
            self.amount,
            &self.currency,
            &crate::FormatOptions::default(),
        )
        .expect("default FormatOptions are always valid")
    }

    /// Formats the amount using custom [`FormatOptions`](crate::FormatOptions).
    ///
    /// # Errors
    ///
    /// Returns [`MoneyError::InvalidAllocation`] if `opts.group_size == 0`.
    /// (The variant is reused from allocation because both are "invalid
    /// caller input" conditions; a dedicated `InvalidFormat` variant can be
    /// added later if the need to distinguish them arises.)
    ///
    /// # Example
    ///
    /// ```
    /// use hnoss::{FormatOptions, Money};
    ///
    /// let m = Money::from_code(123456_i128, "EUR").unwrap();
    /// let opts = FormatOptions {
    ///     decimal_mark: ',',
    ///     thousands_separator: ".".into(),
    ///     ..Default::default()
    /// };
    /// assert_eq!(m.format_with(&opts).unwrap(), "1.234,56 EUR");
    /// ```
    pub fn format_with(&self, opts: &crate::FormatOptions) -> Result<String, MoneyError> {
        crate::format::render_options(self.amount, &self.currency, opts)
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

    /// Splits `self` across N recipients according to the given ratios.
    ///
    /// Returns a `Vec<Money>` of length `ratios.len()`. The sum of the
    /// returned values is exactly `self` — no minor units are lost or
    /// created.
    ///
    /// Each share is `floor(|amount| * ratio_i / sum_of_ratios)`, and the
    /// leftover minor units are handed out one at a time, starting at index
    /// 0. The distribution is **mirror-symmetric**: for a negative amount,
    /// the same indices that would receive an extra unit for the positive
    /// amount instead pay an extra unit. Concretely:
    ///
    /// ```text
    /// Money::from_code(-100, "USD").allocate(&[1, 1, 1])
    /// ```
    ///
    /// is element-wise the negation of
    ///
    /// ```text
    /// Money::from_code( 100, "USD").allocate(&[1, 1, 1])
    /// ```
    ///
    /// # Errors
    ///
    /// * [`MoneyError::InvalidAllocation`] if `ratios` is empty.
    /// * [`MoneyError::InvalidAllocation`] if all ratios are zero.
    /// * [`MoneyError::Overflow`] if `sum_of_ratios` overflows `u128`, or if
    ///   an intermediate `amount * ratio` would overflow `i128`.
    ///
    /// # Example
    ///
    /// ```
    /// use hnoss::Money;
    ///
    /// let hundred = Money::from_code(100_i128, "USD").unwrap();
    /// let parts = hundred.allocate(&[1, 1, 1]).unwrap();
    /// assert_eq!(parts.len(), 3);
    /// assert_eq!(parts[0].as_string(), "34 USD");
    /// assert_eq!(parts[1].as_string(), "33 USD");
    /// assert_eq!(parts[2].as_string(), "33 USD");
    ///
    /// let total = parts
    ///     .iter()
    ///     .fold(Money::from_code(0_i128, "USD").unwrap(), |a, b| {
    ///         a.add(b).unwrap()
    ///     });
    /// assert_eq!(total.as_string(), "100 USD");
    /// ```
    pub fn allocate(&self, ratios: &[u128]) -> Result<Vec<Money>, MoneyError> {
        if ratios.is_empty() {
            return Err(MoneyError::InvalidAllocation("no ratios provided".into()));
        }

        // Sum the ratios, rejecting overflow and all-zero.
        let mut sum: u128 = 0;
        for &r in ratios {
            sum = sum.checked_add(r).ok_or(MoneyError::Overflow)?;
        }
        if sum == 0 {
            return Err(MoneyError::InvalidAllocation("all ratios are zero".into()));
        }

        // Compute base shares on the absolute amount, then negate at the end
        // if the original was negative. This makes the result mirror-symmetric.
        let is_negative = self.amount < 0;
        let abs_amount: u128 = self.amount.unsigned_abs();

        let mut shares: Vec<i128> = Vec::with_capacity(ratios.len());
        let mut allocated: i128 = 0;
        for &r in ratios {
            // numerator = abs_amount * r, must fit in i128 for the division
            // to produce a quotient that fits in i128.
            let numerator = abs_amount.checked_mul(r).ok_or(MoneyError::Overflow)?;
            let q_u128 = numerator / sum; // sum > 0, floor division is exact
            let q: i128 = i128::try_from(q_u128).map_err(|_| MoneyError::Overflow)?;
            shares.push(q);
            allocated = allocated.checked_add(q).ok_or(MoneyError::Overflow)?;
        }

        // Remainder: non-negative by construction (floor division of
        // non-negative values). Hand out one unit at a time, starting at
        // index 0.
        let abs_total: i128 =
            i128::try_from(abs_amount).map_err(|_| MoneyError::Overflow)?;
        let mut rem = abs_total - allocated;
        let n = shares.len();
        let mut i = 0;
        while rem > 0 {
            shares[i] = shares[i].checked_add(1).ok_or(MoneyError::Overflow)?;
            rem -= 1;
            i = (i + 1) % n;
        }

        if is_negative {
            for s in &mut shares {
                *s = s.checked_neg().ok_or(MoneyError::Overflow)?;
            }
        }

        Ok(shares
            .into_iter()
            .map(|a| Money::new(a, self.currency.clone()))
            .collect())
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

    // --- allocate -----------------------------------------------------------

    #[test]
    fn allocate_three_way() {
        let a = Money::new(100, usd());
        let parts = a.allocate(&[1, 1, 1]).unwrap();
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0].as_string(), "34 USD");
        assert_eq!(parts[1].as_string(), "33 USD");
        assert_eq!(parts[2].as_string(), "33 USD");
    }

    #[test]
    fn allocate_by_percentages() {
        let a = Money::new(100, usd());
        let parts = a.allocate(&[70, 20, 10]).unwrap();
        assert_eq!(parts[0].as_string(), "70 USD");
        assert_eq!(parts[1].as_string(), "20 USD");
        assert_eq!(parts[2].as_string(), "10 USD");
    }

    #[test]
    fn allocate_two_way_even() {
        let a = Money::new(100, usd());
        let parts = a.allocate(&[1, 1]).unwrap();
        assert_eq!(parts[0].as_string(), "50 USD");
        assert_eq!(parts[1].as_string(), "50 USD");
    }

    #[test]
    fn allocate_two_way_odd() {
        let a = Money::new(101, usd());
        let parts = a.allocate(&[1, 1]).unwrap();
        assert_eq!(parts[0].as_string(), "51 USD");
        assert_eq!(parts[1].as_string(), "50 USD");
    }

    #[test]
    fn allocate_with_zero_ratios() {
        let a = Money::new(100, usd());
        let parts = a.allocate(&[0, 1]).unwrap();
        assert_eq!(parts[0].as_string(), "0 USD");
        assert_eq!(parts[1].as_string(), "100 USD");
    }

    #[test]
    fn allocate_negative_amount() {
        let a = Money::new(-100, usd());
        let parts = a.allocate(&[1, 1, 1]).unwrap();
        assert_eq!(parts[0].as_string(), "-34 USD");
        assert_eq!(parts[1].as_string(), "-33 USD");
        assert_eq!(parts[2].as_string(), "-33 USD");
    }

    #[test]
    fn allocate_negative_odd() {
        let a = Money::new(-101, usd());
        let parts = a.allocate(&[1, 1, 1]).unwrap();
        assert_eq!(parts[0].as_string(), "-34 USD");
        assert_eq!(parts[1].as_string(), "-34 USD");
        assert_eq!(parts[2].as_string(), "-33 USD");
    }

    #[test]
    fn allocate_mirror_symmetric() {
        // allocate(-x) == -allocate(x), element-wise.
        for amount in [1_i128, 7, 100, 101, 999, 12345] {
            for ratios in [
                vec![1u128, 1],
                vec![1, 1, 1],
                vec![70, 20, 10],
                vec![1, 2, 3, 4],
            ] {
                let pos = Money::new(amount, usd()).allocate(&ratios).unwrap();
                let neg = Money::new(-amount, usd()).allocate(&ratios).unwrap();
                for (p, n) in pos.iter().zip(neg.iter()) {
                    assert_eq!(
                        p.amount(),
                        -n.amount(),
                        "mirror failed for amount={amount} ratios={ratios:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn allocate_sum_equals_original() {
        for amount in [-999_i128, -100, -1, 0, 1, 100, 999] {
            for ratios in [vec![1u128], vec![1, 1], vec![1, 1, 1], vec![7, 11, 13, 17]] {
                let m = Money::new(amount, usd());
                let parts = m.allocate(&ratios).unwrap();
                let sum = parts
                    .iter()
                    .fold(Money::new(0, usd()), |acc, p| acc.add(p).unwrap());
                assert_eq!(
                    sum.amount(),
                    amount,
                    "sum invariant failed for amount={amount} ratios={ratios:?}"
                );
            }
        }
    }

    #[test]
    fn allocate_preserves_currency() {
        let a = Money::new(100, eur());
        let parts = a.allocate(&[1, 1]).unwrap();
        assert_eq!(parts[0].currency().code(), "EUR");
        assert_eq!(parts[1].currency().code(), "EUR");
    }

    #[test]
    fn allocate_zero_amount() {
        let a = Money::new(0, usd());
        let parts = a.allocate(&[1, 1, 1]).unwrap();
        assert_eq!(parts[0].as_string(), "0 USD");
        assert_eq!(parts[1].as_string(), "0 USD");
        assert_eq!(parts[2].as_string(), "0 USD");
    }

    // --- allocate errors ----------------------------------------------------

    #[test]
    fn allocate_empty_ratios_errors() {
        let a = Money::new(100, usd());
        match a.allocate(&[]).unwrap_err() {
            MoneyError::InvalidAllocation(msg) => assert!(msg.contains("no ratios")),
            other => panic!("expected InvalidAllocation, got {other:?}"),
        }
    }

    #[test]
    fn allocate_all_zero_errors() {
        let a = Money::new(100, usd());
        match a.allocate(&[0, 0, 0]).unwrap_err() {
            MoneyError::InvalidAllocation(msg) => {
                assert!(msg.contains("all ratios are zero"))
            }
            other => panic!("expected InvalidAllocation, got {other:?}"),
        }
    }

    #[test]
    fn allocate_ratio_sum_overflow_errors() {
        let a = Money::new(100, usd());
        assert!(matches!(
            a.allocate(&[u128::MAX, 1]),
            Err(MoneyError::Overflow)
        ));
    }

    #[test]
    fn allocate_does_not_touch_receiver() {
        let a = Money::new(100, usd());
        let _ = a.allocate(&[1, 1, 1]).unwrap();
        assert_eq!(a.as_string(), "100 USD");
    }

    // --- format -------------------------------------------------------------

    #[test]
    fn format_default_usd() {
        assert_eq!(Money::new(500, usd()).format(), "5.00 USD");
        assert_eq!(Money::new(0, usd()).format(), "0.00 USD");
        assert_eq!(Money::new(-5, usd()).format(), "-0.05 USD");
        assert_eq!(Money::new(123456, usd()).format(), "1,234.56 USD");
    }

    #[test]
    fn format_default_jpy() {
        let jpy = Currency::new("JPY").unwrap();
        assert_eq!(Money::new(500, jpy.clone()).format(), "500 JPY");
        assert_eq!(Money::new(-500, jpy).format(), "-500 JPY");
    }

    #[test]
    fn format_default_bhd() {
        let bhd = Currency::new("BHD").unwrap();
        assert_eq!(Money::new(500, bhd).format(), "0.500 BHD");
    }

    #[test]
    fn format_default_btc() {
        let btc = Currency::new("BTC").unwrap();
        assert_eq!(Money::new(500, btc).format(), "0.00000500 BTC");
        assert_eq!(
            Money::new(100_000_000, Currency::new("BTC").unwrap()).format(),
            "1.00000000 BTC"
        );
    }

    #[test]
    fn format_default_usdt() {
        let usdt = Currency::new("USDT").unwrap();
        assert_eq!(Money::new(1_000_000, usdt).format(), "1.000000 USDT");
    }

    #[test]
    fn format_with_custom_options() {
        let m = Money::new(123456, usd());
        let opts = crate::FormatOptions {
            decimal_mark: ',',
            thousands_separator: ".".into(),
            ..Default::default()
        };
        assert_eq!(m.format_with(&opts).unwrap(), "1.234,56 USD");
    }

    #[test]
    fn format_with_no_currency() {
        let m = Money::new(500, usd());
        let opts = crate::FormatOptions {
            with_currency: false,
            ..Default::default()
        };
        assert_eq!(m.format_with(&opts).unwrap(), "5.00");
    }

    #[test]
    fn format_with_zero_group_size_errors() {
        let m = Money::new(500, usd());
        let opts = crate::FormatOptions {
            group_size: 0,
            ..Default::default()
        };
        assert!(matches!(
            m.format_with(&opts),
            Err(MoneyError::InvalidAllocation(_))
        ));
    }

    #[test]
    fn format_does_not_touch_receiver() {
        let m = Money::new(500, usd());
        let _ = m.format();
        assert_eq!(m.amount(), 500);
    }

    #[test]
    fn format_differs_from_as_string() {
        // as_string is canonical; format is display.
        let m = Money::new(500, usd());
        assert_eq!(m.as_string(), "500 USD");
        assert_eq!(m.format(), "5.00 USD");
    }
}
