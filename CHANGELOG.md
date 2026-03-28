# Changelog

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
