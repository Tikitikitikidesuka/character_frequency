use character_frequency::*;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::fs;

fn character_frequency_benchmark(c: &mut Criterion) {
	let filename = "benches/bench_text.txt";
    let text = fs::read_to_string(filename).expect(&format!("File not found: {}",filename));
    c.bench_function("sequential", |b| {
        b.iter(|| sequential_character_frequencies(black_box(&text)))
    });
    c.bench_function("concurrent", |b| {
        b.iter(|| character_frequencies(black_box(&text)))
    });
}

criterion_group!(benches, character_frequency_benchmark);
criterion_main!(benches);
