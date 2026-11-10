//! Error type for `hnoss`.

use std::fmt;

use crate::currency::CurrencyError;

/// Every error that can happen inside `hnoss`.
///
/// Some variants carry structured data so callers can match on them without
/// parsing error strings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MoneyError {
    /// A currency code was malformed.
    ///
    /// This wraps [`CurrencyError`] so callers can distinguish "bad code"
    /// from "unknown currency".
    InvalidCurrency(CurrencyError),

    /// An amount would exceed the representable range of `i128` minor units.
    ///
    /// `hnoss` uses `i128` for storage; amounts outside this range are
    /// rejected. In practice, `i128` covers every realistic monetary value
    /// (up to ~1.7×10³⁸ minor units).
    Overflow,

    /// An arithmetic operation was attempted between two different currencies.
    ///
    /// Carries both codes so the error message is actionable.
    CurrencyMismatch {
        /// The code on the left-hand side.
        left: String,
        /// The code on the right-hand side.
        right: String,
    },

    /// A non-integer value was passed where an integer was required.
    ///
    /// Currently unused (all `Money` amounts are `i128` by construction);
    /// reserved for a future API that accepts `&str` factors or divisors.
    NonInteger(String),

    /// A division by zero was attempted.
    DivisionByZero,

    /// A parsing error occurred.
    ///
    /// Carries a free-form message; a richer typed variant may come later.
    Parse(String),
}

impl fmt::Display for MoneyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MoneyError::InvalidCurrency(e) => write!(f, "invalid currency: {e}"),
            MoneyError::Overflow => {
                f.write_str("amount exceeds the representable range of i128")
            }
            MoneyError::CurrencyMismatch { left, right } => {
                write!(f, "currency mismatch: {left} vs {right}")
            }
            MoneyError::NonInteger(s) => write!(f, "expected an integer, got {s:?}"),
            MoneyError::DivisionByZero => f.write_str("division by zero"),
            MoneyError::Parse(s) => write!(f, "parse error: {s}"),
        }
    }
}

impl std::error::Error for MoneyError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            MoneyError::InvalidCurrency(e) => Some(e),
            _ => None,
        }
    }
}

impl From<CurrencyError> for MoneyError {
    fn from(e: CurrencyError) -> Self {
        MoneyError::InvalidCurrency(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn display_invalid_currency() {
        let e = MoneyError::InvalidCurrency(CurrencyError::InvalidCode("XX".into()));
        let s = format!("{e}");
        assert!(s.contains("invalid currency"));
        assert!(s.contains("XX"));
    }

    #[test]
    fn display_overflow() {
        assert_eq!(
            format!("{}", MoneyError::Overflow),
            "amount exceeds the representable range of i128"
        );
    }

    #[test]
    fn display_currency_mismatch() {
        let e = MoneyError::CurrencyMismatch {
            left: "USD".into(),
            right: "EUR".into(),
        };
        assert_eq!(format!("{e}"), "currency mismatch: USD vs EUR");
    }

    #[test]
    fn display_division_by_zero() {
        assert_eq!(
            format!("{}", MoneyError::DivisionByZero),
            "division by zero"
        );
    }

    #[test]
    fn source_chains_for_invalid_currency() {
        let e = MoneyError::InvalidCurrency(CurrencyError::InvalidCode("X".into()));
        assert!(e.source().is_some());
    }

    #[test]
    fn source_none_for_other_variants() {
        assert!(MoneyError::Overflow.source().is_none());
        assert!(MoneyError::DivisionByZero.source().is_none());
    }

    #[test]
    fn from_currency_error() {
        let ce = CurrencyError::InvalidCode("X".into());
        let me: MoneyError = ce.into();
        assert!(matches!(me, MoneyError::InvalidCurrency(_)));
    }
}
