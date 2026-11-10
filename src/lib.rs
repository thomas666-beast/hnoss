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
//! * [`Money`] (coming in a later task) is an immutable amount in a given
//!   currency, with exact arithmetic, comparison, allocation, formatting,
//!   and parsing.
//!
//! # Example
//!
//! ```
//! use hnoss::Currency;
//!
//! let usd = Currency::new("USD").unwrap();
//! assert_eq!(usd.code(), "USD");
//! assert_eq!(usd.exponent(), 2);
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

mod currency;
mod repository;

pub use crate::currency::{Currency, CurrencyError};
pub use crate::repository::Repository;
