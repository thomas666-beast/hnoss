//! Currency value object.

use std::fmt;

use crate::repository::Repository;

/// Errors that can occur when constructing a [`Currency`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CurrencyError {
    /// The code is not between 3 and 8 ASCII letters.
    InvalidCode(String),
    /// The repository rejected the code (currently unused; reserved for a
    /// future strict mode where unknown codes are rejected).
    Unknown(String),
}

impl fmt::Display for CurrencyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CurrencyError::InvalidCode(code) => write!(
                f,
                "invalid currency code {code:?}: must be 3-8 ASCII letters"
            ),
            CurrencyError::Unknown(code) => {
                write!(f, "unknown currency code {code:?}")
            }
        }
    }
}

impl std::error::Error for CurrencyError {}

/// An immutable currency: a 3-to-8-letter code plus its exponent.
///
/// # Example
///
/// ```
/// use hnoss::Currency;
///
/// let usd = Currency::new("USD").unwrap();
/// assert_eq!(usd.code(), "USD");
/// assert_eq!(usd.exponent(), 2);
///
/// let jpy = Currency::new("JPY").unwrap();
/// assert_eq!(jpy.exponent(), 0);
///
/// let btc = Currency::new("BTC").unwrap();
/// assert_eq!(btc.exponent(), 8);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Currency {
    code: String,
}

impl Currency {
    /// Constructs a new `Currency` from a code string.
    ///
    /// The code must be 3 to 8 ASCII letters. Case is normalized to
    /// uppercase.
    ///
    /// # Errors
    ///
    /// Returns [`CurrencyError::InvalidCode`] if the code is not 3 to 8
    /// ASCII letters.
    pub fn new(code: impl AsRef<str>) -> Result<Self, CurrencyError> {
        let code = code.as_ref();
        if !(3..=8).contains(&code.len())
            || !code.bytes().all(|b| b.is_ascii_alphabetic())
        {
            return Err(CurrencyError::InvalidCode(code.to_string()));
        }
        Ok(Currency {
            code: code.to_ascii_uppercase(),
        })
    }

    /// Returns the normalized (uppercase) code.
    pub fn code(&self) -> &str {
        &self.code
    }

    /// Returns the number of decimal digits between the major unit and the
    /// smallest storable unit for this currency.
    ///
    /// For unknown codes, returns the repository's fallback exponent (2 for
    /// the default repository).
    pub fn exponent(&self) -> u8 {
        Repository::default().exponent(&self.code)
    }

    /// Alias for [`Currency::exponent`].
    pub fn subunit(&self) -> u8 {
        self.exponent()
    }
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructs_uppercase_code() {
        let c = Currency::new("usd").unwrap();
        assert_eq!(c.code(), "USD");
    }

    #[test]
    fn normalizes_mixed_case() {
        let c = Currency::new("Eur").unwrap();
        assert_eq!(c.code(), "EUR");
    }

    #[test]
    fn accepts_3_to_8_letters() {
        for code in ["USD", "USDT", "WBTC", "SHIB", "TOOLONGX"] {
            assert!(Currency::new(code).is_ok(), "expected {code:?} to be valid");
        }
    }

    #[test]
    fn rejects_too_short() {
        assert!(matches!(
            Currency::new(""),
            Err(CurrencyError::InvalidCode(_))
        ));
        assert!(matches!(
            Currency::new("U"),
            Err(CurrencyError::InvalidCode(_))
        ));
        assert!(matches!(
            Currency::new("US"),
            Err(CurrencyError::InvalidCode(_))
        ));
    }

    #[test]
    fn rejects_too_long() {
        assert!(matches!(
            Currency::new("TOOLONGXX"),
            Err(CurrencyError::InvalidCode(_))
        ));
    }

    #[test]
    fn rejects_non_letters() {
        for bad in ["U5D", "U D", "US1", "USD!", "USD."] {
            assert!(
                Currency::new(bad).is_err(),
                "expected {bad:?} to be rejected"
            );
        }
    }

    #[test]
    fn exponent_known_currencies() {
        assert_eq!(Currency::new("USD").unwrap().exponent(), 2);
        assert_eq!(Currency::new("JPY").unwrap().exponent(), 0);
        assert_eq!(Currency::new("BHD").unwrap().exponent(), 3);
        assert_eq!(Currency::new("UYW").unwrap().exponent(), 4);
        assert_eq!(Currency::new("BTC").unwrap().exponent(), 8);
        assert_eq!(Currency::new("TRX").unwrap().exponent(), 6);
        assert_eq!(Currency::new("USDT").unwrap().exponent(), 6);
    }

    #[test]
    fn exponent_unknown_falls_back() {
        assert_eq!(Currency::new("ZZZ").unwrap().exponent(), 2);
    }

    #[test]
    fn display_is_code() {
        let c = Currency::new("usd").unwrap();
        assert_eq!(format!("{c}"), "USD");
    }

    #[test]
    fn equality_is_code_based() {
        let a = Currency::new("USD").unwrap();
        let b = Currency::new("usd").unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn can_be_used_as_hash_key() {
        use std::collections::HashMap;
        let mut m = HashMap::new();
        m.insert(Currency::new("USD").unwrap(), 1);
        assert_eq!(m.get(&Currency::new("usd").unwrap()), Some(&1));
    }
}
