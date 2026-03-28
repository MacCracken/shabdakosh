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

## Backlog — High Priority

### Dictionary Expansion — Completed in v0.2.0
- [x] Expand to 10,000+ entries (10,600+)
- [x] Add variant pronunciations (23 heteronyms: read, live, wind, etc.)
- [x] Frequency metadata per entry (enables frequency-weighted selection)
- [x] Regional variants (Region enum: GeneralAmerican, ReceivedPronunciation)

### Performance
- [ ] Dictionary trie for O(1) lookup instead of BTreeMap
- [ ] Lazy dictionary loading (load on first use, not construction)
- [ ] Binary serialization format for fast deserialization
- [ ] Compile-time perfect hash (phf) for static entries

### Import/Export
- [ ] CMUdict variant pronunciation support (WORD(1), WORD(2))
- [ ] IPA format import/export
- [ ] SSML lexicon format
- [ ] Merge dictionaries (combine multiple sources with precedence)

### Multi-Language
- [ ] Spanish pronunciation dictionary
- [ ] German pronunciation dictionary
- [ ] Hindi/Devanagari dictionary (near 1:1 phoneme mapping)
- [ ] Language-tagged entries (single dict, multiple languages)

## Backlog — Medium Priority

### Quality
- [ ] Pronunciation validation (detect impossible phoneme sequences)
- [ ] Coverage reporting (what percentage of a corpus is dictionary-covered)
- [ ] Diff tool (compare two dictionaries, find disagreements)

### Integration
- [ ] C FFI for dictionary lookup
- [ ] WASM target support
- [ ] Dictionary builder CLI tool

## v1.0 Criteria

- [ ] 10,000+ entry English dictionary
- [ ] Trie or perfect hash for O(1) lookup
- [ ] Variant pronunciations with selection API
- [ ] IPA import/export
- [ ] Comprehensive documentation
- [ ] All public types: Serialize + Deserialize + roundtrip tested
- [ ] Benchmarks baselined
