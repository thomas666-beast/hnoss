## [Unreleased]

## [0.6.0] - 2026-11-10

### Added

- `Money::allocate(&[u128]) -> Result<Vec<Money>, MoneyError>` — split an
  amount across N recipients by integer ratios. The sum of the returned
  values is exactly the original amount (no minor units lost or created).
  Leftover units are distributed one at a time to the earliest indices.
  Mirror-symmetric for negative amounts: `allocate(-x)` is element-wise
  the negation of `allocate(x)`.
- `MoneyError::InvalidAllocation(String)` — raised for empty or all-zero
  ratio lists.
- `tests/allocation.rs` integration test.

## [0.5.0] - 2026-11-10

### Added

- `Money::multiply`, `Money::divide` (floor), `Money::divide_with_remainder`.
- `tests/multiplication.rs` integration test.

### Notes

- Division uses floor semantics for consistency with the Perl implementation
  and with Python / Ruby.

## [0.4.0] - 2026-11-10

### Added

- `Money::compare`, `Money::equals`, `Money::is_zero`, `Money::is_positive`,
  `Money::is_negative`.
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
