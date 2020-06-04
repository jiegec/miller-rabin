use criterion::{criterion_group, criterion_main, Criterion};
use miller_rabin::miller_rabin;
use num_bigint::*;

fn benchmark_prime_128(c: &mut Criterion) {
    // 128
    let bytes = include_bytes!("../prime_128");
    let prime = BigUint::parse_bytes(&bytes[..bytes.len() - 1], 10).unwrap();
    c.bench_function("miller rabin 128 prime 10 times", |b| {
        b.iter(|| {
            miller_rabin(&prime, 10);
        });
    });
    c.bench_function("miller rabin 128 prime 100 times", |b| {
        b.iter(|| {
            miller_rabin(&prime, 100);
        });
    });
}

criterion_group!(benches, benchmark_prime_128);
criterion_main!(benches);
