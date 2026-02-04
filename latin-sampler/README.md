# latin-sampler

MCMC sampler for generating approximately uniform Latin squares.

## Algorithm

Uses the [Jacobson-Matthews algorithm](https://doi.org/10.1002/(SICI)1520-6610(1996)4:6<405::AID-JCD3>3.0.CO;2-J) (1996), proven to be **ergodic** — any Latin square can be reached from any starting state, guaranteeing the sampler explores the full space.

Reference: Jacobson & Matthews, "Generating uniformly distributed random Latin squares", *J. Combinatorial Designs* 4(6), 1996.

## Example

```rust
use latin_sampler::{Sampler, SamplerParams};
use rand_chacha::ChaCha20Rng;
use rand::SeedableRng;

let rng = ChaCha20Rng::seed_from_u64(42);
let mut sampler = Sampler::new(8, rng, SamplerParams::default());

let sq = sampler.next().unwrap();
for r in 0..sq.n() {
    for c in 0..sq.n() {
        print!("{} ", sq.get(r, c));
    }
    println!();
}
// Output (seed=42):
// 0 2 4 3 6 7 5 1
// 1 5 2 7 0 4 6 3
// 6 7 3 4 1 5 0 2
// 2 6 0 5 4 3 1 7
// 3 4 6 0 7 1 2 5
// 7 1 5 2 3 0 4 6
// 5 0 7 1 2 6 3 4
// 4 3 1 6 5 2 7 0
```

Or from command line: `cargo run --release --example generate -- 8 42`

## Default Parameters

| Parameter | Default | Description |
|-----------|---------|-------------|
| burn_in | n^3 | Steps to reach equilibrium |
| thinning | 3*n^2 | Steps between samples for independence |
| p_do_nothing | 0.01 | Probability of null move (aperiodicity) |

## Uniformity Verification

**Question: Does the sampler produce all Latin squares with equal probability?**

Chi-square goodness-of-fit test confirms uniform distribution. χ²/df ≈ 1.0 means uniform; values < 1.2 are acceptable.

| n | Iterator | One-shot |
|---|----------|----------|
| 4 | 1.03 | 0.94 |
| 5 | 1.03 | 1.01 |
| 6 | 1.00 | 1.00 |
| 7 | 1.01 | 0.99 |
| 8 | 1.02 | 0.99 |

**Sampling modes:**
- **Iterator** (`Sampler`): Burn-in once, then thinning steps between samples. Efficient for many samples.
- **One-shot** (`sample()`): Fresh burn-in with each seed. Fully independent samples.

Verify with: `cargo run --release --example uniformity_test -- <n> [--light] [--oneshot]`

## Independence Verification (Iterator mode)

**Question: Are consecutive samples from `Sampler` independent of each other?**

With default thinning (3n²), consecutive samples show negligible correlation:

| n | thinning | tau | \|rho1\| |
|---|----------|-----|----------|
| 5 | 75 | ~1.03 | ~0.01 |
| 7 | 147 | ~1.04 | ~0.01 |
| 9 | 243 | ~1.03 | ~0.01 |
| 11 | 363 | ~1.04 | ~0.01 |
| 15 | 675 | ~1.04 | ~0.01 |
| 20 | 1200 | ~1.04 | ~0.01 |
| 26 | 2028 | ~1.05 | ~0.01 |

- **tau** (integrated autocorrelation time): tau ≈ 1.0 means each sample counts as ~1 independent sample.
- **|rho1|** (lag-1 autocorrelation): |rho1| < 0.05 means consecutive samples are nearly uncorrelated.

Verify with: `cargo run --release --example independence_check`

## Notes

- Output is deterministic given the same seed and parameters
- Default burn_in (n³) and thinning (3n²) are empirically validated for n ≤ 26
- For custom thinning: `SamplerParams { thinning: Some(100), ..Default::default() }`
