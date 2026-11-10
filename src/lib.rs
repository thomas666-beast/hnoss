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
//! # Example
//!
//! ```
//! use hnoss::{Currency, Money};
//!
//! let usd = Currency::new("USD").unwrap();
//! let price = Money::new(500_i128, usd);
//! assert_eq!(price.amount(), 500);
//! assert_eq!(price.as_string(), "500 USD");
//! assert_eq!(price.format(), "5.00 USD");
//!
//! let parsed: Money = "5.00 USD".parse().unwrap();
//! assert_eq!(parsed.amount(), 500);
//! ```
//!
//! # Examples
//!
//! See the `examples/` directory for runnable programs:
//! `cargo run --example basic`, `cargo run --example split_bill`,
//! `cargo run --example crypto`.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

mod currency;
mod error;
mod format;
mod money;
mod parse;
mod repository;

pub use crate::currency::{Currency, CurrencyError};
pub use crate::error::MoneyError;
pub use crate::format::FormatOptions;
pub use crate::money::Money;
pub use crate::parse::ParseOptions;
pub use crate::repository::Repository;
