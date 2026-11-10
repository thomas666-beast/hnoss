# hnoss

Exact monetary values for Rust, with **zero dependencies**.

The name is Old Norse: *Hnoss* is a goddess of treasure and precious things.
This is the Rust sibling of the Perl `Coin::Hnoss` library.

## Status

Early development. v0.1.0 ships the `Currency` value object with a built-in
exponent table (ISO 4217 + BTC / TRX / USDT). `Money`, arithmetic,
allocation, formatting, and parsing come in later releases.

## Requirements

- **Rust 1.70 or newer** (uses `OnceLock` for the default repository).
- **No runtime dependencies.** Everything is `std`.

## Installation

```toml
[dependencies]
hnoss = "0.1"
