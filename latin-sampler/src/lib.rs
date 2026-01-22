//! # latin-sampler
//!
//! MCMC sampler for generating approximately uniform Latin squares.
//!
//! ## Example
//!
//! ```
//! use latin_sampler::{sample, SamplerParams};
//! use rand_chacha::ChaCha20Rng;
//! use rand::SeedableRng;
//!
//! let mut rng = ChaCha20Rng::from_seed([0u8; 32]);
//! let params = SamplerParams::default();
//! let sq = sample(8, &mut rng, &params);
//! assert!(sq.is_latin());
//! ```

mod moves;
mod sampler;
mod square;

pub use sampler::{sample, SamplerParams};
pub use square::LatinSquare;
