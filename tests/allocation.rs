//! Integration tests for `hnoss::Money::allocate`.

use hnoss::{Money, MoneyError};

// --- basic splits ------------------------------------------------------------

#[test]
fn allocate_three_way() {
    let a = Money::from_code(100, "USD").unwrap();
    let parts = a.allocate(&[1, 1, 1]).unwrap();
    assert_eq!(parts.len(), 3);
    assert_eq!(parts[0].as_string(), "34 USD");
    assert_eq!(parts[1].as_string(), "33 USD");
    assert_eq!(parts[2].as_string(), "33 USD");
}

#[test]
fn allocate_by_percentages() {
    let a = Money::from_code(100, "USD").unwrap();
    let parts = a.allocate(&[70, 20, 10]).unwrap();
    assert_eq!(parts[0].as_string(), "70 USD");
    assert_eq!(parts[1].as_string(), "20 USD");
    assert_eq!(parts[2].as_string(), "10 USD");
}

#[test]
fn allocate_single_recipient() {
    let a = Money::from_code(100, "USD").unwrap();
    let parts = a.allocate(&[1]).unwrap();
    assert_eq!(parts.len(), 1);
    assert_eq!(parts[0].as_string(), "100 USD");
}

#[test]
fn allocate_zero_ratio_slots() {
    let a = Money::from_code(100, "USD").unwrap();
    let parts = a.allocate(&[0, 1, 0]).unwrap();
    assert_eq!(parts[0].as_string(), "0 USD");
    assert_eq!(parts[1].as_string(), "100 USD");
    assert_eq!(parts[2].as_string(), "0 USD");
}

// --- negative amounts --------------------------------------------------------

#[test]
fn allocate_negative_amount_mirror() {
    let a = Money::from_code(-100, "USD").unwrap();
    let parts = a.allocate(&[1, 1, 1]).unwrap();
    assert_eq!(parts[0].as_string(), "-34 USD");
    assert_eq!(parts[1].as_string(), "-33 USD");
    assert_eq!(parts[2].as_string(), "-33 USD");
}

#[test]
fn allocate_negative_odd_mirror() {
    let a = Money::from_code(-101, "USD").unwrap();
    let parts = a.allocate(&[1, 1, 1]).unwrap();
    assert_eq!(parts[0].as_string(), "-34 USD");
    assert_eq!(parts[1].as_string(), "-34 USD");
    assert_eq!(parts[2].as_string(), "-33 USD");
}

// --- invariants --------------------------------------------------------------

#[test]
fn sum_equals_original() {
    for amount in [-999_i128, -100, -1, 0, 1, 100, 999] {
        for ratios in [vec![1u128], vec![1, 1], vec![1, 1, 1], vec![7, 11, 13, 17]] {
            let m = Money::from_code(amount, "USD").unwrap();
            let parts = m.allocate(&ratios).unwrap();
            let sum = parts
                .iter()
                .fold(Money::from_code(0, "USD").unwrap(), |acc, p| {
                    acc.add(p).unwrap()
                });
            assert_eq!(sum.amount(), amount, "amount={amount} ratios={ratios:?}");
        }
    }
}

#[test]
fn mirror_symmetric_under_negation() {
    for amount in [1_i128, 7, 100, 101, 999] {
        for ratios in [
            vec![1u128, 1],
            vec![1, 1, 1],
            vec![70, 20, 10],
            vec![1, 2, 3, 4],
        ] {
            let pos = Money::from_code(amount, "USD")
                .unwrap()
                .allocate(&ratios)
                .unwrap();
            let neg = Money::from_code(-amount, "USD")
                .unwrap()
                .allocate(&ratios)
                .unwrap();
            for (p, n) in pos.iter().zip(neg.iter()) {
                assert_eq!(p.amount(), -n.amount(), "amount={amount} ratios={ratios:?}");
            }
        }
    }
}

// --- crypto ----------------------------------------------------------------

#[test]
fn allocate_btc_satoshi() {
    // 1 BTC = 100,000,000 satoshi. Split across 3 recipients.
    let one_btc = Money::from_code(100_000_000, "BTC").unwrap();
    let parts = one_btc.allocate(&[1, 1, 1]).unwrap();

    assert_eq!(parts[0].as_string(), "33333334 BTC");
    assert_eq!(parts[1].as_string(), "33333333 BTC");
    assert_eq!(parts[2].as_string(), "33333333 BTC");

    let sum = parts
        .iter()
        .fold(Money::from_code(0, "BTC").unwrap(), |acc, p| {
            acc.add(p).unwrap()
        });
    assert_eq!(sum.amount(), 100_000_000);
}

// --- errors -----------------------------------------------------------------

#[test]
fn allocate_empty_ratios_errors() {
    let a = Money::from_code(100, "USD").unwrap();
    match a.allocate(&[]).unwrap_err() {
        MoneyError::InvalidAllocation(msg) => assert!(msg.contains("no ratios")),
        other => panic!("expected InvalidAllocation, got {other:?}"),
    }
}

#[test]
fn allocate_all_zero_errors() {
    let a = Money::from_code(100, "USD").unwrap();
    match a.allocate(&[0, 0, 0]).unwrap_err() {
        MoneyError::InvalidAllocation(msg) => {
            assert!(msg.contains("all ratios are zero"))
        }
        other => panic!("expected InvalidAllocation, got {other:?}"),
    }
}

#[test]
fn allocate_does_not_touch_receiver() {
    let a = Money::from_code(100, "USD").unwrap();
    let _ = a.allocate(&[1, 1, 1]).unwrap();
    assert_eq!(a.as_string(), "100 USD");
}
