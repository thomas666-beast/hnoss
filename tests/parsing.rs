//! Integration tests for parsing.

use hnoss::{Currency, Money, MoneyError, ParseOptions};

// --- from_str_with_code ------------------------------------------------------

#[test]
fn parse_usd() {
    let m = Money::from_str_with_code("5.00", "USD", &ParseOptions::default()).unwrap();
    assert_eq!(m.amount(), 500);
    assert_eq!(m.currency().code(), "USD");
}

#[test]
fn parse_usd_whole() {
    assert_eq!(
        Money::from_str_with_code("5", "USD", &ParseOptions::default())
            .unwrap()
            .amount(),
        500
    );
}

#[test]
fn parse_usd_half_cent() {
    assert_eq!(
        Money::from_str_with_code("5.5", "USD", &ParseOptions::default())
            .unwrap()
            .amount(),
        550
    );
}

#[test]
fn parse_negative() {
    assert_eq!(
        Money::from_str_with_code("-0.05", "USD", &ParseOptions::default())
            .unwrap()
            .amount(),
        -5
    );
}

#[test]
fn parse_positive_sign() {
    assert_eq!(
        Money::from_str_with_code("+0.05", "USD", &ParseOptions::default())
            .unwrap()
            .amount(),
        5
    );
}

#[test]
fn parse_jpy_whole() {
    assert_eq!(
        Money::from_str_with_code("500", "JPY", &ParseOptions::default())
            .unwrap()
            .amount(),
        500
    );
}

#[test]
fn parse_bhd() {
    assert_eq!(
        Money::from_str_with_code("0.500", "BHD", &ParseOptions::default())
            .unwrap()
            .amount(),
        500
    );
}

#[test]
fn parse_btc() {
    assert_eq!(
        Money::from_str_with_code("0.00000500", "BTC", &ParseOptions::default())
            .unwrap()
            .amount(),
        500
    );
}

#[test]
fn parse_usdt() {
    assert_eq!(
        Money::from_str_with_code("1.000000", "USDT", &ParseOptions::default())
            .unwrap()
            .amount(),
        1_000_000
    );
}

#[test]
fn parse_trx() {
    assert_eq!(
        Money::from_str_with_code("1.000000", "TRX", &ParseOptions::default())
            .unwrap()
            .amount(),
        1_000_000
    );
}

#[test]
fn parse_with_grouping() {
    assert_eq!(
        Money::from_str_with_code("1,234.56", "USD", &ParseOptions::default())
            .unwrap()
            .amount(),
        123456
    );
}

#[test]
fn parse_with_loose_grouping() {
    assert_eq!(
        Money::from_str_with_code("12,34.56", "USD", &ParseOptions::default())
            .unwrap()
            .amount(),
        123456
    );
}

#[test]
fn parse_european() {
    let opts = ParseOptions {
        decimal_mark: ',',
        thousands_separator: ".".into(),
        ..Default::default()
    };
    assert_eq!(
        Money::from_str_with_code("1.234,56", "EUR", &opts)
            .unwrap()
            .amount(),
        123456
    );
}

#[test]
fn parse_custom_separators() {
    let opts = ParseOptions {
        decimal_mark: '.',
        thousands_separator: " ".into(),
        ..Default::default()
    };
    assert_eq!(
        Money::from_str_with_code("1 234.56", "USD", &opts)
            .unwrap()
            .amount(),
        123456
    );
}

// --- errors -----------------------------------------------------------------

#[test]
fn parse_empty_errors() {
    assert!(matches!(
        Money::from_str_with_code("", "USD", &ParseOptions::default()),
        Err(MoneyError::Parse(_))
    ));
}

#[test]
fn parse_malformed_errors() {
    let opts = ParseOptions::default();
    for bad in [
        "abc", "5.", ".5", "5e3", "5 5", "--5", "-", "+", "5..5", "5,", ",5", "5,,6",
    ] {
        let err = Money::from_str_with_code(bad, "USD", &opts).unwrap_err();
        assert!(matches!(err, MoneyError::Parse(_)), "{bad:?} -> {err:?}");
    }
}

#[test]
fn parse_too_many_decimals_errors() {
    assert!(matches!(
        Money::from_str_with_code("5.001", "USD", &ParseOptions::default()),
        Err(MoneyError::Parse(_))
    ));
    assert!(matches!(
        Money::from_str_with_code("5.5", "JPY", &ParseOptions::default()),
        Err(MoneyError::Parse(_))
    ));
    assert!(matches!(
        Money::from_str_with_code("0.0001", "BHD", &ParseOptions::default()),
        Err(MoneyError::Parse(_))
    ));
}

#[test]
fn parse_bad_currency_errors() {
    let opts = ParseOptions::default();
    for bad in ["U5D", "US", "TOOLONGXX", ""] {
        assert!(matches!(
            Money::from_str_with_code("5.00", bad, &opts),
            Err(MoneyError::InvalidCurrency(_))
        ));
    }
}

// --- FromStr -----------------------------------------------------------------

#[test]
fn from_str_basic() {
    let m: Money = "5.00 USD".parse().unwrap();
    assert_eq!(m.amount(), 500);
    assert_eq!(m.currency().code(), "USD");
}

#[test]
fn from_str_negative() {
    let m: Money = "-0.05 USD".parse().unwrap();
    assert_eq!(m.amount(), -5);
}

#[test]
fn from_str_crypto() {
    let m: Money = "0.00000500 BTC".parse().unwrap();
    assert_eq!(m.amount(), 500);
}

#[test]
fn from_str_missing_currency_errors() {
    assert!("5.00".parse::<Money>().is_err());
}

#[test]
fn from_str_bad_currency_errors() {
    assert!("5.00 U5D".parse::<Money>().is_err());
}

// --- roundtrip ---------------------------------------------------------------

#[test]
fn format_then_parse_roundtrips() {
    let cases = [
        Money::new(500, Currency::new("USD").unwrap()),
        Money::new(-5, Currency::new("USD").unwrap()),
        Money::new(123456, Currency::new("USD").unwrap()),
        Money::new(500, Currency::new("JPY").unwrap()),
        Money::new(123456, Currency::new("JPY").unwrap()),
        Money::new(500, Currency::new("BHD").unwrap()),
        Money::new(1234, Currency::new("BHD").unwrap()),
        Money::new(500, Currency::new("BTC").unwrap()),
        Money::new(1_000_000, Currency::new("TRX").unwrap()),
        Money::new(1_000_000, Currency::new("USDT").unwrap()),
    ];

    for m in cases {
        let s = m.format();
        let parsed: Money = s.parse().unwrap();
        assert_eq!(parsed, m, "roundtrip failed for {s}");
    }
}

// --- from_str_with (Currency object, not code string) ------------------------

#[test]
fn from_str_with_currency_object() {
    let usd = Currency::new("USD").unwrap();
    let m = Money::from_str_with("5.00", &usd, &ParseOptions::default()).unwrap();
    assert_eq!(m.amount(), 500);
    assert_eq!(m.currency().code(), "USD");
}
