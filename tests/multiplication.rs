//! Integration tests for multiplication and division on `hnoss::Money`.

use hnoss::{Money, MoneyError};

// --- multiply ----------------------------------------------------------------

#[test]
fn multiply_basics() {
    let a = Money::from_code(500, "USD").unwrap();
    assert_eq!(a.multiply(3).unwrap().as_string(), "1500 USD");
    assert_eq!(a.multiply(0).unwrap().as_string(), "0 USD");
    assert_eq!(a.multiply(-1).unwrap().as_string(), "-500 USD");
    assert_eq!(a.multiply(1).unwrap().as_string(), "500 USD");
}

#[test]
fn multiply_does_not_touch_receiver() {
    let a = Money::from_code(500, "USD").unwrap();
    let _ = a.multiply(3).unwrap();
    assert_eq!(a.as_string(), "500 USD");
}

#[test]
fn multiply_overflow_errors() {
    let a = Money::new(
        i128::MAX,
        Money::from_code(0, "USD").unwrap().currency().clone(),
    );
    assert!(matches!(a.multiply(2), Err(MoneyError::Overflow)));
}

#[test]
fn multiply_preserves_currency() {
    let a = Money::from_code(500, "EUR").unwrap();
    assert_eq!(a.multiply(2).unwrap().currency().code(), "EUR");
}

#[test]
fn multiply_percentage_recipe() {
    // 5% tax on $100.00:
    //   $100.00 = 10000 cents
    //   tax = 10000 * 5 / 100 = 500 cents = $5.00
    let price = Money::from_code(10_000, "USD").unwrap();
    let tax = price.multiply(5).unwrap().divide(100).unwrap();
    assert_eq!(tax.as_string(), "500 USD");
}

// --- divide ------------------------------------------------------------------

#[test]
fn divide_exact() {
    let a = Money::from_code(500, "USD").unwrap();
    assert_eq!(a.divide(2).unwrap().as_string(), "250 USD");
    assert_eq!(a.divide(5).unwrap().as_string(), "100 USD");
}

#[test]
fn divide_floors_positive() {
    assert_eq!(
        Money::from_code(500, "USD")
            .unwrap()
            .divide(3)
            .unwrap()
            .as_string(),
        "166 USD"
    );
}

#[test]
fn divide_floors_negative() {
    assert_eq!(
        Money::from_code(-500, "USD")
            .unwrap()
            .divide(3)
            .unwrap()
            .as_string(),
        "-167 USD"
    );
}

#[test]
fn divide_by_negative() {
    assert_eq!(
        Money::from_code(500, "USD")
            .unwrap()
            .divide(-3)
            .unwrap()
            .as_string(),
        "-167 USD"
    );
    assert_eq!(
        Money::from_code(-500, "USD")
            .unwrap()
            .divide(-3)
            .unwrap()
            .as_string(),
        "166 USD"
    );
}

#[test]
fn divide_by_zero_errors() {
    let a = Money::from_code(500, "USD").unwrap();
    assert!(matches!(a.divide(0), Err(MoneyError::DivisionByZero)));
}

#[test]
fn divide_min_by_minus_one_overflows() {
    let a = Money::new(
        i128::MIN,
        Money::from_code(0, "USD").unwrap().currency().clone(),
    );
    assert!(matches!(a.divide(-1), Err(MoneyError::Overflow)));
}

#[test]
fn divide_does_not_touch_receiver() {
    let a = Money::from_code(500, "USD").unwrap();
    let _ = a.divide(3).unwrap();
    assert_eq!(a.as_string(), "500 USD");
}

// --- divide_with_remainder ---------------------------------------------------

#[test]
fn divide_with_remainder_positive() {
    let a = Money::from_code(500, "USD").unwrap();
    let (q, r) = a.divide_with_remainder(3).unwrap();
    assert_eq!(q.as_string(), "166 USD");
    assert_eq!(r.as_string(), "2 USD");
}

#[test]
fn divide_with_remainder_exact() {
    let a = Money::from_code(500, "USD").unwrap();
    let (q, r) = a.divide_with_remainder(5).unwrap();
    assert_eq!(q.as_string(), "100 USD");
    assert_eq!(r.as_string(), "0 USD");
}

#[test]
fn divide_with_remainder_negative_dividend() {
    let a = Money::from_code(-500, "USD").unwrap();
    let (q, r) = a.divide_with_remainder(3).unwrap();
    assert_eq!(q.as_string(), "-167 USD");
    assert_eq!(r.as_string(), "1 USD");
}

#[test]
fn divide_with_remainder_negative_divisor() {
    let a = Money::from_code(500, "USD").unwrap();
    let (q, r) = a.divide_with_remainder(-3).unwrap();
    assert_eq!(q.as_string(), "-167 USD");
    assert_eq!(r.as_string(), "-1 USD");
}

#[test]
fn divide_with_remainder_both_negative() {
    let a = Money::from_code(-500, "USD").unwrap();
    let (q, r) = a.divide_with_remainder(-3).unwrap();
    assert_eq!(q.as_string(), "166 USD");
    assert_eq!(r.as_string(), "-2 USD");
}

#[test]
fn divide_with_remainder_invariant_property() {
    // The defining invariant: q * d + r == original.
    for dividend in [-1000_i128, -999, -7, -1, 0, 1, 7, 999, 1000] {
        for divisor in [-9_i128, -5, -2, 1, 2, 5, 9] {
            let m = Money::from_code(dividend, "USD").unwrap();
            let (q, r) = m.divide_with_remainder(divisor).unwrap();
            let back = q.multiply(divisor).unwrap().add(&r).unwrap();
            assert_eq!(
                back.amount(),
                dividend,
                "{dividend} / {divisor}: q={} r={} back={}",
                q.amount(),
                r.amount(),
                back.amount()
            );
        }
    }
}

#[test]
fn divide_with_remainder_by_zero_errors() {
    let a = Money::from_code(500, "USD").unwrap();
    assert!(matches!(
        a.divide_with_remainder(0),
        Err(MoneyError::DivisionByZero)
    ));
}

#[test]
fn divide_with_remainder_preserves_currency() {
    let a = Money::from_code(500, "EUR").unwrap();
    let (q, r) = a.divide_with_remainder(3).unwrap();
    assert_eq!(q.currency().code(), "EUR");
    assert_eq!(r.currency().code(), "EUR");
}
