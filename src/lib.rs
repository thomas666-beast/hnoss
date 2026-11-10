//! Exact monetary values for Rust. No floating point, ever.
//!
//! # Overview
//!
//! `hnoss` is a value-object library for monetary amounts. Amounts are
//! stored as integers in the currency's smallest unit (e.g. cents for USD)
//! and arithmetic is exact.
//!
//! The crate has **zero runtime dependencies**: everything here uses only
//! `std`.
//!
//! # Design
//!
//! * [`Currency`] is a small value object wrapping a 3-to-8-letter currency
//!   code and its exponent (number of decimal places).
//! * [`Money`] is an immutable amount in a given currency. Construction,
//!   equality, and stringification are available; arithmetic, comparison,
//!   allocation, formatting, and parsing arrive in later releases.
//! * [`MoneyError`] is the crate's single error type.
//!
//! # Example
//!
//! ```
//! use hnoss::{Currency, Money};
//!
//! let usd = Currency::new("USD").unwrap();
//! let price = Money::new(500_i128, usd);
//! assert_eq!(price.amount(), 500);
//! assert_eq!(price.as_string(), "500 USD");
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

mod currency;
mod error;
mod format;
mod money;
mod repository;

pub use crate::currency::{Currency, CurrencyError};
pub use crate::error::MoneyError;
pub use crate::format::FormatOptions;
pub use crate::money::Money;
pub use crate::repository::Repository;
