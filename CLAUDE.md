# shabdakosh — Claude Code Instructions

## Project Identity

**shabdakosh** (Sanskrit: dictionary) — Pronunciation dictionary crate for AGNOS

- **Type**: Flat library crate
- **License**: GPL-3.0
- **MSRV**: 1.89
- **Version**: SemVer 0.1.0

## Consumers

shabda (G2P engine), dhvani (audio engine), vansh (voice AI shell), and any AGNOS component needing pronunciation lookup.

## Dependencies

- **svara**: Phoneme types, synthesis engine, voice profiles

## Key Principles

- Never skip benchmarks
- `#[non_exhaustive]` on ALL public enums
- `#[must_use]` on all pure functions
- Every type must be Serialize + Deserialize (serde)
- Zero unwrap/panic in library code
- All types must have serde roundtrip tests
- Dictionary-first, accuracy over speed

## DO NOT

- **Do not commit or push** — the user handles all git operations
- **NEVER use `gh` CLI** — use `curl` to GitHub API only
- Do not add unnecessary dependencies
- Do not skip benchmarks before claiming performance improvements
