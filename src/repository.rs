//! Currency exponent table.
//!
//! The default table contains:
//!
//! * Active ISO 4217 currencies.
//! * ISO 4217 metatokens (XAU, XAG, XDR, XTS, XXX, ...).
//! * Historical ISO 4217 codes still common in legacy data (BYR, MRO, ...).
//! * A small, explicit set of documented non-ISO extensions: `BTC`, `TRX`,
//!   `USDT`.
//!
//! All other codes fall back to exponent 2.
//!
//! # Two ways to get the default table
//!
//! * [`Repository::shared`] — returns a `&'static Repository`, the
//!   process-wide singleton. Cheap; use this for lookups.
//! * [`Repository::default`] (via the `Default` trait) — returns an owned
//!   `Repository` built from scratch. Useful when you need to own one (e.g.
//!   to combine with a custom table).

/// A lookup table of currency codes to exponents.
///
/// The default repository is a fixed table with the entries listed in the
/// module docs. Custom repositories can be built with
/// [`Repository::from_pairs`].
#[derive(Debug, Clone)]
pub struct Repository {
    entries: Vec<(&'static str, u8)>,
    fallback: u8,
}

impl Repository {
    /// Returns the process-wide default repository as a static reference.
    ///
    /// This is the cheap, allocation-free way to get the default table for
    /// lookups. Use [`Repository::default`] (the `Default` trait impl) if
    /// you need an owned `Repository`.
    pub fn shared() -> &'static Repository {
        use std::sync::OnceLock;
        static INSTANCE: OnceLock<Repository> = OnceLock::new();
        INSTANCE.get_or_init(|| Repository::from_pairs(DEFAULT_TABLE, 2))
    }

    /// Builds a custom repository from a list of `(code, exponent)` pairs.
    ///
    /// Codes must be exactly 3 to 8 uppercase ASCII letters. Exponents are
    /// `u8` values, so any non-negative integer up to 255 is accepted.
    ///
    /// # Panics
    ///
    /// Panics if any code is not 3 to 8 uppercase ASCII letters. This is a
    /// programmer error, not a runtime condition: the table is expected to
    /// be correct.
    pub fn from_pairs(pairs: &[(&'static str, u8)], fallback: u8) -> Self {
        for (code, _) in pairs {
            assert!(
                (3..=8).contains(&code.len())
                    && code.bytes().all(|b| b.is_ascii_uppercase()),
                "invalid code {code:?} in currency table"
            );
        }
        // Sort by code so lookup is binary-search-able and tests are stable.
        let mut entries = pairs.to_vec();
        entries.sort_by_key(|(code, _)| *code);
        Repository { entries, fallback }
    }

    /// Returns the exponent for `code`, or the fallback if unknown.
    ///
    /// Case-insensitive on input.
    pub fn exponent(&self, code: &str) -> u8 {
        let upper = code.to_ascii_uppercase();
        match self
            .entries
            .binary_search_by_key(&upper.as_str(), |(c, _)| c)
        {
            Ok(i) => self.entries[i].1,
            Err(_) => self.fallback,
        }
    }

    /// Returns true if `code` is in the table (does not consult the fallback).
    pub fn knows(&self, code: &str) -> bool {
        let upper = code.to_ascii_uppercase();
        self.entries
            .binary_search_by_key(&upper.as_str(), |(c, _)| c)
            .is_ok()
    }

    /// Returns the fallback exponent.
    pub fn fallback(&self) -> u8 {
        self.fallback
    }

    /// Returns all known codes, sorted.
    pub fn all_codes(&self) -> impl Iterator<Item = &'static str> + '_ {
        self.entries.iter().map(|(code, _)| *code)
    }
}

impl Default for Repository {
    /// Builds an owned `Repository` from the built-in table.
    ///
    /// For lookups only, prefer [`Repository::shared`] — it returns a
    /// `&'static Repository` without copying the table.
    fn default() -> Self {
        Repository::from_pairs(DEFAULT_TABLE, 2)
    }
}

/// The built-in table. Kept sorted so binary search works and the file stays
/// readable.
///
/// Sources:
/// * ISO 4217 (as of 2024/2025) for fiat, metatokens, and historical codes.
/// * Bitcoin: 1 BTC = 100,000,000 satoshi.
/// * TRON: 1 TRX = 1,000,000 sun.
/// * Tether USD: 1 USDT = 1,000,000 of the smallest unit (6 decimals), on
///   both TRC20 (TRON) and ERC20 (Ethereum).
const DEFAULT_TABLE: &[(&str, u8)] = &[
    // --- exponent 0 ---
    ("BIF", 0),
    ("CLP", 0),
    ("DJF", 0),
    ("GNF", 0),
    ("ISK", 0),
    ("JPY", 0),
    ("KMF", 0),
    ("KRW", 0),
    ("PYG", 0),
    ("RWF", 0),
    ("UGX", 0),
    ("UYI", 0),
    ("VND", 0),
    ("VUV", 0),
    ("XAF", 0),
    ("XOF", 0),
    ("XPF", 0),
    // --- exponent 2 (the vast majority) ---
    ("AED", 2),
    ("AFN", 2),
    ("ALL", 2),
    ("AMD", 2),
    ("ANG", 2),
    ("AOA", 2),
    ("ARS", 2),
    ("AUD", 2),
    ("AWG", 2),
    ("AZN", 2),
    ("BAM", 2),
    ("BBD", 2),
    ("BDT", 2),
    ("BGN", 2),
    ("BMD", 2),
    ("BND", 2),
    ("BOB", 2),
    ("BOV", 2),
    ("BRL", 2),
    ("BSD", 2),
    ("BTN", 2),
    ("BWP", 2),
    ("BYN", 2),
    ("BZD", 2),
    ("CAD", 2),
    ("CDF", 2),
    ("CHE", 2),
    ("CHF", 2),
    ("CHW", 2),
    ("CNY", 2),
    ("COP", 2),
    ("COU", 2),
    ("CRC", 2),
    ("CUC", 2),
    ("CUP", 2),
    ("CVE", 2),
    ("CZK", 2),
    ("DKK", 2),
    ("DOP", 2),
    ("DZD", 2),
    ("EGP", 2),
    ("ERN", 2),
    ("ETB", 2),
    ("EUR", 2),
    ("FJD", 2),
    ("FKP", 2),
    ("GBP", 2),
    ("GEL", 2),
    ("GHS", 2),
    ("GIP", 2),
    ("GMD", 2),
    ("GTQ", 2),
    ("GYD", 2),
    ("HKD", 2),
    ("HNL", 2),
    ("HRK", 2),
    ("HTG", 2),
    ("HUF", 2),
    ("IDR", 2),
    ("ILS", 2),
    ("INR", 2),
    ("IRR", 2),
    ("JMD", 2),
    ("KES", 2),
    ("KGS", 2),
    ("KHR", 2),
    ("KPW", 2),
    ("KYD", 2),
    ("KZT", 2),
    ("LAK", 2),
    ("LBP", 2),
    ("LKR", 2),
    ("LRD", 2),
    ("LSL", 2),
    ("MAD", 2),
    ("MDL", 2),
    ("MGA", 2),
    ("MKD", 2),
    ("MMK", 2),
    ("MNT", 2),
    ("MOP", 2),
    ("MRU", 2),
    ("MUR", 2),
    ("MVR", 2),
    ("MWK", 2),
    ("MXN", 2),
    ("MXV", 2),
    ("MYR", 2),
    ("MZN", 2),
    ("NAD", 2),
    ("NGN", 2),
    ("NIO", 2),
    ("NOK", 2),
    ("NPR", 2),
    ("NZD", 2),
    ("PAB", 2),
    ("PEN", 2),
    ("PGK", 2),
    ("PHP", 2),
    ("PKR", 2),
    ("PLN", 2),
    ("QAR", 2),
    ("RON", 2),
    ("RSD", 2),
    ("RUB", 2),
    ("SAR", 2),
    ("SBD", 2),
    ("SCR", 2),
    ("SDG", 2),
    ("SEK", 2),
    ("SGD", 2),
    ("SHP", 2),
    ("SLE", 2),
    ("SLL", 2),
    ("SOS", 2),
    ("SRD", 2),
    ("SSP", 2),
    ("STN", 2),
    ("SVC", 2),
    ("SYP", 2),
    ("SZL", 2),
    ("THB", 2),
    ("TJS", 2),
    ("TMT", 2),
    ("TOP", 2),
    ("TRY", 2),
    ("TTD", 2),
    ("TWD", 2),
    ("TZS", 2),
    ("UAH", 2),
    ("USD", 2),
    ("USN", 2),
    ("UYU", 2),
    ("UZS", 2),
    ("VED", 2),
    ("VES", 2),
    ("WST", 2),
    ("XCD", 2),
    ("YER", 2),
    ("ZAR", 2),
    ("ZMW", 2),
    ("ZWG", 2),
    // --- exponent 3 ---
    ("BHD", 3),
    ("IQD", 3),
    ("JOD", 3),
    ("KWD", 3),
    ("LYD", 3),
    ("OMR", 3),
    ("TND", 3),
    // --- exponent 4 ---
    ("CLF", 4),
    ("UYW", 4),
    // --- metatokens ---
    ("XAG", 0),
    ("XAU", 0),
    ("XBA", 0),
    ("XBB", 0),
    ("XBC", 0),
    ("XBD", 0),
    ("XDR", 0),
    ("XPD", 0),
    ("XPT", 0),
    ("XSU", 0),
    ("XTS", 0),
    ("XUA", 0),
    ("XXX", 0),
    // --- historical ISO 4217 ---
    ("BYR", 0),
    ("MRO", 1),
    ("STD", 2),
    ("VEF", 2),
    ("VEB", 2),
    ("ZWL", 2),
    ("ZWD", 2),
    // --- documented non-ISO extensions ---
    ("BTC", 8),
    ("TRX", 6),
    ("USDT", 6),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_singleton_is_stable() {
        let a = Repository::shared();
        let b = Repository::shared();
        assert!(std::ptr::eq(a, b));
    }

    #[test]
    fn default_trait_returns_owned_copy() {
        let owned: Repository = Default::default();
        assert_eq!(owned.exponent("USD"), 2);
        assert_eq!(owned.exponent("ZZZ"), 2);
        assert!(owned.knows("BTC"));
    }

    #[test]
    fn known_exponents() {
        let r = Repository::shared();
        assert_eq!(r.exponent("USD"), 2);
        assert_eq!(r.exponent("JPY"), 0);
        assert_eq!(r.exponent("BHD"), 3);
        assert_eq!(r.exponent("UYW"), 4);
        assert_eq!(r.exponent("BTC"), 8);
        assert_eq!(r.exponent("TRX"), 6);
        assert_eq!(r.exponent("USDT"), 6);
    }

    #[test]
    fn case_insensitive_lookup() {
        let r = Repository::shared();
        assert_eq!(r.exponent("usd"), 2);
        assert_eq!(r.exponent("btc"), 8);
        assert!(r.knows("usdt"));
    }

    #[test]
    fn unknown_falls_back() {
        let r = Repository::shared();
        assert_eq!(r.exponent("ZZZ"), 2);
        assert!(!r.knows("ZZZ"));
    }

    #[test]
    fn all_codes_sorted() {
        let r = Repository::shared();
        let codes: Vec<_> = r.all_codes().collect();
        let mut sorted = codes.clone();
        sorted.sort();
        assert_eq!(codes, sorted);
    }

    #[test]
    fn custom_repository() {
        let r = Repository::from_pairs(&[("AAA", 1), ("BBB", 3)], 0);
        assert_eq!(r.exponent("AAA"), 1);
        assert_eq!(r.exponent("BBB"), 3);
        assert_eq!(r.exponent("ZZZ"), 0);
        assert!(r.knows("AAA"));
        assert!(!r.knows("ZZZ"));
    }

    #[test]
    #[should_panic(expected = "invalid code")]
    fn custom_repository_rejects_lowercase() {
        let _ = Repository::from_pairs(&[("usd", 2)], 2);
    }
}
