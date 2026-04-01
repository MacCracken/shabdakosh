# Changelog

## 1.1.0 — 2026-04-01

Multi-language foundation via varna integration.

- **`varna` feature flag** — optional dependency on varna for multi-language support
- **Language-tagged dictionaries** — `PronunciationDict` gains `language()`, `set_language()`, `with_language()` for ISO 639 codes
- **Inventory validation** — `validate()` checks dictionary phonemes against varna's per-language phoneme inventories; `validate_inventory()` for explicit inventory
- **Lexicon ingestion** — `from_lexicon()` converts `varna::lexicon::Lexicon` into a `PronunciationDict`, parsing IPA transcriptions into svara phonemes
- **Script detection** — `detect_script()` identifies writing system (Latin, Devanagari, Arabic, etc.) from Unicode code points
- **Language detection hint** — `detect_language_hint()` suggests candidate languages based on detected script
- **Seed dictionaries** — `spanish()`, `hindi()`, `german()`, `sanskrit()` constructors from varna's Swadesh lists
- **New error variants** — `PhonemeNotInInventory`, `UnknownLanguage`
- IPA length mark normalization in validation (handles ɔ/ɔː, ɑ/ɑː convention differences)
- Serde backward compatibility: `language` field is optional, absent in v1.0 serialized data

## 1.0.0 — 2026-03-27

Stable release with O(1) lookup, IPA, standards compliance, and dictionary operations.

- **O(1) lookup** via `hashbrown::HashMap` (replaces BTreeMap for base entries)
- **IPA module** (`ipa.rs`): bidirectional IPA-Phoneme mapping, `parse_ipa_word()`, `phonemes_to_ipa()`
- **IPA format** import/export: `parse_ipa()`, `to_ipa()` in `dictionary::format`
- **W3C PLS** (Pronunciation Lexicon Specification) import/export: `format::pls::parse_pls()`, `to_pls()`
- **SSML** `<phoneme>` tag support: `format::ssml::parse_ssml_phoneme()`, `to_ssml_phoneme()`
- **Dictionary merge**: `merge()` (override) and `merge_conservative()` (skip on conflict)
- **Dictionary diff**: `diff()` returns `DictDiff` with added/removed/changed words
- All v1.0 criteria met: 10K+ entries, O(1) lookup, variant pronunciations, IPA, PLS, SSML
- **Breaking**: `entries()` returns `&hashbrown::HashMap<String, DictEntry>` (was `BTreeMap`)

## 0.2.0 — 2026-03-27

Dictionary expansion with variant pronunciations and metadata.

- **10,600+ entry English dictionary** (doubled from 5K)
- **Variant pronunciations** for 23 common heteronyms (read, live, wind, etc.)
- **`Pronunciation` struct** with `phonemes()`, `frequency()`, `region()` accessors
- **`DictEntry` struct** wrapping one or more `Pronunciation` variants
- **`Region` enum** (`GeneralAmerican`, `ReceivedPronunciation`)
- **Frequency metadata** (`@freq` annotations) for heteronym disambiguation
- **New lookup methods**: `lookup_entry()`, `lookup_all()`, `insert_entry()`, `insert_user_entry()`
- `lookup()` backward compatible — returns primary (highest-frequency) pronunciation
- Extended CMUdict format: `WORD(n)` variants, `@freq`/`@region` annotations
- Serde backward compatibility: can deserialize v0.1.0 JSON format
- **Breaking**: `entries()` / `user_entries()` return `BTreeMap<String, DictEntry>` (was `Vec<Phoneme>`)

## 0.1.0 — 2026-03-27

Initial release. Extracted from shabda's dictionary subsystem.

- 5,014-entry English pronunciation dictionary (CMUdict-derived, compile-time codegen)
- ARPABET-to-svara Phoneme bidirectional mapping (39 symbols)
- `PronunciationDict` with user overlay (application-specific overrides)
- CMUdict text format import/export (no_std compatible)
- JSON import/export (optional `json` feature)
- File I/O convenience wrappers (`std` feature)
- `no_std` + `alloc` support
- Serde Serialize + Deserialize on all types
- Criterion benchmarks for construction and lookup
