//! Error type for `hnoss`.

use std::fmt;

use crate::currency::CurrencyError;

/// Every error that can happen inside `hnoss`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MoneyError {
    /// A currency code was malformed.
    InvalidCurrency(CurrencyError),

    /// An amount would exceed the representable range of `i128` minor units.
    Overflow,

    /// An arithmetic operation was attempted between two different currencies.
    CurrencyMismatch {
        /// The code on the left-hand side.
        left: String,
        /// The code on the right-hand side.
        right: String,
    },

    /// A non-integer value was passed where an integer was required.
    ///
    /// Currently unused; reserved for a future API that accepts `&str` factors.
    NonInteger(String),

    /// A division by zero was attempted.
    DivisionByZero,

    /// An allocation (split) was requested with invalid inputs.
    ///
    /// The carried string explains the specific issue (e.g. empty ratio
    /// list, or all ratios zero).
    InvalidAllocation(String),

    /// A parsing error occurred.
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
            MoneyError::InvalidAllocation(s) => {
                write!(f, "invalid allocation: {s}")
            }
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
    fn display_invalid_allocation() {
        let e = MoneyError::InvalidAllocation("no ratios".into());
        assert_eq!(format!("{e}"), "invalid allocation: no ratios");
    }

    #[test]
    fn display_parse() {
        let e = MoneyError::Parse("unexpected character".into());
        assert_eq!(format!("{e}"), "parse error: unexpected character");
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
        assert!(MoneyError::InvalidAllocation("x".into()).source().is_none());
    }

    #[test]
    fn from_currency_error() {
        let ce = CurrencyError::InvalidCode("X".into());
        let me: MoneyError = ce.into();
        assert!(matches!(me, MoneyError::InvalidCurrency(_)));
    }
}
