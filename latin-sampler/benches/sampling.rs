#![feature(test)]

extern crate test;

use latin_sampler::{SamplerParams, sample};
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use test::Bencher;

#[bench]
fn bench_sample_n4(b: &mut Bencher) {
    let params = SamplerParams::default();
    let mut rng = ChaCha20Rng::seed_from_u64(42);

    b.iter(|| {
        let sq = sample(4, &mut rng, &params);
        test::black_box(sq)
    });
}

#[bench]
fn bench_sample_n7(b: &mut Bencher) {
    let params = SamplerParams::default();
    let mut rng = ChaCha20Rng::seed_from_u64(42);

    b.iter(|| {
        let sq = sample(7, &mut rng, &params);
        test::black_box(sq)
    });
}

#[bench]
fn bench_sample_n10(b: &mut Bencher) {
    let params = SamplerParams::default();
    let mut rng = ChaCha20Rng::seed_from_u64(42);

    b.iter(|| {
        let sq = sample(10, &mut rng, &params);
        test::black_box(sq)
    });
}
