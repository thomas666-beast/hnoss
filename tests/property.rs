//! Property-based tests using a hand-rolled deterministic PRNG.
//!
//! This file deliberately avoids `proptest` / `quickcheck` to keep the
//! crate at zero dependencies. The PRNG is a 64-bit linear congruential
//! generator (Numerical Recipes constants). It's not cryptographic, but
//! it's good enough for finding edge cases in our arithmetic.

use hnoss::{Money, MoneyError};

const A: u64 = 6364136223846793005;
const C: u64 = 1442695040888963407;

struct Lcg(u64);

impl Lcg {
    fn new(seed: u64) -> Self {
        Lcg(seed)
    }

    fn next_u64(&mut self) -> u64 {
        self.0 = A.wrapping_mul(self.0).wrapping_add(C);
        self.0
    }

    /// Uniform-ish value in `[lo, hi)`. `lo < hi` must hold.
    fn range(&mut self, lo: i64, hi: i64) -> i64 {
        debug_assert!(lo < hi);
        let span = (hi - lo) as u64;
        lo + (self.next_u64() % span) as i64
    }

    /// A "small" amount, biased toward interesting values (0, ±1, ±100).
    fn interesting_amount(&mut self) -> i128 {
        match self.next_u64() % 8 {
            0 => 0,
            1 => 1,
            2 => -1,
            3 => 100,
            4 => -100,
            5 => self.range(-1_000_000, 1_000_001) as i128,
            6 => self.range(-1_000_000_000_000, 1_000_000_000_001) as i128,
            _ => self.range(-1_000_000, 1_000_001) as i128,
        }
    }

    /// A "reasonable" nonzero divisor.
    fn interesting_divisor(&mut self) -> i128 {
        loop {
            let d = self.range(-1_000, 1_001) as i128;
            if d != 0 {
                return d;
            }
        }
    }

    /// A small ratio vector.
    fn ratios(&mut self) -> Vec<u128> {
        let n = (self.next_u64() % 5) as usize + 1; // 1..=5
        (0..n)
            .map(|_| (self.next_u64() % 20) as u128) // 0..=19
            .collect()
    }
}

fn usd(amount: i128) -> Money {
    Money::from_code(amount, "USD").unwrap()
}

// --- allocate invariants -----------------------------------------------------

#[test]
fn allocate_sum_equals_original_property() {
    let mut rng = Lcg::new(0xDEADBEEF);
    let mut tested = 0;

    for _ in 0..10_000 {
        let amount = rng.interesting_amount();
        let ratios = rng.ratios();
        if ratios.iter().all(|&r| r == 0) {
            continue; // invalid input; expect error, tested separately
        }
        let m = usd(amount);
        let parts = m.allocate(&ratios).unwrap();
        let sum = parts.iter().fold(usd(0), |a, b| a.add(b).unwrap());
        assert_eq!(sum.amount(), amount, "amount={amount} ratios={ratios:?}");
        assert_eq!(parts.len(), ratios.len());
        tested += 1;
    }
    assert!(tested > 9_000, "only tested {tested} valid cases");
}

#[test]
fn allocate_mirror_property() {
    let mut rng = Lcg::new(0xCAFEBABE);
    for _ in 0..5_000 {
        let amount = rng.interesting_amount();
        let ratios = rng.ratios();
        if amount == 0 || ratios.iter().all(|&r| r == 0) {
            continue;
        }
        let pos = usd(amount.abs()).allocate(&ratios).unwrap();
        let neg = usd(-amount.abs()).allocate(&ratios).unwrap();
        for (p, n) in pos.iter().zip(neg.iter()) {
            assert_eq!(
                p.amount(),
                -n.amount(),
                "mirror failed: amount={amount} ratios={ratios:?}"
            );
        }
    }
}

#[test]
fn allocate_bad_input_property() {
    // Empty and all-zero ratios must error, never panic.
    let m = usd(100);
    assert!(matches!(
        m.allocate(&[]),
        Err(MoneyError::InvalidAllocation(_))
    ));
    assert!(matches!(
        m.allocate(&[0, 0, 0]),
        Err(MoneyError::InvalidAllocation(_))
    ));
}

// --- divide_with_remainder invariants ---------------------------------------

#[test]
fn divide_with_remainder_invariant_property() {
    let mut rng = Lcg::new(0x1234_5678_9ABC_DEF0);
    for _ in 0..10_000 {
        let dividend = rng.interesting_amount();
        let divisor = rng.interesting_divisor();
        let m = usd(dividend);
        let (q, r) = m.divide_with_remainder(divisor).unwrap();

        // q * divisor + r == dividend
        let back = q.multiply(divisor).unwrap().add(&r).unwrap();
        assert_eq!(
            back.amount(),
            dividend,
            "amount={dividend} divisor={divisor} q={} r={}",
            q.amount(),
            r.amount()
        );

        // r has divisor's sign (or is zero)
        assert!(
            r.amount() == 0 || (r.amount() < 0) == (divisor < 0),
            "amount={dividend} divisor={divisor} r={}",
            r.amount()
        );

        // |r| < |divisor|
        assert!(
            r.amount().unsigned_abs() < divisor.unsigned_abs(),
            "amount={dividend} divisor={divisor} r={}",
            r.amount()
        );
    }
}

// --- format / parse round-trip ----------------------------------------------

#[test]
fn format_then_parse_roundtrip_property() {
    let mut rng = Lcg::new(0xFEEDFACE);
    let currencies = ["USD", "EUR", "JPY", "BHD", "BTC", "USDT", "TRX"];
    for _ in 0..5_000 {
        let amount = rng.interesting_amount();
        let code = currencies[(rng.next_u64() as usize) % currencies.len()];
        let m = Money::from_code(amount, code).unwrap();
        let text = m.format();
        let parsed: Money = text.parse().unwrap();
        assert_eq!(parsed, m, "roundtrip failed: {text} (code={code})");
    }
}

// --- add/subtract associativity ---------------------------------------------

#[test]
fn addition_associativity_property() {
    let mut rng = Lcg::new(0x5555_5555_5555_5555);
    for _ in 0..5_000 {
        // Small amounts so sums don't overflow.
        let a = (rng.range(-1_000_000, 1_000_000) as i128) * 100;
        let b = (rng.range(-1_000_000, 1_000_000) as i128) * 100;
        let c = (rng.range(-1_000_000, 1_000_000) as i128) * 100;

        let left = usd(a).add(&usd(b)).unwrap().add(&usd(c)).unwrap();
        let right = usd(a).add(&usd(b).add(&usd(c)).unwrap()).unwrap();
        assert_eq!(
            left.amount(),
            right.amount(),
            "(a+b)+c == a+(b+c) failed: a={a} b={b} c={c}"
        );
    }
}

#[test]
fn subtraction_inverse_of_addition_property() {
    let mut rng = Lcg::new(0xAAAA_BBBB_CCCC_DDDD);
    for _ in 0..5_000 {
        let a = rng.range(-1_000_000, 1_000_000) as i128;
        let b = rng.range(-1_000_000, 1_000_000) as i128;

        let sum = usd(a).add(&usd(b)).unwrap();
        let back = sum.subtract(&usd(b)).unwrap();
        assert_eq!(back.amount(), a, "(a + b) - b == a failed: a={a} b={b}");
    }
}
