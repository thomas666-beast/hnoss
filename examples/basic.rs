//! Basic usage of `hnoss`: construction, arithmetic, formatting, parsing.
//!
//! Run with: `cargo run --example basic`

use hnoss::{Currency, Money, ParseOptions};

fn main() {
    // Construct from minor units (cents for USD).
    let price = Money::from_code(500, "USD").unwrap(); // $5.00
    let tax = Money::from_code(50, "USD").unwrap(); // $0.50

    // Exact arithmetic.
    let total = price.add(&tax).unwrap();
    println!("Price:  {}", price.format()); // 5.00 USD
    println!("Tax:    {}", tax.format()); // 0.50 USD
    println!("Total:  {}", total.format()); // 5.50 USD

    // Canonical form vs display form.
    println!("Canonical: {}", total.as_string()); // 550 USD
    println!("Display:   {}", total.format()); // 5.50 USD

    // Parse from major-unit text.
    let parsed: Money = "1,234.56 USD".parse().unwrap();
    println!("Parsed:  {}", parsed.format()); // 1,234.56 USD

    // Parse with an explicit currency and custom separators (European).
    let usd = Currency::new("USD").unwrap();
    let _ = Money::from_str_with(
        "1.234,56",
        &usd,
        &ParseOptions {
            decimal_mark: ',',
            thousands_separator: ".".into(),
            ..Default::default()
        },
    )
    .unwrap();

    // Divide, with remainder preserved.
    let (q, r) = Money::from_code(500, "USD")
        .unwrap()
        .divide_with_remainder(3)
        .unwrap();
    println!("500 / 3 = {} with remainder {}", q.format(), r.format());
}
