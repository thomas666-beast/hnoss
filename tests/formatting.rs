//! Integration tests for `hnoss::Money::format` and `format_with`.

use hnoss::{FormatOptions, Money, MoneyError};

// --- format (default options) ------------------------------------------------

#[test]
fn format_usd() {
    assert_eq!(Money::from_code(500, "USD").unwrap().format(), "5.00 USD");
    assert_eq!(Money::from_code(0, "USD").unwrap().format(), "0.00 USD");
    assert_eq!(Money::from_code(-5, "USD").unwrap().format(), "-0.05 USD");
    assert_eq!(
        Money::from_code(1234567890, "USD").unwrap().format(),
        "12,345,678.90 USD"
    );
}

#[test]
fn format_jpy() {
    assert_eq!(Money::from_code(500, "JPY").unwrap().format(), "500 JPY");
    assert_eq!(
        Money::from_code(123456, "JPY").unwrap().format(),
        "123,456 JPY"
    );
}

#[test]
fn format_bhd() {
    assert_eq!(Money::from_code(500, "BHD").unwrap().format(), "0.500 BHD");
    assert_eq!(Money::from_code(1234, "BHD").unwrap().format(), "1.234 BHD");
}

#[test]
fn format_btc_and_usdt() {
    assert_eq!(
        Money::from_code(500, "BTC").unwrap().format(),
        "0.00000500 BTC"
    );
    assert_eq!(
        Money::from_code(1_000_000, "USDT").unwrap().format(),
        "1.000000 USDT"
    );
}

#[test]
fn format_differs_from_as_string() {
    let m = Money::from_code(500, "USD").unwrap();
    assert_eq!(m.as_string(), "500 USD");
    assert_eq!(m.format(), "5.00 USD");
}

#[test]
fn display_remains_as_string_not_format() {
    let m = Money::from_code(500, "USD").unwrap();
    assert_eq!(format!("{m}"), "500 USD"); // not "5.00 USD"
}

// --- format_with -------------------------------------------------------------

#[test]
fn format_with_european_separators() {
    let m = Money::from_code(123456, "EUR").unwrap();
    let opts = FormatOptions {
        decimal_mark: ',',
        thousands_separator: ".".into(),
        ..Default::default()
    };
    assert_eq!(m.format_with(&opts).unwrap(), "1.234,56 EUR");
}

#[test]
fn format_with_no_currency() {
    let m = Money::from_code(123456, "USD").unwrap();
    let opts = FormatOptions {
        with_currency: false,
        ..Default::default()
    };
    assert_eq!(m.format_with(&opts).unwrap(), "1,234.56");
}

#[test]
fn format_with_no_grouping() {
    let m = Money::from_code(123456, "USD").unwrap();
    let opts = FormatOptions {
        thousands_separator: "".into(),
        with_currency: false,
        ..Default::default()
    };
    assert_eq!(m.format_with(&opts).unwrap(), "1234.56");
}

#[test]
fn format_with_custom_group_size() {
    let m = Money::from_code(123456789, "USD").unwrap();
    let opts = FormatOptions {
        group_size: 2,
        ..Default::default()
    };
    assert_eq!(m.format_with(&opts).unwrap(), "1,23,45,67.89 USD");
}

#[test]
fn format_with_custom_currency_separator() {
    let m = Money::from_code(500, "USD").unwrap();
    let opts = FormatOptions {
        currency_separator: "".into(),
        ..Default::default()
    };
    assert_eq!(m.format_with(&opts).unwrap(), "5.00USD");
}

#[test]
fn format_with_zero_group_size_errors() {
    let m = Money::from_code(500, "USD").unwrap();
    let opts = FormatOptions {
        group_size: 0,
        ..Default::default()
    };
    assert!(matches!(
        m.format_with(&opts),
        Err(MoneyError::InvalidAllocation(_))
    ));
}

#[test]
fn format_with_does_not_touch_receiver() {
    let m = Money::from_code(123456, "USD").unwrap();
    let opts = FormatOptions {
        decimal_mark: ',',
        ..Default::default()
    };
    let _ = m.format_with(&opts).unwrap();
    assert_eq!(m.amount(), 123456);
}

// --- round-trip intent ------------------------------------------------------
//
// format followed by a hypothetical from_string should be lossless, but
// from_string isn't implemented yet. This test is a placeholder to document
// the intent and will be filled in when Task 8 lands.

#[test]
fn format_roundtrips_via_as_string() {
    // as_string is canonical and always round-trippable through
    // Money::new(parse::<i128>(), currency). Format is not; but format's
    // output without the currency is parseable by hand:
    let m = Money::from_code(123456, "USD").unwrap();
    let s = m
        .format_with(&FormatOptions {
            with_currency: false,
            thousands_separator: "".into(),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(s, "1234.56");
    // "1234.56" -> 123456 cents is what from_string will do.
}
