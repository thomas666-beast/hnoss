# hnoss

Exact monetary values for Rust, with **zero dependencies**.

The name is Old Norse: *Hnoss* is a goddess of treasure and precious things.
This is the Rust sibling of the Perl `Coin::Hnoss` library; the two share a
design and produce numerically identical results for every supported
operation.

## Status

`0.8.1` — feature-complete for the core value-object scope. Currency, money
construction, arithmetic, comparison, allocation, formatting, and parsing
are all implemented and tested on stable Rust (1.70+).

Planned for later releases: `serde` support (opt-in feature flag),
`Money<C>` phantom-typed API for compile-time currency safety, and
locale-aware formatting.

## Requirements

- Rust 1.70 or newer (uses `std::sync::OnceLock` for the default repository).
- No runtime dependencies. Everything is `std`.

## Installation

    [dependencies]
    hnoss = "0.8"

## Quickstart

    use hnoss::Money;

    // Construct from minor units (cents, satoshi, ...)
    let price = Money::from_code(500, "USD").unwrap();   // $5.00
    let tax   = Money::from_code( 50, "USD").unwrap();   // $0.50

    let total = price.add(&tax).unwrap();
    assert_eq!(total.format(), "5.50 USD");
    assert_eq!(total.as_string(), "550 USD");   // raw canonical form

    // Construct from major-unit strings
    let m: Money = "1,234.56 USD".parse().unwrap();
    assert_eq!(m.format(), "1,234.56 USD");

    // Currency-aware exponent: USD 2, JPY 0, BHD 3, BTC 8
    assert_eq!(Money::from_code(500, "JPY").unwrap().format(), "500 JPY");
    assert_eq!(Money::from_code(500, "BHD").unwrap().format(), "0.500 BHD");
    assert_eq!(Money::from_code(500, "BTC").unwrap().format(), "0.00000500 BTC");

## Core concepts

- Amounts are integers in the currency's smallest unit.
  `Money::from_code(500, "USD")` means 500 cents. No floating-point
  arithmetic anywhere.

- Currency-aware. Exponent comes from an ISO 4217 table (USD has 2, JPY
  has 0, BHD has 3, UYW has 4), plus a small set of documented non-ISO
  extensions: BTC (Bitcoin, 8 = satoshi), TRX (TRON, 6 = sun), and USDT
  (Tether USD, 6 decimals on both TRC20/TRON and ERC20/Ethereum).

- Immutable. All operations return new `Money` values; the receiver is
  never modified.

- Exact. `i128` internally, so amounts up to ~1.7e38 minor units (far more
  money than exists) are safe. Overflow is a typed error, never a silent
  wrap.

- Zero dependencies. No `num-bigint`, no `serde`, no `thiserror`, no
  `lazy_static`. Everything is `std`.

## Common operations

### Construction

    use hnoss::{Currency, Money, ParseOptions};

    // from minor units
    Money::from_code(500, "USD").unwrap();

    // from major-unit string (currency supplied separately)
    let usd = Currency::new("USD").unwrap();
    Money::from_str_with("5.00", &usd, &ParseOptions::default()).unwrap();

    // from "<amount> <code>" via FromStr
    let m: Money = "5.00 USD".parse().unwrap();

    // with custom separators
    let m = Money::from_str_with_code(
        "1.234,56", "EUR",
        &ParseOptions {
            decimal_mark: ',',
            thousands_separator: ".".into(),
            ..Default::default()
        },
    ).unwrap();

### Arithmetic

    let a = Money::from_code(500, "USD").unwrap();
    let b = Money::from_code(250, "USD").unwrap();

    a.add(&b).unwrap();                                  // 750 USD
    a.subtract(&b).unwrap();                             // 250 USD
    a.multiply(3).unwrap();                              // 1500 USD
    a.divide(2).unwrap();                                // 250 USD (floor)
    a.divide_with_remainder(3).unwrap();                 // (166 USD, 2 USD)
    a.negate().unwrap();                                 // -500 USD
    a.abs().unwrap();                                    // 500 USD

Every arithmetic operation returns `Result<Money, MoneyError>`. Overflow,
currency mismatch, and division by zero are typed errors, not panics. For
percentages, use `multiply(n).divide(d)` — exact by construction:

    let price = Money::from_code(10_000, "USD").unwrap();  // $100.00
    let tax   = price.multiply(5).unwrap().divide(100).unwrap();  // $5.00

### Comparison

    use std::cmp::Ordering;

    let a = Money::from_code(500, "USD").unwrap();
    let b = Money::from_code(250, "USD").unwrap();

    a.compare(&b).unwrap();   // Ordering::Greater
    a.equals(&b);             // false
    a.is_positive();          // true
    a.is_zero();              // false
    a.is_negative();          // false

`compare` requires the same currency and returns `Result<Ordering,
MoneyError>`. `equals` is the same as `==`; it never errors on currency
mismatch (two different currencies are simply unequal). `Ord` /
`PartialOrd` are deliberately not implemented, because `cmp` cannot fail
and panicking on currency mismatch would be a footgun.

### Allocation

Split money without losing a minor unit:

    let hundred = Money::from_code(100, "USD").unwrap();
    let parts = hundred.allocate(&[1, 1, 1]).unwrap();
    assert_eq!(parts[0].as_string(), "34 USD");
    assert_eq!(parts[1].as_string(), "33 USD");
    assert_eq!(parts[2].as_string(), "33 USD");

    // Sum of parts == original amount, always.
    // For negative amounts, the result is the element-wise negation of the
    // positive case: allocate(-x) == -allocate(x).

### Formatting

    use hnoss::FormatOptions;

    Money::from_code(123456, "USD").unwrap().format();   // "1,234.56 USD"
    Money::from_code(500, "JPY").unwrap().format();      // "500 JPY"
    Money::from_code(500, "BHD").unwrap().format();      // "0.500 BHD"

    let opts = FormatOptions {
        decimal_mark: ',',
        thousands_separator: ".".into(),
        ..Default::default()
    };
    Money::from_code(123456, "EUR").unwrap()
        .format_with(&opts).unwrap();   // "1.234,56 EUR"

`format()` uses default options and cannot fail. `format_with` accepts
custom options and fails on `group_size == 0`.

### Parsing

    // Currency in the string
    let m: Money = "5.00 USD".parse().unwrap();

    // Currency supplied separately
    let m = Money::from_str_with_code("5.00", "USD", &ParseOptions::default())
        .unwrap();

    // Round-trip with format
    let original = Money::from_code(123456, "USD").unwrap();
    let text = original.format();          // "1,234.56 USD"
    let back: Money = text.parse().unwrap();
    assert_eq!(back, original);

Parsing is strict: too many decimal digits is an error, never a silent
round. `.` and `,` positions are configurable via `ParseOptions`.

## Supported currencies

The built-in table covers:

- Active ISO 4217 currencies (USD, EUR, GBP, JPY, BHD, ...).
- ISO 4217 metatokens (XAU, XAG, XDR, XTS, XXX, ...).
- Historical ISO 4217 codes still common in legacy data (BYR, MRO, ...).
- Documented non-ISO extensions: BTC (Bitcoin, 8 = satoshi), TRX (TRON,
  6 = sun), USDT (Tether USD, 6).

Unknown codes fall back to exponent 2. Use `Repository::from_pairs` to
build a custom table.

## Examples

See `examples/` for runnable programs:

    cargo run --example basic
    cargo run --example split_bill
    cargo run --example crypto

## Design and non-goals

- No currency symbols or locales in `format`. It emits `"5.00 USD"`, not
  `"$5.00"`. Locale-aware formatting is a planned `hnoss::Formatter`-style
  layer.

- No exchange rates. Two different currencies are not comparable and not
  addable.

- No rounding. `format` never rounds; `from_str_with` never rounds;
  arithmetic is exact. If you need rounding, do it explicitly at the
  boundary.

- No database layer. Serialize to `(amount, code)` yourself; the library
  gives you `->amount()` and `->currency().code()` for exactly this.

- No crypto chain awareness. A `Money` in `USDT` doesn't know whether it
  lives on TRON or Ethereum. Model that at the application layer.

## Documentation

- API reference: `cargo doc --open`, or https://docs.rs/hnoss.

- The Perl sibling has an extended documentation set (concepts, cookbook,
  FAQ) in the Perl repository's `docs/` directory. The concepts and
  semantics are identical.

## See also

- Martin Fowler, *Money* pattern:
  https://martinfowler.com/eaaCatalog/money.html
- ISO 4217 currency codes:
  https://www.iso.org/iso-4217-currency-codes.html
- PHP: moneyphp/money — https://github.com/moneyphp/money
- Perl: Coin::Hnoss — https://metacpan.org/pod/Coin::Hnoss

## License

Licensed under either of:

- Apache License, Version 2.0 (LICENSE-APACHE)
- MIT License (LICENSE-MIT)

at your option.

## Contributing

See CONTRIBUTING.md.

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual-licensed as above, without any additional terms or
conditions.
