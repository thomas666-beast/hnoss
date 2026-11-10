## [Unreleased]

### Added

- `README.md` rewritten with quickstart, common operations, examples,
  design notes, and non-goals.
- `LICENSE-MIT` and `LICENSE-APACHE` (dual-licensed as `MIT OR Apache-2.0`).
- `examples/basic.rs`, `examples/split_bill.rs`, `examples/crypto.rs`.
- `tests/property.rs` — deterministic-PRNG property tests (no external
  dependencies) covering `allocate` sum/mirror invariants,
  `divide_with_remainder` invariants, `format`/`parse` round-trip,
  and add/subtract algebraic properties.
- `Cargo.toml`: `include` list for a minimal published crate;
  `[package.metadata.docs.rs]`.

### Notes

- No API changes. `0.8.0` remains the current published version.

## [0.8.1] - 2026-11-10

### Added

Update README

## [0.8.0] - 2026-11-10

### Added

- `ParseOptions` struct with `decimal_mark`, `thousands_separator`, and
  `group_size` fields. All defaulted.
- `Money::from_str_with(&str, &Currency, &ParseOptions) -> Result<Money, MoneyError>`
  — parse a major-unit numeric string given a known currency.
- `Money::from_str_with_code(&str, impl AsRef<str>, &ParseOptions) -> Result<Money, MoneyError>`
  — convenience over `from_str_with` accepting a code string.
- `impl FromStr for Money` — parses `"<amount> <code>"` (e.g. `"5.00 USD"`)
  using default `ParseOptions`.
- `tests/parsing.rs` integration test.

### Notes

- Parsing is strict: too many decimal digits is an error, never a silent
  round. Round-trips with `format()` are lossless.
- Thousands separators are stripped but not validated for group positions.

## [0.7.0] - 2026-11-10

### Added

- `FormatOptions`, `Money::format`, `Money::format_with`.
- `tests/formatting.rs`.

## [0.6.0] - 2026-11-10

### Added

- `Money::allocate`, `MoneyError::InvalidAllocation`.
- `tests/allocation.rs`.

## [0.5.0] - 2026-11-10

### Added

- `Money::multiply`, `Money::divide` (floor), `Money::divide_with_remainder`.
- `tests/multiplication.rs`.

## [0.4.0] - 2026-11-10

### Added

- `Money::compare`, `Money::equals`, `Money::is_zero`, `Money::is_positive`,
  `Money::is_negative`.
- `tests/comparison.rs`.

## [0.3.0] - 2026-11-10

### Added

- `Money::add`, `Money::subtract`, `Money::negate`, `Money::abs`.
- `tests/arithmetic.rs`.

## [0.2.0] - 2026-11-10

### Added

- `Money` value object: `new`, `from_code`, `amount`, `currency`,
  `as_string`, `Display`, `Debug`, `PartialEq`, `Eq`, `Hash`.
- `MoneyError` enum.

## [0.1.0] - 2026-11-09

### Added

- `Currency` value object.
- `Repository`.
- `CurrencyError` enum.
