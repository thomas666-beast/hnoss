//! Parsing options and the parsing algorithm.

use std::str::FromStr;

use crate::currency::Currency;
use crate::error::MoneyError;
use crate::money::Money;

/// Options controlling [`crate::Money::from_str_with`] and
/// [`std::str::FromStr`] for [`crate::Money`].
///
/// All fields have sensible defaults, so most callers use
/// [`ParseOptions::default`] and override only what they need:
///
/// ```
/// use hnoss::ParseOptions;
///
/// let opts = ParseOptions {
///     decimal_mark: ',',
///     thousands_separator: ".".into(),
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseOptions {
    /// Decimal separator. Default: `'.'`.
    ///
    /// Must be exactly one `char`. Multi-byte codepoints are supported.
    pub decimal_mark: char,

    /// Integer grouping separator. Default: `","`.
    ///
    /// Stripped from the input before parsing, but only when the separators
    /// appear in plausible positions (not leading, trailing, or doubled).
    /// Group *positions* are not validated; `"12,34.56"` is accepted.
    pub thousands_separator: String,

    /// Number of digits per integer group. Default: `3`.
    ///
    /// Validated (must be strictly positive) but not used to check group
    /// positions. It exists for symmetry with
    /// [`crate::FormatOptions::group_size`].
    pub group_size: usize,
}

impl Default for ParseOptions {
    fn default() -> Self {
        ParseOptions {
            decimal_mark: '.',
            thousands_separator: ",".into(),
            group_size: 3,
        }
    }
}

pub(crate) fn parse_options(
    input: &str,
    currency: &Currency,
    opts: &ParseOptions,
) -> Result<Money, MoneyError> {
    if opts.group_size == 0 {
        return Err(MoneyError::InvalidAllocation(
            "group_size must be positive".into(),
        ));
    }

    if !opts.thousands_separator.is_empty()
        && opts.thousands_separator.contains(opts.decimal_mark)
    {
        return Err(MoneyError::Parse(
            "decimal_mark and thousands_separator must differ".into(),
        ));
    }

    // Reject separators in obviously bad positions (leading, trailing,
    // doubled). Group *positions* themselves are not validated — "12,34"
    // is accepted, matching the Perl behaviour.
    if !opts.thousands_separator.is_empty() {
        let sep = &opts.thousands_separator;
        if input.starts_with(sep.as_str())
            || input.ends_with(sep.as_str())
            || input.contains(&format!("{sep}{sep}"))
        {
            return Err(MoneyError::Parse(format!(
                "misplaced thousands separator in {input:?}"
            )));
        }
    }

    let working: String = if opts.thousands_separator.is_empty() {
        input.to_string()
    } else {
        input.replace(&opts.thousands_separator, "")
    };

    let bytes = working.as_bytes();
    let mut i = 0;

    let sign: i128 = if i < bytes.len() && (bytes[i] == b'+' || bytes[i] == b'-') {
        let s = if bytes[i] == b'-' { -1 } else { 1 };
        i += 1;
        s
    } else {
        1
    };

    let int_start = i;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    let int_end = i;
    if int_start == int_end {
        return Err(MoneyError::Parse(format!(
            "expected at least one digit in {input:?}"
        )));
    }

    let mut dm_buf = [0u8; 4];
    let dm_bytes = opts.decimal_mark.encode_utf8(&mut dm_buf).as_bytes();

    let frac_start = if i < bytes.len() && bytes[i..].starts_with(dm_bytes) {
        let start = i + dm_bytes.len();
        let mut j = start;
        while j < bytes.len() && bytes[j].is_ascii_digit() {
            j += 1;
        }
        if start == j {
            return Err(MoneyError::Parse(format!(
                "expected at least one digit after decimal mark in {input:?}"
            )));
        }
        i = j;
        start
    } else {
        i
    };
    let frac_end = i;

    if i != bytes.len() {
        return Err(MoneyError::Parse(format!(
            "unexpected trailing characters in {input:?}"
        )));
    }

    let int_str = &working[int_start..int_end];
    let frac_str = &working[frac_start..frac_end];

    let exponent = currency.exponent() as usize;
    if frac_str.len() > exponent {
        return Err(MoneyError::Parse(format!(
            "too many decimal digits for {}: got {:?} (exponent is {})",
            currency.code(),
            frac_str,
            exponent
        )));
    }

    let mut frac_padded = String::with_capacity(exponent);
    frac_padded.push_str(frac_str);
    while frac_padded.len() < exponent {
        frac_padded.push('0');
    }

    let mut combined = String::with_capacity(int_str.len() + exponent);
    combined.push_str(int_str);
    combined.push_str(&frac_padded);

    let mut minor: i128 = combined.parse::<i128>().map_err(|_| MoneyError::Overflow)?;

    if sign < 0 {
        minor = minor.checked_neg().ok_or(MoneyError::Overflow)?;
    }

    Ok(Money::new(minor, currency.clone()))
}

pub(crate) fn parse_with_currency(input: &str) -> Result<Money, MoneyError> {
    let (amount_str, code) = input.rsplit_once(' ').ok_or_else(|| {
        MoneyError::Parse(format!(
            "expected \"<amount> <code>\" (e.g. \"5.00 USD\"), got {input:?}"
        ))
    })?;

    let currency = Currency::new(code)?;
    parse_options(amount_str, &currency, &ParseOptions::default())
}

impl FromStr for Money {
    type Err = MoneyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_with_currency(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn usd() -> Currency {
        Currency::new("USD").unwrap()
    }

    fn jpy() -> Currency {
        Currency::new("JPY").unwrap()
    }

    fn bhd() -> Currency {
        Currency::new("BHD").unwrap()
    }

    #[test]
    fn default_options() {
        let o = ParseOptions::default();
        assert_eq!(o.decimal_mark, '.');
        assert_eq!(o.thousands_separator, ",");
        assert_eq!(o.group_size, 3);
    }

    #[test]
    fn parse_usd_whole_dollars() {
        assert_eq!(
            parse_options("5", &usd(), &ParseOptions::default())
                .unwrap()
                .amount(),
            500
        );
    }

    #[test]
    fn parse_usd_cents() {
        let o = ParseOptions::default();
        assert_eq!(parse_options("5.00", &usd(), &o).unwrap().amount(), 500);
        assert_eq!(parse_options("5.5", &usd(), &o).unwrap().amount(), 550);
        assert_eq!(parse_options("0.05", &usd(), &o).unwrap().amount(), 5);
    }

    #[test]
    fn parse_signs() {
        let o = ParseOptions::default();
        assert_eq!(parse_options("-5", &usd(), &o).unwrap().amount(), -500);
        assert_eq!(parse_options("+5", &usd(), &o).unwrap().amount(), 500);
        assert_eq!(parse_options("-0.05", &usd(), &o).unwrap().amount(), -5);
    }

    #[test]
    fn parse_jpy() {
        assert_eq!(
            parse_options("500", &jpy(), &ParseOptions::default())
                .unwrap()
                .amount(),
            500
        );
    }

    #[test]
    fn parse_bhd() {
        let o = ParseOptions::default();
        assert_eq!(parse_options("0.500", &bhd(), &o).unwrap().amount(), 500);
        assert_eq!(parse_options("1.2", &bhd(), &o).unwrap().amount(), 1200);
    }

    #[test]
    fn parse_with_thousands_separator() {
        assert_eq!(
            parse_options("1,234.56", &usd(), &ParseOptions::default())
                .unwrap()
                .amount(),
            123456
        );
    }

    #[test]
    fn parse_with_loose_grouping() {
        assert_eq!(
            parse_options("12,34.56", &usd(), &ParseOptions::default())
                .unwrap()
                .amount(),
            123456
        );
    }

    #[test]
    fn parse_without_separators() {
        let o = ParseOptions {
            thousands_separator: "".into(),
            ..Default::default()
        };
        assert_eq!(
            parse_options("1234.56", &usd(), &o).unwrap().amount(),
            123456
        );
    }

    #[test]
    fn parse_european() {
        let o = ParseOptions {
            decimal_mark: ',',
            thousands_separator: ".".into(),
            ..Default::default()
        };
        assert_eq!(
            parse_options("1.234,56", &usd(), &o).unwrap().amount(),
            123456
        );
    }

    #[test]
    fn parse_empty_errors() {
        assert!(matches!(
            parse_options("", &usd(), &ParseOptions::default()),
            Err(MoneyError::Parse(_))
        ));
    }

    #[test]
    fn parse_bad_grammar_errors() {
        let o = ParseOptions::default();
        for bad in [
            "abc", "5.", ".5", "5e3", "5 5", "--5", "-", "+", "5..5", "5,", ",5", "5,,6",
            ",5,6", "5,6,", ",,5",
        ] {
            assert!(
                matches!(parse_options(bad, &usd(), &o), Err(MoneyError::Parse(_))),
                "expected {bad:?} to fail"
            );
        }
    }

    #[test]
    fn parse_too_many_decimals_errors() {
        let o = ParseOptions::default();
        assert!(matches!(
            parse_options("5.001", &usd(), &o),
            Err(MoneyError::Parse(_))
        ));
        assert!(matches!(
            parse_options("5.5", &jpy(), &o),
            Err(MoneyError::Parse(_))
        ));
        assert!(matches!(
            parse_options("0.0001", &bhd(), &o),
            Err(MoneyError::Parse(_))
        ));
    }

    #[test]
    fn parse_group_size_zero_errors() {
        let o = ParseOptions {
            group_size: 0,
            ..Default::default()
        };
        assert!(matches!(
            parse_options("5", &usd(), &o),
            Err(MoneyError::InvalidAllocation(_))
        ));
    }

    #[test]
    fn parse_identical_separators_errors() {
        let o = ParseOptions {
            decimal_mark: ',',
            thousands_separator: ",".into(),
            ..Default::default()
        };
        assert!(matches!(
            parse_options("5,00", &usd(), &o),
            Err(MoneyError::Parse(_))
        ));
    }

    #[test]
    fn from_str_basic() {
        let m: Money = "5.00 USD".parse().unwrap();
        assert_eq!(m.amount(), 500);
        assert_eq!(m.currency().code(), "USD");
    }

    #[test]
    fn from_str_negative() {
        let m: Money = "-0.05 USD".parse().unwrap();
        assert_eq!(m.amount(), -5);
    }

    #[test]
    fn from_str_missing_currency_errors() {
        assert!("5.00".parse::<Money>().is_err());
    }

    #[test]
    fn from_str_bad_currency_errors() {
        assert!("5.00 U5D".parse::<Money>().is_err());
        assert!("5.00 TOOLONGXX".parse::<Money>().is_err());
    }

    #[test]
    fn from_str_roundtrip() {
        let m = Money::new(123456, usd());
        let s = m.as_string();
        let parsed: Money = s.parse().unwrap();
        assert_eq!(parsed.currency().code(), "USD");
        assert_eq!(parsed.amount(), 12345600);
    }

    #[test]
    fn from_str_format_roundtrip() {
        let m = Money::new(123456, usd());
        let s = m.format();
        let parsed: Money = s.parse().unwrap();
        assert_eq!(parsed, m);
    }
}
