# shabdakosh Roadmap

Current version: **1.1.0**

---

## Backlog

### Data Expansion

- [ ] **Spanish dictionary** (5,000+ entries) — validated against `varna::phoneme::spanish()`. RAE-sourced pronunciation data.
- [ ] **Hindi/Devanagari dictionary** (5,000+ entries) — validated against `varna::phoneme::hindi()`. Near 1:1 grapheme-phoneme mapping.
- [ ] **German dictionary** (5,000+ entries) — validated against `varna::phoneme::german()`. Compound word handling via decomposition.
- [ ] **Sanskrit dictionary** (5,000+ entries) — validated against `varna::phoneme::sanskrit()`. Leverages varna's Swadesh list + Devanagari script metadata.

### v1.2 — Neural G2P Integration

**Goal**: Bridge dictionary lookup with neural G2P models for unknown words, inspired by [DeepPhonemizer](https://github.com/ExpressiveLabs/deepphonemizer-rs), [OpenPhonemizer](https://github.com/NeuralVox/OpenPhonemizer), and [OLaPh](https://arxiv.org/abs/2509.20086).

- [ ] **G2P model trait** — Define a `trait G2PModel { fn predict(&self, word: &str) -> Vec<Phoneme>; }` that neural models can implement. shabdakosh provides the trait; model crates provide implementations.
- [ ] **Fallback chain** — `PronunciationDict::with_fallback(model: impl G2PModel)` — lookup tries: user overlay -> base dict -> neural model. This is the pattern used by gruut and eSpeak-ng.
- [ ] **Phonetisaurus FST support** — Optional integration with `phonetisaurus-g2p-rs` for WFST-based G2P. Lighter than neural models, good accuracy.
- [ ] **Confidence scores** — G2P predictions include a confidence score (0.0-1.0). Low-confidence predictions can be flagged for human review and addition to the dictionary.
- [ ] **Dictionary learning** — When a G2P prediction is confirmed (by user or downstream feedback), automatically promote it to the user overlay. Dictionaries grow from usage.

### v1.3 — Performance & Scale

**Goal**: Handle 100K+ entry dictionaries efficiently for production TTS workloads.

- [ ] **Compile-time perfect hash (phf)** — Replace hashbrown with phf for the static base dictionary. Zero runtime allocation for base lookups.
- [ ] **Binary dictionary format** — Compact serialization for fast loading. Avoid JSON/text parsing overhead for large dictionaries. Memory-mapped file support.
- [ ] **Lazy loading** — Load dictionary entries on demand from a binary file. Only materialize entries that are actually looked up. Critical for embedded/mobile.
- [ ] **Trie index** — Prefix-based lookup for autocomplete and partial matching (e.g., "comput" matches "computer", "compute", "computing").
- [ ] **Streaming lookup** — Process words as a stream without materializing the full dictionary in memory. For real-time TTS pipelines.

### v1.4 — Quality & Tooling

**Goal**: Tools for dictionary maintainers and TTS developers.

- [ ] **Pronunciation validation** — Detect impossible or unlikely phoneme sequences using varna's phonotactic constraints per language (e.g., three consecutive plosives). Replaces ad-hoc rules with varna's structured data.
- [ ] **Coverage reporting** — Given a text corpus, report what percentage of tokens are dictionary-covered vs. falling through to rules/G2P. Identifies gaps.
- [ ] **Dictionary builder CLI** — Command-line tool for: importing CMUdict/IPA/PLS sources, merging dictionaries, validating entries, exporting to any format, computing diff between versions.
- [ ] **C FFI** — `extern "C"` API for dictionary lookup, enabling integration with C/C++ TTS engines, Python bindings (via PyO3), and WASM.
- [ ] **WASM target** — Compile to WebAssembly for browser-based TTS. Dictionary served as a binary blob, looked up client-side.
- [ ] **Heteronym disambiguation hooks** — Callback API where a POS tagger can inform the dictionary which variant to select for heteronyms (e.g., "I read books" vs "I have read books").

### v2.0 — Breaking Changes (eventual)

- [ ] **Unified phoneme notation** — Abstract over ARPABET, IPA, and SAMPA with a `PhonemeNotation` trait. Dictionary entries store notation-agnostic phoneme IDs.
- [ ] **Syllabification** — Dictionary entries include syllable boundaries. Needed for stress rules, hyphenation, and rhythm-based prosody.
- [ ] **Morphological awareness** — Dictionary entries can be tagged with morphological decomposition (un+happy, re+write). Enables productive pronunciation of derived forms without explicit entries.
