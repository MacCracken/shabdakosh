# shabdakosh — Claude Code Instructions

## Project Identity

**shabdakosh** (Sanskrit: dictionary) — Pronunciation dictionary crate for AGNOS

- **Type**: Flat library crate
- **License**: GPL-3.0
- **MSRV**: 1.89
- **Version**: SemVer 1.0.0

## Consumers

shabda (G2P engine), dhvani (audio engine), vansh (voice AI shell), and any AGNOS component needing pronunciation lookup.

## Dependencies

- **svara**: Phoneme types, synthesis engine, voice profiles
- **hashbrown**: O(1) HashMap for base dictionary lookup

## Work Loop

1. Read the relevant code before proposing changes
2. Make the change
3. `cargo fmt`
4. `cargo clippy --all-features --all-targets -- -D warnings`
5. `cargo test --all-features`
6. `cargo test --doc`
7. `cargo check --no-default-features` (no_std verification)
8. `cargo bench` (if performance-relevant)
9. Update CHANGELOG.md if user-facing
10. Update docs/development/roadmap.md if completing a roadmap item

## Task Sizing

- **Small**: Single-function change, test fix, doc tweak
- **Medium**: New format support, new dictionary method, test suite expansion
- **Large**: Storage migration, new module (e.g., ipa.rs), data expansion

## Key Principles

- Never skip benchmarks
- `#[non_exhaustive]` on ALL public enums
- `#[must_use]` on all pure functions
- Every type must be Serialize + Deserialize (serde)
- Zero unwrap/panic in library code
- All types must have serde roundtrip tests
- Dictionary-first, accuracy over speed
- Phoneme output must be compatible with svara's PhonemeEvent
- O(1) lookup for base dictionary (hashbrown::HashMap)
- User overlay uses BTreeMap (small, sorted for export)

## Module Structure

- `arpabet.rs` — ARPABET-to-Phoneme bidirectional mapping
- `ipa.rs` — IPA-to-Phoneme bidirectional mapping
- `dictionary/mod.rs` — PronunciationDict, merge, diff, DictDiff
- `dictionary/entry.rs` — DictEntry, Pronunciation, Region
- `dictionary/format/mod.rs` — CMUdict, IPA, JSON import/export
- `dictionary/format/pls.rs` — W3C PLS XML import/export
- `dictionary/format/ssml.rs` — SSML phoneme tag support
- `error.rs` — ShabdakoshError

## Data File

`data/cmudict-5k.txt` is processed by `build.rs` at compile time. Format:
- `WORD  PH1 PH2` — entry (two-space separator)
- `WORD(n)  PH1 PH2` — variant pronunciation
- `;;; @freq=0.85` — frequency annotation for next entry
- `;;; @region=GA` — region annotation for next entry

## DO NOT

- **Do not commit or push** — the user handles all git operations
- **NEVER use `gh` CLI** — use `curl` to GitHub API only
- Do not add unnecessary dependencies
- Do not skip benchmarks before claiming performance improvements

## Documentation

- CHANGELOG.md: Keep a Changelog format (Added/Changed/Fixed/Removed)
- README.md: Quick start, feature list, architecture
- docs/development/roadmap.md: Completed versions + backlog

## CHANGELOG Format

```
## [version] — YYYY-MM-DD

Description.

- **Feature**: description
- **Breaking**: description
```
