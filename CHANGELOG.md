
## File: `CHANGELOG.md`

```markdown
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-11-09

### Added

- `Currency` value object: constructs from a 3-to-8-letter code, normalizes
  case, exposes `code()`, `exponent()`, `subunit()`.
- `Repository`: built-in exponent table covering active ISO 4217 currencies,
  metatokens, historical codes, and the documented non-ISO extensions BTC,
  TRX, USDT. `Repository::default()`, `Repository::from_pairs()`, `exponent()`,
  `knows()`, `fallback()`, `all_codes()`.
- `CurrencyError` enum for construction errors.

[Unreleased]: https://github.com/thomas666-beast/hnoss/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/thomas666-beast/hnoss/releases/tag/v0.1.0
