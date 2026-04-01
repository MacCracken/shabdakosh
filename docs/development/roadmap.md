# shabdakosh Roadmap

Current version: **2.0.0**

---

## Completed

### v2.0 — Breaking Changes

Unified phoneme notation, syllabification, morphological decomposition, performance optimizations, and `#[non_exhaustive]` on all public structs.

- **Feature**: `PhonemeNotation` trait — abstract over ARPABET, IPA, X-SAMPA (`notation` module)
- **Feature**: `Syllable`, `StressLevel`, `syllabify()` — syllable boundaries via Maximal Onset Principle
- **Feature**: `Morpheme`, `MorphemeKind`, `Decomposition` — morphological decomposition tags
- **Performance**: Lookup 3x faster, binary serialize 35% faster, trie construction 18% faster
- **Breaking**: `#[non_exhaustive]` on all public structs; trie children switched to HashMap

### v1.4 — Quality & Tooling

Phonotactic validation, coverage reporting, CLI, C FFI, WASM, heteronym hooks.

- **Feature**: `PhonotacticReport` — detect forbidden phoneme sequences via varna
- **Feature**: `CoverageReport` — text corpus coverage analysis
- **Feature**: `shabdakosh-cli` binary — import/export/merge/validate/diff/coverage/info
- **Feature**: C FFI (`extern "C"` API with opaque handles, `ffi` feature)
- **Feature**: WASM bindings via `wasm-bindgen` (`wasm` feature)
- **Feature**: `HeteronymResolver` trait and `lookup_with_context()` for POS-tagger integration

### v1.3 — Performance & Scale

Compile-time perfect hash, binary format, lazy loading, trie index, streaming lookup.

- **Feature**: `PrefixTrie` for O(k) prefix search and autocomplete
- **Feature**: Binary dictionary format via postcard (`to_binary` / `from_binary`)
- **Feature**: PHF static dictionary — zero-allocation base lookups (`static_dict`)
- **Feature**: `LazyDict` with mmap for on-demand loading (`mmap` feature)
- **Feature**: `LookupStream` iterator for zero-allocation streaming word processing

### v1.2 — Neural G2P Integration

G2P model trait, fallback chain (user overlay → base dict → model), Phonetisaurus FST stub, confidence scores, dictionary learning.

- **Feature**: `G2PModel` trait, `G2PResult`, `FallbackDict<M>`
- **Feature**: `LookupSource` enum for identifying where results came from
- **Feature**: `FstModel` / `FstNotation` integration point for WFST-based G2P
- **Feature**: `promote_prediction()` / `promote_if_confident()` for dictionary learning

---

## Backlog

### Data Expansion

- [ ] **Spanish dictionary** (5,000+ entries) — validated against `varna::phoneme::spanish()`. RAE-sourced pronunciation data.
- [ ] **Hindi/Devanagari dictionary** (5,000+ entries) — validated against `varna::phoneme::hindi()`. Near 1:1 grapheme-phoneme mapping.
- [ ] **German dictionary** (5,000+ entries) — validated against `varna::phoneme::german()`. Compound word handling via decomposition.
- [ ] **Sanskrit dictionary** (5,000+ entries) — validated against `varna::phoneme::sanskrit()`. Leverages varna's Swadesh list + Devanagari script metadata.
