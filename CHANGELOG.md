## [Unreleased]

## [0.5.0] - 2026-11-10

### Added

- `Money::multiply(factor: i128)` — exact integer multiplication. Returns
  `Result<Money, MoneyError>`, failing on `i128` overflow.
- `Money::divide(divisor: i128)` — **floor** division (like Python's `//`
  and Ruby's `/`). Returns `Result<Money, MoneyError>`, failing on division
  by zero or `i128::MIN / -1`.
- `Money::divide_with_remainder(divisor: i128)` — returns `(quotient,
  remainder)` as two `Money` values, with the invariant
  `q * divisor + r == original` and the remainder's sign matching the
  divisor's. Same failure modes as `divide`.
- `tests/multiplication.rs` integration test.

### Notes

- Rust's `/` operator truncates toward zero. `Money::divide` deliberately
  uses floor division for cross-language consistency with the Perl
  `Coin::Hnoss` library and with Python / Ruby. For positive amounts the
  two are identical.

## [0.4.0] - 2026-11-10

### Added

- `Money::compare`, `Money::equals`, `Money::is_zero`,
  `Money::is_positive`, `Money::is_negative`.
- `tests/comparison.rs` integration test.

### Notes

- `Ord` / `PartialOrd` are deliberately not implemented.

## [0.3.0] - 2026-11-10

### Added

- `Money::add`, `Money::subtract`, `Money::negate`, `Money::abs`.
- `tests/arithmetic.rs` integration test.

## [0.2.0] - 2026-11-10

### Added

- `Money` value object: `new`, `from_code`, `amount`, `currency`,
  `as_string`, `Display`, `Debug`, `PartialEq`, `Eq`, `Hash`.
- `MoneyError` enum with `InvalidCurrency`, `Overflow`, `CurrencyMismatch`,
  `NonInteger`, `DivisionByZero`, and `Parse` variants.

## [0.1.0] - 2026-11-09

### Added

- `Currency` value object.
- `Repository`: built-in exponent table (ISO 4217 + BTC / TRX / USDT).
- `CurrencyError` enum.
