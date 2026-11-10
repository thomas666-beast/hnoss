//! Working with the crypto extensions: BTC, TRX, USDT.
//!
//! Run with: `cargo run --example crypto`

use hnoss::Money;

fn main() {
    // BTC has exponent 8 (satoshi).
    let one_btc = Money::from_code(100_000_000, "BTC").unwrap();
    println!("1 BTC (as satoshi):  {}", one_btc.format()); // 1.00000000 BTC
    println!("1 BTC (canonical):   {}", one_btc.as_string()); // 100000000 BTC

    let five_hundred_sat = Money::from_code(500, "BTC").unwrap();
    println!("500 sat:             {}", five_hundred_sat.format()); // 0.00000500 BTC

    // TRX has exponent 6 (sun).
    let one_trx = Money::from_code(1_000_000, "TRX").unwrap();
    println!("1 TRX:               {}", one_trx.format()); // 1.000000 TRX

    // USDT has exponent 6 on both TRON (TRC20) and Ethereum (ERC20).
    let usdt: Money = "100.000000 USDT".parse().unwrap();
    println!("100 USDT:            {}", usdt.format()); // 100.000000 USDT

    // Parsing BTC from major units:
    let parsed: Money = "0.00000500 BTC".parse().unwrap();
    println!("0.00000500 BTC =     {} satoshi", parsed.amount()); // 500
}
