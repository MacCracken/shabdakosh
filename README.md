# shabdakosh

**shabdakosh** (Sanskrit: dictionary) — Pronunciation dictionary crate for [AGNOS](https://github.com/MacCracken).

Maps words to [svara](https://crates.io/crates/svara) `Phoneme` sequences using a 10,600+ entry English dictionary derived from the CMU Pronouncing Dictionary. Multi-language support via optional [varna](https://crates.io/crates/varna) integration.

## Features

- **10,600+ entry English dictionary** generated at compile time from CMUdict (zero runtime parsing)
- **ARPABET mapping** — bidirectional conversion between ARPABET notation and svara phonemes
- **IPA mapping** — bidirectional IPA-Phoneme conversion with greedy parser
- **User overlay** — application-specific entries that override the base dictionary
- **Variant pronunciations** — heteronyms (read, live, wind) with frequency and region metadata
- **Import/export** — CMUdict, IPA, JSON, W3C PLS, SSML `<phoneme>` tags
- **Dictionary operations** — merge (override/conservative), diff (added/removed/changed)
- **Multi-language** (varna feature) — inventory validation, lexicon ingestion, script/language detection
- **no\_std compatible** — works with `alloc`, no standard library required

## Quick Start

```rust
use shabdakosh::PronunciationDict;

let dict = PronunciationDict::english();
assert!(dict.lookup("hello").is_some());
assert!(dict.len() >= 10000);
```

## User Overlay

Override or extend the built-in dictionary with application-specific pronunciations:

```rust
use shabdakosh::PronunciationDict;
use svara::phoneme::Phoneme;

let mut dict = PronunciationDict::english();

// Add a custom word
dict.insert_user("agnos", &[
    Phoneme::VowelAsh, Phoneme::PlosiveG,
    Phoneme::NasalN, Phoneme::VowelO, Phoneme::FricativeS,
]);

// User entries take precedence over base entries
assert!(dict.lookup("agnos").is_some());
```

## Import/Export

```rust
use shabdakosh::dictionary::format;

// Parse CMUdict format
let input = "hello  HH AH0 L OW1\nworld  W ER1 L D\n";
let dict = format::parse_cmudict(input).unwrap();

// Export back to CMUdict format
let output = format::to_cmudict(&dict);
```

Also supported: IPA text, JSON (`json` feature), W3C PLS XML, SSML `<phoneme>` tags.

## Multi-Language (varna feature)

```rust
use shabdakosh::PronunciationDict;

// Ingest a varna lexicon
let lexicon = varna::lexicon::swadesh::by_code("es").unwrap();
let dict = PronunciationDict::from_lexicon(&lexicon);
assert_eq!(dict.language(), Some("es"));

// Detect script from Unicode
use shabdakosh::dictionary::detect;
assert_eq!(detect::detect_script("नमस्ते"), Some("Deva"));
```

## Feature Flags

| Feature | Default | Description |
|---------|---------|-------------|
| `std`   | Yes     | Standard library support. Disable for `no_std` + `alloc` |
| `json`  | No      | JSON import/export via serde\_json |
| `varna` | No      | Multi-language: validation, lexicon ingestion, script detection |
| `full`  | No      | All features |

## Architecture

```text
shabdakosh
├── arpabet.rs          ARPABET <-> svara Phoneme
├── ipa.rs              IPA <-> svara Phoneme
└── dictionary/
    ├── mod.rs           PronunciationDict (hashbrown + BTreeMap overlay)
    ├── entry.rs         DictEntry, Pronunciation, Region
    ├── validate.rs      inventory validation (varna feature)
    ├── detect.rs        script/language detection (varna feature)
    └── format/
        ├── mod.rs       CMUdict, IPA, JSON import/export
        ├── pls.rs       W3C PLS XML
        └── ssml.rs      SSML <phoneme> tags
```

See [docs/architecture/overview.md](docs/architecture/overview.md) for the full architecture overview.

## Documentation

- [Usage Guide](docs/guides/usage.md) — comprehensive examples for all features
- [Architecture Overview](docs/architecture/overview.md) — module map, data flow, design principles
- [ADRs](docs/adr/) — architecture decision records

## Consumers

- [shabda](https://github.com/MacCracken/shabda) — G2P engine (dictionary lookup + rules fallback)
- [dhvani](https://github.com/MacCracken/dhvani) — Audio engine
- [vansh](https://github.com/MacCracken/vansh) — Voice AI shell

## License

GPL-3.0-only
