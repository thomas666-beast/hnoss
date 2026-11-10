//! Formatting options and the formatting algorithm.

use std::fmt::Write as _;

use crate::currency::Currency;
use crate::error::MoneyError;

/// Options controlling [`crate::Money::format_with`].
///
/// All fields have sensible defaults, so most callers use
/// [`FormatOptions::default`] and override only what they need:
///
/// ```
/// use hnoss::FormatOptions;
///
/// let opts = FormatOptions {
///     decimal_mark: ',',
///     thousands_separator: ".".into(),
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormatOptions {
    /// Whether to append the currency code. Default: `true`.
    ///
    /// When `true`, the output looks like `"5.00 USD"`.
    /// When `false`, the output looks like `"5.00"`.
    pub with_currency: bool,

    /// String placed between the amount and the currency code.
    /// Default: `" "` (single space).
    ///
    /// Used only when `with_currency` is `true`.
    pub currency_separator: String,

    /// Decimal separator. Default: `'.'`.
    ///
    /// Must be exactly one `char`. Multi-byte codepoints are supported
    /// (e.g. U+066B ARABIC DECIMAL SEPARATOR).
    pub decimal_mark: char,

    /// Integer grouping separator. Default: `","`.
    ///
    /// Pass an empty string to disable grouping.
    pub thousands_separator: String,

    /// Number of digits per integer group. Default: `3`.
    ///
    /// Must be strictly positive. `format_with` returns
    /// [`MoneyError::InvalidAllocation`] when `group_size == 0` (the variant
    /// is reused from allocation because both are "bad caller input"
    /// conditions; a dedicated `InvalidFormat` variant can be added later
    /// if it becomes useful to distinguish them).
    pub group_size: usize,
}

impl Default for FormatOptions {
    fn default() -> Self {
        FormatOptions {
            with_currency: true,
            currency_separator: " ".into(),
            decimal_mark: '.',
            thousands_separator: ",".into(),
            group_size: 3,
        }
    }
}

/// Renders `amount` (in minor units) plus `currency` according to `opts`.
///
/// This is the pure, testable core of formatting. [`crate::Money::format`]
/// and [`crate::Money::format_with`] delegate to it.
pub(crate) fn render_options(
    amount: i128,
    currency: &Currency,
    opts: &FormatOptions,
) -> Result<String, MoneyError> {
    if opts.group_size == 0 {
        return Err(MoneyError::InvalidAllocation(
            "group_size must be positive".into(),
        ));
    }

    let exponent = currency.exponent() as u32;

    // Split into sign, integer part, and fractional part.
    let (sign, abs) = if amount < 0 {
        ("-", amount.unsigned_abs())
    } else {
        ("", amount.unsigned_abs())
    };

    let (int_part, frac_part) = if exponent == 0 {
        (abs.to_string(), String::new())
    } else {
        let divisor: u128 = 10u128.pow(exponent);
        let int = abs / divisor;
        let frac = abs % divisor;
        (
            int.to_string(),
            format!("{:0width$}", frac, width = exponent as usize),
        )
    };

    // Insert grouping into the integer part.
    let int_grouped = group_digits(&int_part, opts.group_size, &opts.thousands_separator);

    // Assemble.
    let mut out =
        String::with_capacity(sign.len() + int_grouped.len() + 1 + exponent as usize + 4);
    out.push_str(sign);
    out.push_str(&int_grouped);
    if exponent > 0 {
        out.push(opts.decimal_mark);
        out.push_str(&frac_part);
    }
    if opts.with_currency {
        out.push_str(&opts.currency_separator);
        write!(out, "{currency}").unwrap();
    }

    Ok(out)
}

fn group_digits(int_part: &str, group_size: usize, sep: &str) -> String {
    if sep.is_empty() || int_part.len() <= group_size {
        return int_part.to_string();
    }
    let mut groups: Vec<&str> = Vec::new();
    let mut rest = int_part;
    while rest.len() > group_size {
        let (head, tail) = rest.split_at(rest.len() - group_size);
        groups.push(tail);
        rest = head;
    }
    groups.push(rest);
    groups.reverse();
    groups.join(sep)
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

    // --- defaults -----------------------------------------------------------

    #[test]
    fn default_options() {
        let o = FormatOptions::default();
        assert!(o.with_currency);
        assert_eq!(o.currency_separator, " ");
        assert_eq!(o.decimal_mark, '.');
        assert_eq!(o.thousands_separator, ",");
        assert_eq!(o.group_size, 3);
    }

    // --- simple cases -------------------------------------------------------

    #[test]
    fn usd_basic() {
        assert_eq!(
            render_options(500, &usd(), &FormatOptions::default()).unwrap(),
            "5.00 USD"
        );
        assert_eq!(
            render_options(0, &usd(), &FormatOptions::default()).unwrap(),
            "0.00 USD"
        );
        assert_eq!(
            render_options(-5, &usd(), &FormatOptions::default()).unwrap(),
            "-0.05 USD"
        );
    }

    #[test]
    fn jpy_no_decimals() {
        assert_eq!(
            render_options(500, &jpy(), &FormatOptions::default()).unwrap(),
            "500 JPY"
        );
        assert_eq!(
            render_options(-500, &jpy(), &FormatOptions::default()).unwrap(),
            "-500 JPY"
        );
    }

    #[test]
    fn bhd_three_decimals() {
        assert_eq!(
            render_options(500, &bhd(), &FormatOptions::default()).unwrap(),
            "0.500 BHD"
        );
        assert_eq!(
            render_options(1, &bhd(), &FormatOptions::default()).unwrap(),
            "0.001 BHD"
        );
        assert_eq!(
            render_options(1234, &bhd(), &FormatOptions::default()).unwrap(),
            "1.234 BHD"
        );
    }

    // --- grouping -----------------------------------------------------------

    #[test]
    fn default_grouping() {
        assert_eq!(
            render_options(123456, &usd(), &FormatOptions::default()).unwrap(),
            "1,234.56 USD"
        );
        assert_eq!(
            render_options(1234567890, &usd(), &FormatOptions::default()).unwrap(),
            "12,345,678.90 USD"
        );
    }

    #[test]
    fn grouping_disabled() {
        let o = FormatOptions {
            thousands_separator: "".into(),
            ..Default::default()
        };
        assert_eq!(render_options(123456, &usd(), &o).unwrap(), "1234.56 USD");
    }

    #[test]
    fn group_size_custom() {
        let o = FormatOptions {
            group_size: 2,
            ..Default::default()
        };
        assert_eq!(
            render_options(123456789, &usd(), &o).unwrap(),
            "1,23,45,67.89 USD"
        );
    }

    #[test]
    fn grouping_small_numbers() {
        // No grouping needed for numbers with <= group_size digits.
        assert_eq!(
            render_options(999, &usd(), &FormatOptions::default()).unwrap(),
            "9.99 USD"
        );
        assert_eq!(
            render_options(1000, &usd(), &FormatOptions::default()).unwrap(),
            "10.00 USD"
        );
    }

    // --- separators ---------------------------------------------------------

    #[test]
    fn european_style() {
        let o = FormatOptions {
            decimal_mark: ',',
            thousands_separator: ".".into(),
            ..Default::default()
        };
        assert_eq!(render_options(123456, &usd(), &o).unwrap(), "1.234,56 USD");
    }

    #[test]
    fn only_decimal_mark_changed() {
        let o = FormatOptions {
            decimal_mark: ',',
            ..Default::default()
        };
        assert_eq!(render_options(123456, &usd(), &o).unwrap(), "1,234,56 USD");
    }

    // --- currency toggling --------------------------------------------------

    #[test]
    fn with_currency_false() {
        let o = FormatOptions {
            with_currency: false,
            ..Default::default()
        };
        assert_eq!(render_options(500, &usd(), &o).unwrap(), "5.00");
    }

    #[test]
    fn custom_currency_separator() {
        let o = FormatOptions {
            currency_separator: "".into(),
            ..Default::default()
        };
        assert_eq!(render_options(500, &usd(), &o).unwrap(), "5.00USD");
    }

    // --- errors -------------------------------------------------------------

    #[test]
    fn group_size_zero_errors() {
        let o = FormatOptions {
            group_size: 0,
            ..Default::default()
        };
        assert!(matches!(
            render_options(500, &usd(), &o),
            Err(MoneyError::InvalidAllocation(_))
        ));
    }

    // --- extremes -----------------------------------------------------------

    #[test]
    fn i128_max_usd() {
        // i128::MAX cents. Just verify it renders without panicking and
        // contains the expected sign and currency.
        let s = render_options(i128::MAX, &usd(), &FormatOptions::default()).unwrap();
        assert!(s.ends_with(" USD"), "{s}");
        assert!(!s.starts_with('-'), "{s}");
    }

    #[test]
    fn i128_min_usd() {
        let s = render_options(i128::MIN, &usd(), &FormatOptions::default()).unwrap();
        assert!(s.ends_with(" USD"), "{s}");
        assert!(s.starts_with('-'), "{s}");
    }

    #[test]
    fn i128_min_jpy() {
        let s = render_options(i128::MIN, &jpy(), &FormatOptions::default()).unwrap();
        assert!(s.starts_with('-'), "{s}");
    }
}
