//! Integration test for `hnoss::Currency`.
//!
//! These tests use only the public API. Unit tests inside `src/` cover
//! implementation details; this file is what a downstream user can run.

use hnoss::{Currency, CurrencyError};

#[test]
fn constructs_and_normalizes() {
    let c = Currency::new("usd").unwrap();
    assert_eq!(c.code(), "USD");
    assert_eq!(format!("{c}"), "USD");
}

#[test]
fn accepts_longer_codes() {
    for code in ["USD", "USDT", "WBTC", "SHIB", "TOOLONGX"] {
        assert!(Currency::new(code).is_ok(), "{code}");
    }
}

#[test]
fn rejects_malformed() {
    for bad in ["", "U", "US", "TOOLONGXX", "U5D", "U D"] {
        assert!(
            matches!(Currency::new(bad), Err(CurrencyError::InvalidCode(_))),
            "{bad:?}"
        );
    }
}

#[test]
fn exponent_lookup() {
    assert_eq!(Currency::new("USD").unwrap().exponent(), 2);
    assert_eq!(Currency::new("JPY").unwrap().exponent(), 0);
    assert_eq!(Currency::new("BHD").unwrap().exponent(), 3);
    assert_eq!(Currency::new("BTC").unwrap().exponent(), 8);
    assert_eq!(Currency::new("TRX").unwrap().exponent(), 6);
    assert_eq!(Currency::new("USDT").unwrap().exponent(), 6);
    assert_eq!(Currency::new("ZZZ").unwrap().exponent(), 2);
}

#[test]
fn subunit_is_alias_for_exponent() {
    let c = Currency::new("USD").unwrap();
    assert_eq!(c.exponent(), c.subunit());
}

#[test]
fn equality_is_code_based() {
    assert_eq!(Currency::new("USD").unwrap(), Currency::new("usd").unwrap());
    assert_ne!(Currency::new("USD").unwrap(), Currency::new("EUR").unwrap());
}
