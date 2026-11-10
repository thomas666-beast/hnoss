//! Splitting a bill without losing a cent.
//!
//! Run with: `cargo run --example split_bill`

use hnoss::Money;

fn main() {
    let total = Money::from_code(10_001, "USD").unwrap(); // $100.01
    let people = ["alice", "bob", "carol", "dave"];

    let ratios: Vec<u128> = (0..people.len()).map(|_| 1).collect();
    let shares = total.allocate(&ratios).unwrap();

    println!("Total: {}", total.format()); // 100.01 USD
    for (person, share) in people.iter().zip(shares.iter()) {
        println!("  {} pays {}", person, share.format());
    }

    let sum = shares
        .iter()
        .fold(Money::from_code(0, "USD").unwrap(), |a, b| {
            a.add(b).unwrap()
        });
    println!("Sum of shares: {}", sum.format()); // 100.01 USD
}
