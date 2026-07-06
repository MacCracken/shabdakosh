# Changelog

## [3.0.0] — 2026-07-05

Complete port of shabdakosh from Rust to the **CYRIUS** language (AGNOS toolchain). A
full-parity port: every Rust module reproduced against the preserved `rust-old/` oracle and
cross-checked by a 689-assertion suite across 26 test groups, plus a consumer-verified distlib
bundle (`dist/shabdakosh.cyr`).

- **Breaking**: Language change — shabdakosh is now a CYRIUS (`.cyr`) library, not a Rust crate. The API is flat, `shabda_`-prefixed C-style functions (`shabda_dict_lookup`, `shabda_parse_cmudict`, …) rather than Rust methods/traits/generics. Consumers pull `dist/shabdakosh.cyr`.
- **Breaking**: Errors are **sakshi** packed-i64 codes (`0 == ok`) instead of `thiserror` enums; fallible functions return a payload pointer (`0` == none) or a packed error (test with `shabda_is_err`).
- **Breaking**: Traits → function-pointer dispatch (G2P `FallbackDict`, heteronym resolver) and enum-tag dispatch (notation, lookup source); `Option<T>` → sentinels; `Result<T>` → pointer-or-0.
- **Feature**: full pronunciation surface — base dictionary (ARPABET/IPA↔svara phonemes), user overlay, O(1) hashmap lookup (~135 ns/hit), merge/diff, coverage, streaming lookup, prefix trie, heteronym context, G2P fallback chain.
- **Feature**: I/O formats — CMUdict / IPA / PLS / SSML text codecs (hand-written), JSON via the bayan DOM, and a compact hand-rolled binary format (replacing postcard); `LazyDict` (mmap-backed with a `file_read_all` fallback).
- **Feature**: varna-backed inventory + phonotactics **validation**, script/language **detection** (with a UTF-8 code-point decoder), and **Swadesh seed-dictionary constructors** (`shabda_from_lexicon`, `shabda_dict_spanish`/`hindi`/`german`/`sanskrit`).
- **Feature**: WASM binding surface (`WasmDict`) and the static dictionary (`static_dict`) ported as `.cyr` modules.
- **Feature**: base CMUdict data generated as a single checked-in `.cyr` module (`gen_cmudict.cyr`, the `build.rs` port); it fits under the distlib 1 MB per-module cap.
- **Changed**: `phf` static dict → lazy cached singleton — CYRIUS has no compile-time perfect hash; surface + lookup preserved, one-time ~9.6 ms load instead of a compile-time-baked table.
- **Removed**: C FFI (`ffi.rs`) — dead in the CYRIUS/AGNOS stack (no C-ABI consumers).
- **Removed**: the Rust `cli` binary and criterion harness — replaced by `.tcyr` tests and `benches/hotpath.bcyr` (see `docs/benchmarks.md`).

## [2.0.0] — 2026-04-01

Major release for shabda integration. Adds unified phoneme notation, syllabification,
morphological decomposition, and the complete v1.2–v1.4 feature set.

- **Breaking**: svara dependency bumped from 1.x to 2.0.0
- **Breaking**: `lookup_entry()` fast path skips `to_lowercase()` allocation when input is already lowercase — semantics unchanged, but internal storage relies on lowercase keys
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

## [1.0.0] — 2026-03-28

Initial stable release. 10K+ entry English dictionary with O(1) lookup, multiple notation
formats, and dictionary operations. Includes all pre-1.0 development (variant pronunciations,
metadata, IPA, PLS, SSML support).

- 10,600+ entry English pronunciation dictionary (CMUdict-derived, compile-time codegen)
- ARPABET-to-svara Phoneme bidirectional mapping (39 symbols)
- IPA module (`ipa.rs`) — bidirectional IPA-Phoneme mapping, `parse_ipa_word()`, `phonemes_to_ipa()`
- `PronunciationDict` with two-layer lookup: user overlay (BTreeMap) over base (hashbrown::HashMap)
- Variant pronunciations for 23 common heteronyms (read, live, wind, etc.)
- `Pronunciation` struct with `phonemes()`, `frequency()`, `region()` metadata
- `DictEntry` struct with multiple `Pronunciation` variants sorted by frequency
- `Region` enum (`GeneralAmerican`, `ReceivedPronunciation`)
- Import/export: CMUdict text, IPA text, W3C PLS XML, SSML `<phoneme>` tags, JSON (`json` feature)
- Dictionary merge (`merge()`, `merge_conservative()`) and diff (`diff()` → `DictDiff`)
- File I/O convenience wrappers (`std` feature)
- `no_std` + `alloc` support
- Serde Serialize + Deserialize on all types
- Criterion benchmarks for construction and lookup
