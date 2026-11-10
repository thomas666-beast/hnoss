## [Unreleased]

## [0.7.0] - 2026-11-10

### Added

- `FormatOptions` struct with `with_currency`, `currency_separator`,
  `decimal_mark`, `thousands_separator`, and `group_size` fields. All
  defaulted.
- `Money::format() -> String` — human-readable rendering using the
  currency's exponent and default options. Cannot fail.
- `Money::format_with(&FormatOptions) -> Result<String, MoneyError>` —
  custom-option rendering. Fails on `group_size == 0`.
- `tests/formatting.rs` integration test.

### Notes

- `format` respects the currency's exponent: USD renders `"5.00"`, JPY
  renders `"500"`, BHD renders `"0.500"`, BTC renders `"0.00000500"`.
- `Display` still yields `as_string()`, not `format()`. `as_string` is the
  canonical, round-trippable form.

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
