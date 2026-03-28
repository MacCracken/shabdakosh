//! Criterion benchmarks for shabdakosh dictionary operations.

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use shabdakosh::PronunciationDict;

fn bench_dict_construction(c: &mut Criterion) {
    c.bench_function("dict_english_construction", |b| {
        b.iter(|| {
            let dict = PronunciationDict::english();
            black_box(dict);
        });
    });
}

fn bench_dict_lookup_hit(c: &mut Criterion) {
    c.bench_function("dict_lookup_hit", |b| {
        let dict = PronunciationDict::english();
        let words = ["the", "beautiful", "psychology", "computer", "knight", "enough"];
        b.iter(|| {
            for word in &words {
                black_box(dict.lookup(word));
            }
        });
    });
}

fn bench_dict_lookup_miss(c: &mut Criterion) {
    c.bench_function("dict_lookup_miss", |b| {
        let dict = PronunciationDict::english();
        b.iter(|| {
            black_box(dict.lookup("zxqvbnm"));
        });
    });
}

criterion_group!(
    benches,
    bench_dict_construction,
    bench_dict_lookup_hit,
    bench_dict_lookup_miss,
);

criterion_main!(benches);
