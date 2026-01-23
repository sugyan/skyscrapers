# latin-sampler

MCMC sampler for generating approximately uniform Latin squares.

## Example

```rust
use latin_sampler::{sample, SamplerParams};
use rand_chacha::ChaCha20Rng;
use rand::SeedableRng;

let mut rng = ChaCha20Rng::from_seed([0u8; 32]);
let params = SamplerParams::default();
let sq = sample(8, &mut rng, &params);
println!("Cell (0,0) = {}", sq.get(0, 0));
```

The output is deterministic given the same seed and parameters.

## Notes

- Output is "approximately uniform" after sufficient burn-in and mixing
- Default parameters are tuned for `n=7` or `n=8`
- For strict statistical needs, parameters may require adjustment
