# shabdakosh Roadmap

## Completed

### v0.1.0 — Initial Release (2026-03-27)

- 5,014-entry English dictionary from CMUdict (compile-time codegen)
- ARPABET-to-svara Phoneme bidirectional mapping (39 symbols)
- User overlay (application-specific entries override base dictionary)
- CMUdict text format import/export (no_std compatible)
- JSON import/export (optional `json` feature)
- File I/O convenience wrappers (std-only)
- no_std + alloc support
- Serde roundtrip for all types
- Send + Sync compile-time assertions
- Criterion benchmarks for construction and lookup

### v0.2.0 — Dictionary Expansion (2026-03-27)

- 10,600+ entry English dictionary (doubled from 5K)
- Variant pronunciations for 23 common heteronyms (read, live, wind, etc.)
- `Pronunciation`, `DictEntry`, `Region` types
- Frequency metadata (`@freq` annotations)
- Extended CMUdict format: `WORD(n)` variants, `@freq`/`@region` annotations
- New lookup methods: `lookup_entry()`, `lookup_all()`, `insert_entry()`

### v1.0.0 — Stable Release (2026-03-27)

- O(1) lookup via hashbrown::HashMap
- IPA module: bidirectional IPA-Phoneme mapping
- IPA format import/export
- W3C PLS (Pronunciation Lexicon Specification) import/export
- SSML `<phoneme>` tag support
- Dictionary merge (override + conservative)
- Dictionary diff (added/removed/changed)
- All v1.0 criteria met

## v1.0 Criteria — All Met

- [x] 10,000+ entry English dictionary (10,600+)
- [x] Hash map for O(1) lookup (hashbrown::HashMap)
- [x] Variant pronunciations with selection API (DictEntry, lookup_all, for_region)
- [x] IPA import/export (ipa.rs, format::parse_ipa/to_ipa)
- [x] Comprehensive documentation (rustdoc on all public items)
- [x] All public types: Serialize + Deserialize + roundtrip tested
- [x] Benchmarks baselined (criterion)

---

## Post-v1.0 Roadmap

### v1.1 — Standards & Interop

**Goal**: First-class integration with speech synthesis standards used by industry TTS engines (Google Cloud TTS, Amazon Polly, Azure Speech).

- [ ] **W3C PLS (Pronunciation Lexicon Specification) import/export** — XML format used by SSML-based TTS engines for custom pronunciation lexicons. Parse `<lexicon>` documents, emit valid PLS XML. Enables shabdakosh dictionaries to be used directly with Amazon Polly, Google Cloud TTS, and Azure Speech.
- [ ] **SSML `<phoneme>` tag support** — Parse and emit `<phoneme alphabet="ipa" ph="...">` tags. Used by all major TTS APIs for inline pronunciation overrides.
- [ ] **Merge dictionaries** — Combine multiple `PronunciationDict` instances with configurable precedence (base < domain < user). Enables modular dictionary composition (medical + legal + general).
- [ ] **Dictionary diff** — Compare two dictionaries, report additions, removals, and pronunciation disagreements. Useful for quality assurance when updating dictionary sources.

### v1.2 — Multi-Language Foundation (lipi integration)

**Goal**: Support languages beyond English. lipi provides IPA inventories to validate dictionary entries against; shabdakosh ingests lipi's `Lexicon` type as a dictionary source alongside CMUdict.

**Boundary**: lipi owns "what phonemes exist in language X". shabdakosh owns "word → pronunciation mapping". Dictionary entries are validated against lipi inventories to catch transcription errors.

- [ ] **Optional `lipi` feature flag** — lipi inventory validation on dictionary entries (reject phonemes not in target language's inventory)
- [ ] **IPA dictionary format using lipi types** — `lipi::lexicon::Lexicon` can be ingested as a dictionary source, converting `LexEntry` → `DictEntry`
- [ ] **Language-tagged entries** — `PronunciationDict` gains a `language: Option<Language>` field. Entries can be tagged per-language, enabling polyglot dictionaries.
- [ ] **Spanish dictionary** (5,000+ entries) — validated against `lipi::phoneme::spanish()`. RAE-sourced pronunciation data.
- [ ] **Hindi/Devanagari dictionary** — validated against `lipi::phoneme::hindi()`. Near 1:1 grapheme-phoneme mapping.
- [ ] **German dictionary** (5,000+ entries) — validated against `lipi::phoneme::german()`. Compound word handling via decomposition.
- [ ] **Sanskrit dictionary** — validated against `lipi::phoneme::sanskrit()`. Leverages lipi's Swadesh list + Devanagari script metadata.
- [ ] **Language detection hint** — Given a word, suggest most likely source language based on lipi's script Unicode ranges and n-gram patterns.

### v1.3 — Neural G2P Integration

**Goal**: Bridge dictionary lookup with neural G2P models for unknown words, inspired by [DeepPhonemizer](https://github.com/ExpressiveLabs/deepphonemizer-rs), [OpenPhonemizer](https://github.com/NeuralVox/OpenPhonemizer), and [OLaPh](https://arxiv.org/abs/2509.20086).

- [ ] **G2P model trait** — Define a `trait G2PModel { fn predict(&self, word: &str) -> Vec<Phoneme>; }` that neural models can implement. shabdakosh provides the trait; model crates provide implementations.
- [ ] **Fallback chain** — `PronunciationDict::with_fallback(model: impl G2PModel)` — lookup tries: user overlay -> base dict -> neural model. This is the pattern used by gruut and eSpeak-ng.
- [ ] **Phonetisaurus FST support** — Optional integration with `phonetisaurus-g2p-rs` for WFST-based G2P. Lighter than neural models, good accuracy.
- [ ] **Confidence scores** — G2P predictions include a confidence score (0.0-1.0). Low-confidence predictions can be flagged for human review and addition to the dictionary.
- [ ] **Dictionary learning** — When a G2P prediction is confirmed (by user or downstream feedback), automatically promote it to the user overlay. Dictionaries grow from usage.

### v1.4 — Performance & Scale

**Goal**: Handle 100K+ entry dictionaries efficiently for production TTS workloads.

- [ ] **Compile-time perfect hash (phf)** — Replace hashbrown with phf for the static base dictionary. Zero runtime allocation for base lookups.
- [ ] **Binary dictionary format** — Compact serialization for fast loading. Avoid JSON/text parsing overhead for large dictionaries. Memory-mapped file support.
- [ ] **Lazy loading** — Load dictionary entries on demand from a binary file. Only materialize entries that are actually looked up. Critical for embedded/mobile.
- [ ] **Trie index** — Prefix-based lookup for autocomplete and partial matching (e.g., "comput" matches "computer", "compute", "computing").
- [ ] **Streaming lookup** — Process words as a stream without materializing the full dictionary in memory. For real-time TTS pipelines.

### v1.5 — Quality & Tooling

**Goal**: Tools for dictionary maintainers and TTS developers.

- [ ] **Pronunciation validation** — Detect impossible or unlikely phoneme sequences using lipi's phonotactic constraints per language (e.g., three consecutive plosives). Replaces ad-hoc rules with lipi's structured data.
- [ ] **Coverage reporting** — Given a text corpus, report what percentage of tokens are dictionary-covered vs. falling through to rules/G2P. Identifies gaps.
- [ ] **Dictionary builder CLI** — Command-line tool for: importing CMUdict/IPA/PLS sources, merging dictionaries, validating entries, exporting to any format, computing diff between versions.
- [ ] **C FFI** — `extern "C"` API for dictionary lookup, enabling integration with C/C++ TTS engines, Python bindings (via PyO3), and WASM.
- [ ] **WASM target** — Compile to WebAssembly for browser-based TTS. Dictionary served as a binary blob, looked up client-side.
- [ ] **Heteronym disambiguation hooks** — Callback API where a POS tagger can inform the dictionary which variant to select for heteronyms (e.g., "I read books" vs "I have read books").

### v2.0 — Breaking Changes (eventual)

- [ ] **Unified phoneme notation** — Abstract over ARPABET, IPA, and SAMPA with a `PhonemeNotation` trait. Dictionary entries store notation-agnostic phoneme IDs.
- [ ] **Syllabification** — Dictionary entries include syllable boundaries. Needed for stress rules, hyphenation, and rhythm-based prosody.
- [ ] **Morphological awareness** — Dictionary entries can be tagged with morphological decomposition (un+happy, re+write). Enables productive pronunciation of derived forms without explicit entries.
