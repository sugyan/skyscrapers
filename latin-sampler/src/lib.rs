#![doc = include_str!("../README.md")]

mod jacobson_matthews;
mod sampler;
mod square;

pub use sampler::{Sampler, SamplerParams, sample};
pub use square::LatinSquare;
