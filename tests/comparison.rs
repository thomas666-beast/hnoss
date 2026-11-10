//! Integration tests for comparison on `hnoss::Money`.

use std::cmp::Ordering;

use hnoss::{Money, MoneyError};

// --- compare -----------------------------------------------------------------

#[test]
fn compare_returns_ordering() {
    let a = Money::from_code(500, "USD").unwrap();
    let b = Money::from_code(250, "USD").unwrap();
    let c = Money::from_code(500, "USD").unwrap();

    assert_eq!(a.compare(&b).unwrap(), Ordering::Greater);
    assert_eq!(b.compare(&a).unwrap(), Ordering::Less);
    assert_eq!(a.compare(&c).unwrap(), Ordering::Equal);
}

#[test]
fn compare_is_transitive() {
    let a = Money::from_code(1, "USD").unwrap();
    let b = Money::from_code(2, "USD").unwrap();
    let c = Money::from_code(3, "USD").unwrap();

    assert_eq!(a.compare(&b).unwrap(), Ordering::Less);
    assert_eq!(b.compare(&c).unwrap(), Ordering::Less);
    assert_eq!(a.compare(&c).unwrap(), Ordering::Less);
}

#[test]
fn compare_across_currencies_errors() {
    let a = Money::from_code(500, "USD").unwrap();
    let b = Money::from_code(500, "EUR").unwrap();
    match a.compare(&b).unwrap_err() {
        MoneyError::CurrencyMismatch { left, right } => {
            assert_eq!(left, "USD");
            assert_eq!(right, "EUR");
        }
        other => panic!("expected CurrencyMismatch, got {other:?}"),
    }
}

#[test]
fn compare_extreme_amounts() {
    let min = Money::new(
        i128::MIN,
        Money::from_code(0, "USD").unwrap().currency().clone(),
    );
    let max = Money::new(
        i128::MAX,
        Money::from_code(0, "USD").unwrap().currency().clone(),
    );
    let zero = Money::from_code(0, "USD").unwrap();

    assert_eq!(min.compare(&zero).unwrap(), Ordering::Less);
    assert_eq!(zero.compare(&max).unwrap(), Ordering::Less);
    assert_eq!(min.compare(&max).unwrap(), Ordering::Less);
}

// --- equals ------------------------------------------------------------------

#[test]
fn equals_same() {
    let a = Money::from_code(500, "USD").unwrap();
    let b = Money::from_code(500, "USD").unwrap();
    assert!(a.equals(&b));
}

#[test]
fn equals_different_amount() {
    let a = Money::from_code(500, "USD").unwrap();
    let b = Money::from_code(501, "USD").unwrap();
    assert!(!a.equals(&b));
}

#[test]
fn equals_different_currency_is_false_not_error() {
    let a = Money::from_code(500, "USD").unwrap();
    let b = Money::from_code(500, "EUR").unwrap();
    assert!(!a.equals(&b));
}

#[test]
fn equals_agrees_with_eq_operator() {
    let cases = [
        (
            Money::from_code(500, "USD").unwrap(),
            Money::from_code(500, "USD").unwrap(),
        ),
        (
            Money::from_code(500, "USD").unwrap(),
            Money::from_code(500, "usd").unwrap(),
        ),
        (
            Money::from_code(500, "USD").unwrap(),
            Money::from_code(501, "USD").unwrap(),
        ),
        (
            Money::from_code(500, "USD").unwrap(),
            Money::from_code(500, "EUR").unwrap(),
        ),
        (
            Money::from_code(0, "USD").unwrap(),
            Money::from_code(0, "USD").unwrap(),
        ),
    ];
    for (a, b) in cases {
        assert_eq!(a == b, a.equals(&b), "disagreement for {a} vs {b}");
    }
}

// --- predicates --------------------------------------------------------------

#[test]
fn is_zero_basics() {
    assert!(Money::from_code(0, "USD").unwrap().is_zero());
    assert!(!Money::from_code(1, "USD").unwrap().is_zero());
    assert!(!Money::from_code(-1, "USD").unwrap().is_zero());
}

#[test]
fn is_positive_excludes_zero() {
    assert!(Money::from_code(1, "USD").unwrap().is_positive());
    assert!(!Money::from_code(0, "USD").unwrap().is_positive());
    assert!(!Money::from_code(-1, "USD").unwrap().is_positive());
}

#[test]
fn is_negative_basics() {
    assert!(Money::from_code(-1, "USD").unwrap().is_negative());
    assert!(!Money::from_code(0, "USD").unwrap().is_negative());
    assert!(!Money::from_code(1, "USD").unwrap().is_negative());
}

#[test]
fn predicates_partition_the_amount_space() {
    for amount in [-1_000_000_i128, -1, 0, 1, 1_000_000] {
        let m = Money::new(
            amount,
            Money::from_code(0, "USD").unwrap().currency().clone(),
        );
        let hits = m.is_zero() as u8 + m.is_positive() as u8 + m.is_negative() as u8;
        assert_eq!(
            hits, 1,
            "amount {amount} should match exactly one predicate"
        );
    }
}
