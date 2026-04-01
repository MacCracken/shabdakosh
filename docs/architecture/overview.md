# Architecture Overview

> **shabdakosh** — pronunciation dictionary for AGNOS

## Module Map

```
shabdakosh/
├── src/
│   ├── lib.rs                — public API, module re-exports, feature flag docs
│   ├── error.rs              — ShabdakoshError enum (non_exhaustive)
│   ├── arpabet.rs            — ARPABET <-> svara Phoneme bidirectional mapping
│   ├── ipa.rs                — IPA <-> svara Phoneme mapping, greedy parser
│   └── dictionary/
│       ├── mod.rs            — PronunciationDict, merge, diff, from_lexicon
│       ├── entry.rs          — DictEntry, Pronunciation, Region
│       ├── validate.rs       — inventory validation via varna (feature: varna)
│       ├── detect.rs         — script/language detection via varna (feature: varna)
│       └── format/
│           ├── mod.rs        — CMUdict, IPA, JSON import/export, file I/O
│           ├── pls.rs        — W3C PLS XML import/export
│           └── ssml.rs       — SSML <phoneme> tag support
├── data/
│   └── cmudict-5k.txt       — 10,600+ entry CMUdict source (compile-time input)
├── build.rs                  — compile-time dictionary codegen from cmudict-5k.txt
├── benches/
│   └── benchmarks.rs         — criterion benchmarks (construction + lookup)
├── tests/
│   └── integration.rs        — cross-module integration tests
└── examples/
    └── basic.rs              — runnable usage example
```

## Data Flow

```
Word input ("hello")
  │
  ├─→ PronunciationDict::lookup()
  │     │
  │     ├─→ user_entries (BTreeMap)  ← checked first, application overrides
  │     │
  │     └─→ entries (HashMap)        ← base dictionary, 10,600+ compiled-in entries
  │           │
  │           └─→ DictEntry
  │                 └─→ Pronunciation[] (sorted by frequency)
  │                       ├── phonemes: Vec<svara::Phoneme>
  │                       ├── frequency: Option<f32>
  │                       └── region: Option<Region>
  │
  ├─→ format::parse_cmudict()    ← import from CMUdict text
  ├─→ format::parse_ipa()        ← import from IPA text
  ├─→ format::pls::parse_pls()   ← import from W3C PLS XML
  │
  └─→ [varna feature]
        ├─→ from_lexicon()       ← ingest varna::lexicon::Lexicon
        ├─→ validate()           ← check phonemes against varna inventory
        └─→ detect::detect_script()  ← identify writing system from Unicode
```

## Compile-Time Dictionary Generation

```
data/cmudict-5k.txt
  │  (10,600+ entries with @freq/@region annotations)
  │
  build.rs
  │  (parses CMUdict, maps ARPABET -> svara Phoneme, generates Rust code)
  │
  └─→ $OUT_DIR/generated_dict.rs
       │  (batched insert functions, 500 entries per function)
       │
       └─→ generated_english_entries() -> HashMap<String, DictEntry>
            │
            └─→ PronunciationDict::english()
```

## Dependency Stack

```
shabdakosh
  │
  ├── svara         — Phoneme enum (the phoneme type shabdakosh stores)
  ├── hashbrown     — O(1) HashMap for base dictionary (no_std compatible)
  ├── serde         — serialization for all types (alloc; std optional)
  ├── thiserror     — error derivation (no_std compatible)
  │
  └── optional (feature-gated):
      ├── serde_json — JSON import/export (feature: json)
      └── varna      — phoneme inventories, lexicons, scripts (feature: varna)
```

## Feature Flags

| Flag    | Default | Requires | Enables |
|---------|---------|----------|---------|
| `std`   | Yes     | —        | std-backed serde + thiserror, file I/O |
| `json`  | No      | —        | JSON import/export via serde_json |
| `varna` | No      | `std`    | multi-language validation, lexicon ingestion, script detection |
| `full`  | No      | —        | all of the above |

## Key Types

| Type | Location | Purpose |
|------|----------|---------|
| `PronunciationDict` | `dictionary/mod.rs` | Two-layer dictionary (base + user overlay) |
| `DictEntry` | `dictionary/entry.rs` | One or more pronunciations of a word |
| `Pronunciation` | `dictionary/entry.rs` | Phoneme sequence + frequency + region metadata |
| `Region` | `dictionary/entry.rs` | `GeneralAmerican` or `ReceivedPronunciation` |
| `DictDiff` | `dictionary/mod.rs` | Added/removed/changed words between two dicts |
| `ValidationReport` | `dictionary/validate.rs` | Result of inventory validation (varna feature) |
| `InvalidEntry` | `dictionary/validate.rs` | Entry with phonemes outside target inventory |
| `ShabdakoshError` | `error.rs` | Parse errors, unknown symbols, validation failures |

## Downstream Consumers

```
shabdakosh
  ├─→ shabda   — G2P engine (dictionary lookup + rules fallback)
  ├─→ dhvani   — audio engine (pronunciation for TTS)
  └─→ vansh    — voice AI shell (user-facing pronunciation overrides)
```

## Design Principles

- **Dictionary-first**: known-correct entries over algorithmic guessing
- **O(1) lookup**: hashbrown for base dictionary, BTreeMap only for small user overlay
- **Two-layer override**: user entries shadow base entries without modifying them
- **Compile-time generation**: 10,600+ entries embedded at build time, zero runtime I/O
- **Format-agnostic**: CMUdict, IPA, JSON, PLS, SSML all import/export the same types
- **no_std compatible**: core works with `alloc` only; `std` and `varna` are opt-in
- **Serializable**: all public types implement Serialize + Deserialize with roundtrip tests
- **Non-exhaustive**: all public enums use `#[non_exhaustive]` for forward compatibility
