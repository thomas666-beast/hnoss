//! Integration tests for `hnoss::Money`.
//!
//! These tests exercise only the public API. They are what a downstream
//! user would write.

use hnoss::{Currency, Money, MoneyError};

fn usd() -> Currency {
    Currency::new("USD").unwrap()
}

#[test]
fn constructs_from_minor_and_currency() {
    let m = Money::new(500, usd());
    assert_eq!(m.amount(), 500);
    assert_eq!(m.currency().code(), "USD");
}

#[test]
fn constructs_from_code() {
    let m = Money::from_code(500, "USD").unwrap();
    assert_eq!(m.amount(), 500);
    assert_eq!(m.currency().code(), "USD");
}

#[test]
fn from_code_accepts_lowercase() {
    let m = Money::from_code(500, "usd").unwrap();
    assert_eq!(m.currency().code(), "USD");
}

#[test]
fn from_code_rejects_bad_input() {
    for bad in ["", "U", "US", "U5D", "U D", "TOOLONGXX"] {
        let err = Money::from_code(500, bad).unwrap_err();
        assert!(
            matches!(err, MoneyError::InvalidCurrency(_)),
            "{bad:?} => {err:?}"
        );
    }
}

#[test]
fn as_string_is_canonical() {
    assert_eq!(Money::new(500, usd()).as_string(), "500 USD");
    assert_eq!(Money::new(-5, usd()).as_string(), "-5 USD");
    assert_eq!(Money::new(0, usd()).as_string(), "0 USD");
    assert_eq!(
        Money::new(i128::MAX, usd()).as_string(),
        format!("{} USD", i128::MAX)
    );
    assert_eq!(
        Money::new(i128::MIN, usd()).as_string(),
        format!("{} USD", i128::MIN)
    );
}

#[test]
fn display_matches_as_string() {
    let m = Money::new(500, usd());
    assert_eq!(format!("{m}"), m.as_string());
}

#[test]
fn equality_same_currency_same_amount() {
    assert_eq!(
        Money::new(500, usd()),
        Money::new(500, Currency::new("usd").unwrap())
    );
}

#[test]
fn equality_different_amount() {
    assert_ne!(Money::new(500, usd()), Money::new(501, usd()));
}

#[test]
fn equality_different_currency() {
    assert_ne!(
        Money::new(500, usd()),
        Money::new(500, Currency::new("EUR").unwrap())
    );
}

#[test]
fn clone_is_independent() {
    let a = Money::new(500, usd());
    let b = a.clone();
    assert_eq!(a, b);
    // `a` and `b` are separate values; mutating one (if it were mutable)
    // would not affect the other. Since `Money` has no public mutators,
    // "independent" here just means the clone is a distinct value that
    // compares equal to the original.
}

#[test]
fn debug_includes_currency_and_amount() {
    let m = Money::new(500, usd());
    let s = format!("{m:?}");
    assert!(s.contains("500"), "{s}");
    assert!(s.contains("USD"), "{s}");
}

#[test]
fn works_with_btc_trx_usdt() {
    for code in ["BTC", "TRX", "USDT"] {
        let m = Money::from_code(500, code).unwrap();
        assert_eq!(m.currency().code(), code);
        assert_eq!(m.amount(), 500);
    }
}

#[test]
fn hashable_for_use_in_maps() {
    use std::collections::HashMap;
    let mut map = HashMap::new();
    map.insert(Money::new(500, usd()), "five dollars");
    assert_eq!(map.get(&Money::new(500, usd())), Some(&"five dollars"));
    assert_eq!(
        map.get(&Money::new(500, Currency::new("EUR").unwrap())),
        None
    );
}
