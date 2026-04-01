# Changelog

## [2.0.0] — 2026-04-01

Major release for shabda integration. Adds unified phoneme notation, syllabification,
morphological decomposition, and the complete v1.2–v1.4 feature set.

- **Breaking**: `lookup_entry()` fast path now skips `to_lowercase()` allocation when input is already lowercase — semantics unchanged, but internal storage relies on lowercase keys
- **Breaking**: `PrefixTrie` children switched from `BTreeMap` to `HashMap` — `search_prefix()` results are still sorted, but serialization order may differ
- **Breaking**: `#[non_exhaustive]` added to all public structs (`CoverageReport`, `HeteronymContext`, `InvalidEntry`, `ValidationReport`, `PhonotacticViolation`, `PhonotacticReport`, `StaticPronunciation`, `StaticEntry`, `Syllable`, `Morpheme`, `Decomposition`)
- **Feature**: `PhonemeNotation` trait — unified abstraction over ARPABET, IPA, and X-SAMPA (`notation` module)
- **Feature**: `Syllable`, `StressLevel`, `syllabify()` — syllable boundary detection using Maximal Onset Principle (`dictionary::syllable`)
- **Feature**: `Morpheme`, `MorphemeKind`, `Decomposition` — morphological decomposition tags for productive pronunciation (`dictionary::morphology`)
- **Feature**: `G2PModel` trait, `G2PResult`, `FallbackDict<M>` — neural/rule-based G2P fallback chain with confidence scores (`dictionary::g2p`)
- **Feature**: `FstModel` / `FstNotation` — Phonetisaurus WFST integration point
- **Feature**: Dictionary learning — `promote_prediction()`, `promote_if_confident()` on `FallbackDict`
- **Feature**: `PrefixTrie` — O(k) prefix search and autocomplete (`dictionary::trie`)
- **Feature**: Binary dictionary format via postcard — `to_binary()`, `from_binary()` (`binary` feature)
- **Feature**: PHF static dictionary — zero-allocation compile-time perfect hash lookups (`phf` feature, `dictionary::static_dict`)
- **Feature**: `LazyDict` — memory-mapped binary dictionary loading (`mmap` feature)
- **Feature**: `LookupStream` — zero-allocation streaming word-to-phoneme iterator
- **Feature**: Phonotactic validation — `validate_phonotactics()` detects forbidden sequences via varna constraints
- **Feature**: `CoverageReport` — text corpus coverage analysis
- **Feature**: `HeteronymResolver` trait — context-aware pronunciation selection for heteronyms
- **Feature**: C FFI — `extern "C"` API with opaque handles (`ffi` feature)
- **Feature**: WASM bindings — `WasmDict` via wasm-bindgen (`wasm` feature)
- **Feature**: `shabdakosh-cli` binary — import/export/merge/validate/diff/coverage/info (`cli` feature)
- **Performance**: Lookup 3x faster (skip allocation for lowercase inputs)
- **Performance**: Binary serialization 35% faster (zero-copy borrowing)
- **Performance**: Trie construction 18% faster (HashMap children)
- New feature flags: `binary`, `phf`, `mmap`, `ffi`, `wasm`, `cli`

## [1.1.0] — 2026-04-01

Multi-language foundation via varna integration.

- **Feature**: `varna` feature flag — optional dependency on varna for multi-language support
- **Feature**: Language-tagged dictionaries — `language()`, `set_language()`, `with_language()` for ISO 639 codes
- **Feature**: Inventory validation — `validate()` checks phonemes against varna's per-language inventories
- **Feature**: Lexicon ingestion — `from_lexicon()` converts `varna::lexicon::Lexicon` into a `PronunciationDict`
- **Feature**: Script detection — `detect_script()` identifies writing system from Unicode code points
- **Feature**: Language detection hint — `detect_language_hint()` suggests candidate languages by script
- **Feature**: Seed dictionaries — `spanish()`, `hindi()`, `german()`, `sanskrit()` constructors from varna's Swadesh lists
- **Feature**: New error variants — `PhonemeNotInInventory`, `UnknownLanguage`
- IPA length mark normalization in validation (handles convention differences)
- Serde backward compatibility: `language` field is optional, absent in v1.0 serialized data

## [1.0.0] — 2026-03-27

Stable release with O(1) lookup, IPA, standards compliance, and dictionary operations.

- **Feature**: O(1) lookup via `hashbrown::HashMap` (replaces BTreeMap for base entries)
- **Feature**: IPA module (`ipa.rs`) — bidirectional IPA-Phoneme mapping, `parse_ipa_word()`, `phonemes_to_ipa()`
- **Feature**: IPA format import/export — `parse_ipa()`, `to_ipa()`
- **Feature**: W3C PLS (Pronunciation Lexicon Specification) import/export
- **Feature**: SSML `<phoneme>` tag support
- **Feature**: Dictionary merge — `merge()` (override) and `merge_conservative()` (keep on conflict)
- **Feature**: Dictionary diff — `diff()` returns `DictDiff` with added/removed/changed words
- **Breaking**: `entries()` returns `&hashbrown::HashMap<String, DictEntry>` (was `BTreeMap`)

## [0.2.0] — 2026-03-27

Dictionary expansion with variant pronunciations and metadata.

- **Feature**: 10,600+ entry English dictionary (doubled from 5K)
- **Feature**: Variant pronunciations for 23 common heteronyms (read, live, wind, etc.)
- **Feature**: `Pronunciation` struct with `phonemes()`, `frequency()`, `region()` accessors
- **Feature**: `DictEntry` struct wrapping one or more `Pronunciation` variants
- **Feature**: `Region` enum (`GeneralAmerican`, `ReceivedPronunciation`)
- **Feature**: Frequency metadata (`@freq` annotations) for heteronym disambiguation
- **Feature**: New lookup methods — `lookup_entry()`, `lookup_all()`, `insert_entry()`, `insert_user_entry()`
- Extended CMUdict format: `WORD(n)` variants, `@freq`/`@region` annotations
- Serde backward compatibility: can deserialize v0.1.0 JSON format
- **Breaking**: `entries()` / `user_entries()` return `BTreeMap<String, DictEntry>` (was `Vec<Phoneme>`)

## [0.1.0] — 2026-03-27

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
