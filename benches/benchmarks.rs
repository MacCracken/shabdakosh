//! Criterion benchmarks for shabdakosh dictionary operations.

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use shabdakosh::PrefixTrie;
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
        let words = [
            "the",
            "beautiful",
            "psychology",
            "computer",
            "knight",
            "enough",
        ];
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

fn bench_trie_construction(c: &mut Criterion) {
    c.bench_function("trie_construction", |b| {
        let dict = PronunciationDict::english();
        b.iter(|| {
            let trie = PrefixTrie::from_dict(&dict);
            black_box(trie);
        });
    });
}

fn bench_trie_prefix_search(c: &mut Criterion) {
    c.bench_function("trie_prefix_search", |b| {
        let dict = PronunciationDict::english();
        let trie = PrefixTrie::from_dict(&dict);
        b.iter(|| {
            black_box(trie.search_prefix("comp"));
            black_box(trie.search_prefix("th"));
            black_box(trie.search_prefix("psych"));
        });
    });
}

fn bench_binary_roundtrip(c: &mut Criterion) {
    #[cfg(feature = "binary")]
    {
        use shabdakosh::dictionary::format::binary;

        c.bench_function("binary_serialize", |b| {
            let dict = PronunciationDict::english();
            b.iter(|| {
                black_box(binary::to_binary(&dict).unwrap());
            });
        });

        c.bench_function("binary_deserialize", |b| {
            let dict = PronunciationDict::english();
            let data = binary::to_binary(&dict).unwrap();
            b.iter(|| {
                black_box(binary::from_binary(&data).unwrap());
            });
        });
    }
    #[cfg(not(feature = "binary"))]
    {
        let _ = c;
    }
}

fn bench_phf_lookup(c: &mut Criterion) {
    #[cfg(feature = "phf")]
    {
        use shabdakosh::dictionary::static_dict;

        c.bench_function("phf_lookup_hit", |b| {
            let words = [
                "the",
                "beautiful",
                "psychology",
                "computer",
                "knight",
                "enough",
            ];
            b.iter(|| {
                for word in &words {
                    black_box(static_dict::lookup(word));
                }
            });
        });

        c.bench_function("phf_lookup_miss", |b| {
            b.iter(|| {
                black_box(static_dict::lookup("zxqvbnm"));
            });
        });
    }
    #[cfg(not(feature = "phf"))]
    {
        let _ = c;
    }
}

criterion_group!(
    benches,
    bench_dict_construction,
    bench_dict_lookup_hit,
    bench_dict_lookup_miss,
    bench_trie_construction,
    bench_trie_prefix_search,
    bench_binary_roundtrip,
    bench_phf_lookup,
);

criterion_main!(benches);
