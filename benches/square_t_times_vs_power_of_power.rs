use std::iter::repeat_with;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use rug::{rand::RandState, Integer};

fn square_t_timers(t: u32, n: &Integer) -> Integer {
    let mut res = Integer::from(2);
    for _ in 0..t {
        res = res.square() % n;
    }
    res
}

fn power_of_power(t: u32, n: &Integer, phi_n: &Integer) -> Integer {
    let b = Integer::from(2).pow_mod(&Integer::from(t), phi_n).unwrap();
    Integer::from(2).pow_mod(&b, n).unwrap()
}

fn generate_prime(rand: &mut RandState) -> Integer {
    let mut p = Integer::from(Integer::random_bits(3072, rand));
    p |= Integer::from(1) << 3071;
    p.next_prime()
}

fn criterion_benchmark(c: &mut Criterion) {
    const TS: [u32; 10] = [
        1 * 1000,
        2 * 1000,
        5 * 1000,
        10 * 1000,
        20 * 1000,
        40 * 1000,
        60 * 1000,
        80 * 1000,
        100 * 1000,
        120 * 1000,
    ];
    let mut rand = RandState::new();
    let (p, q) = (generate_prime(&mut rand), generate_prime(&mut rand));
    // println!("p = {p}");
    // println!("q = {q}");
    let n = Integer::from(&p * &q);
    let phi_n = (p - 1) * (q - 1);
    assert_eq!(
        square_t_timers(120 * 1000, &n),
        power_of_power(120 * 1000, &n, &phi_n),
        "sanity check failed"
    );

    const NUM_PRIME: usize = 100;
    let primes = repeat_with(|| generate_prime(&mut rand))
        .take(NUM_PRIME)
        .collect::<Vec<_>>();
    let (n, phi_n) = primes
        .iter()
        .zip(primes.iter().skip(1))
        .map(|(p, q)| {
            let (p, q) = (Integer::from(p), Integer::from(q));
            (Integer::from(&p * &q), (p - 1) * (q - 1))
        })
        .unzip::<_, _, Vec<_>, Vec<_>>();

    let mut group = c.benchmark_group("timelock");
    for t in &TS {
        let mut i = 0;
        group.bench_with_input(BenchmarkId::new("square t times", t), t, |b, &t| {
            b.iter(|| {
                i = (i + 1) % (NUM_PRIME - 1);
                square_t_timers(black_box(t), &n[i])
            })
        });
        group.bench_with_input(BenchmarkId::new("power of power", t), t, |b, &t| {
            b.iter(|| {
                i = (i + 1) % (NUM_PRIME - 1);
                power_of_power(black_box(t), &n[i], &phi_n[i])
            })
        });
    }
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
