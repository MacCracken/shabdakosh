# shabdakosh

**shabdakosh** (Sanskrit: dictionary) — Pronunciation dictionary crate for [AGNOS](https://github.com/MacCracken).

Maps words to [svara](https://crates.io/crates/svara) `Phoneme` sequences using a 5,000+ entry English dictionary derived from the CMU Pronouncing Dictionary.

## Features

- **5,000+ entry English dictionary** generated at compile time from CMUdict (zero runtime parsing)
- **ARPABET mapping** — bidirectional conversion between ARPABET notation and svara phonemes
- **User overlay** — application-specific entries that override the base dictionary
- **Import/export** — CMUdict text format (no\_std) and JSON (with `json` feature)
- **no\_std compatible** — works with `alloc`, no standard library required

## Quick Start

```rust
use shabdakosh::PronunciationDict;

let dict = PronunciationDict::english();
assert!(dict.lookup("hello").is_some());
assert!(dict.len() >= 5000);
```

## User Overlay

Override or extend the built-in dictionary with application-specific pronunciations:

```rust
use shabdakosh::PronunciationDict;
use svara::phoneme::Phoneme;

let mut dict = PronunciationDict::english();

// Add a custom word
dict.insert_user("agnos", &[
    Phoneme::VowelAsh,
    Phoneme::PlosiveG,
    Phoneme::NasalN,
    Phoneme::VowelO,
    Phoneme::FricativeS,
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

## Feature Flags

| Feature | Default | Description |
|---------|---------|-------------|
| `std`   | Yes     | Standard library support. Disable for `no_std` + `alloc` |
| `json`  | No      | JSON import/export via serde\_json |

## Architecture

```text
                   shabdakosh
                  /          \
         arpabet              dictionary
    (ARPABET <-> Phoneme)    /          \
                          mod.rs      format.rs
                     (PronunciationDict,  (CMUdict/JSON
                      user overlay,        import/export)
                      generated 5K dict)
```

## Consumers

- [shabda](https://github.com/MacCracken/shabda) — G2P engine (re-exports shabdakosh types)
- [dhvani](https://github.com/MacCracken/dhvani) — Audio engine
- [vansh](https://github.com/MacCracken/vansh) — Voice AI shell

## License

GPL-3.0-only
