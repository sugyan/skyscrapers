#![doc = include_str!("../README.md")]

mod moves;
mod sampler;
mod square;

pub use sampler::{SamplerParams, sample};
pub use square::LatinSquare;
