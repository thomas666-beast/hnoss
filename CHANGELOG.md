## [Unreleased]

## [0.4.0] - 2026-11-10

### Added

- `Money::compare` — same-currency ordering returning
  `Result<Ordering, MoneyError>` (`Ordering::Less` / `Equal` / `Greater`).
  Errors on currency mismatch.
- `Money::equals` — named alias for `==`; returns `bool`, never errors
  (currencies that differ are simply unequal).
- `Money::is_zero`, `Money::is_positive`, `Money::is_negative` — boolean
  predicates. `is_positive` is false for zero.
- `tests/comparison.rs` integration test.

### Notes

- `Ord` / `PartialOrd` are deliberately not implemented. `cmp` cannot fail,
  and panicking on currency mismatch is a footgun. Use `compare` explicitly.

## [0.3.0] - 2026-11-10

### Added

- `Money::add`, `Money::subtract` — same-currency addition and subtraction.
  Both return `Result<Money, MoneyError>`, failing on currency mismatch or
  `i128` overflow.
- `Money::negate`, `Money::abs` — sign flip and absolute value, failing only
  on `i128::MIN`.
- `tests/arithmetic.rs` integration test.

## [0.2.0] - 2026-11-10

### Added

- `Money` value object: `new`, `from_code`, `amount`, `currency`,
  `as_string`, `Display`, `Debug`, `PartialEq`, `Eq`, `Hash`.
- `MoneyError` enum with `InvalidCurrency`, `Overflow`, `CurrencyMismatch`,
  `NonInteger`, `DivisionByZero`, and `Parse` variants.

## [0.1.0] - 2026-11-09

### Added

- `Currency` value object: constructs from a 3-to-8-letter code, normalizes
  case, exposes `code()`, `exponent()`, `subunit()`.
- `Repository`: built-in exponent table covering active ISO 4217 currencies,
  metatokens, historical codes, and the documented non-ISO extensions BTC,
  TRX, USDT. `Repository::shared()`, `Repository::from_pairs()`, `Default`
  impl, `exponent()`, `knows()`, `fallback()`, `all_codes()`.
- `CurrencyError` enum for construction errors.
