//! Integration tests for arithmetic on `hnoss::Money`.

use hnoss::{Money, MoneyError};

fn usd() -> Money {
    Money::from_code(0, "USD").unwrap()
}

fn eur() -> Money {
    Money::from_code(0, "EUR").unwrap()
}

// --- add ---------------------------------------------------------------------

#[test]
fn add_basics() {
    let a = Money::from_code(500, "USD").unwrap();
    let b = Money::from_code(250, "USD").unwrap();
    assert_eq!(a.add(&b).unwrap().as_string(), "750 USD");
}

#[test]
fn add_with_zero() {
    let a = Money::from_code(500, "USD").unwrap();
    let zero = Money::from_code(0, "USD").unwrap();
    assert_eq!(a.add(&zero).unwrap(), a);
}

#[test]
fn add_with_negative() {
    let a = Money::from_code(500, "USD").unwrap();
    let b = Money::from_code(-200, "USD").unwrap();
    assert_eq!(a.add(&b).unwrap().as_string(), "300 USD");
}

#[test]
fn add_across_currencies_errors() {
    let a = Money::from_code(500, "USD").unwrap();
    let b = Money::from_code(500, "EUR").unwrap();
    match a.add(&b).unwrap_err() {
        MoneyError::CurrencyMismatch { left, right } => {
            assert_eq!(left, "USD");
            assert_eq!(right, "EUR");
        }
        other => panic!("expected CurrencyMismatch, got {other:?}"),
    }
}

#[test]
fn add_overflow_errors() {
    let a = Money::new(i128::MAX, usd().currency().clone());
    let b = Money::new(1, usd().currency().clone());
    assert!(matches!(a.add(&b), Err(MoneyError::Overflow)));
}

// --- subtract ----------------------------------------------------------------

#[test]
fn subtract_basics() {
    let a = Money::from_code(500, "USD").unwrap();
    let b = Money::from_code(250, "USD").unwrap();
    assert_eq!(a.subtract(&b).unwrap().as_string(), "250 USD");
}

#[test]
fn subtract_to_negative() {
    let a = Money::from_code(250, "USD").unwrap();
    let b = Money::from_code(500, "USD").unwrap();
    assert_eq!(a.subtract(&b).unwrap().as_string(), "-250 USD");
}

#[test]
fn subtract_across_currencies_errors() {
    let a = Money::from_code(500, "USD").unwrap();
    let b = Money::from_code(500, "EUR").unwrap();
    assert!(matches!(
        a.subtract(&b),
        Err(MoneyError::CurrencyMismatch { .. })
    ));
}

#[test]
fn subtract_overflow_errors() {
    let a = Money::new(i128::MIN, eur().currency().clone());
    let b = Money::new(1, eur().currency().clone());
    assert!(matches!(a.subtract(&b), Err(MoneyError::Overflow)));
}

// --- negate ------------------------------------------------------------------

#[test]
fn negate_basics() {
    assert_eq!(
        Money::from_code(500, "USD")
            .unwrap()
            .negate()
            .unwrap()
            .as_string(),
        "-500 USD"
    );
    assert_eq!(
        Money::from_code(-500, "USD")
            .unwrap()
            .negate()
            .unwrap()
            .as_string(),
        "500 USD"
    );
    assert_eq!(
        Money::from_code(0, "USD")
            .unwrap()
            .negate()
            .unwrap()
            .as_string(),
        "0 USD"
    );
}

#[test]
fn negate_extreme_errors() {
    let a = Money::new(i128::MIN, usd().currency().clone());
    assert!(matches!(a.negate(), Err(MoneyError::Overflow)));
}

// --- abs ---------------------------------------------------------------------

#[test]
fn abs_basics() {
    assert_eq!(
        Money::from_code(-500, "USD")
            .unwrap()
            .abs()
            .unwrap()
            .as_string(),
        "500 USD"
    );
    assert_eq!(
        Money::from_code(500, "USD")
            .unwrap()
            .abs()
            .unwrap()
            .as_string(),
        "500 USD"
    );
    assert_eq!(
        Money::from_code(0, "USD")
            .unwrap()
            .abs()
            .unwrap()
            .as_string(),
        "0 USD"
    );
}

#[test]
fn abs_extreme_errors() {
    let a = Money::new(i128::MIN, usd().currency().clone());
    assert!(matches!(a.abs(), Err(MoneyError::Overflow)));
}

// --- receiver immutability ---------------------------------------------------

#[test]
fn arithmetic_does_not_touch_the_receiver() {
    let a = Money::from_code(500, "USD").unwrap();
    let b = Money::from_code(250, "USD").unwrap();

    let _ = a.add(&b).unwrap();
    let _ = a.subtract(&b).unwrap();
    let _ = a.negate().unwrap();
    let _ = a.abs().unwrap();

    assert_eq!(a.as_string(), "500 USD");
    assert_eq!(b.as_string(), "250 USD");
}
