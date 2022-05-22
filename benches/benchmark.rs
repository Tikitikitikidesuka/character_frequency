use std::fs;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use character_frequency::*;

fn character_frequency_benchmark(c: &mut Criterion) {
    let text = fs::read_to_string("benches/bench_text.txt").expect("File not found");
    c.bench_function("sequential", |b| b.iter(|| sequential_character_frequencies(black_box(&text))));
    c.bench_function("concurrent", |b| b.iter(|| character_frequencies(black_box(&text))));
}

criterion_group!(benches, character_frequency_benchmark);
criterion_main!(benches);