# shabdakosh — Claude Code Instructions

> **Core rule**: this file is **preferences, process, and procedures** — durable
> rules that change rarely. Volatile state (current version, module line counts,
> port progress, test counts) lives in
> [`docs/development/state.md`](docs/development/state.md). Do not inline state here.

## Project Identity

**shabdakosh** (Sanskrit: *dictionary*) — pronunciation dictionary for AGNOS.
Cyrius port of a 7,085-line Rust library (kept in git at tag `3.1.3`, as `rust-old/`).

- **Type**: Port (Rust → Cyrius), flat library crate
- **License**: GPL-3.0-only
- **Language**: Cyrius (toolchain pinned in `cyrius.cyml [package].cyrius`)
- **Version**: `VERSION` at the project root is the source of truth — do not inline the number here

## Goal

shabdakosh OWNS **pronunciation lookup** in the AGNOS stack: a dictionary-first
grapheme→phoneme store mapping words to sequences of **svara phonemes**
(`SVARA_PH_*`), with ARPABET/IPA/X-SAMPA notation bridges, CMUdict/PLS/SSML/JSON
I/O, a user overlay, and G2P fallback. Accuracy over speed; dictionary-first.

## Consumers

shabda (G2P engine), dhvani (audio engine), vansh (voice AI shell) — and any AGNOS
component needing pronunciation lookup. Will pull `dist/shabdakosh.cyr`.

## Scaffolding

Scaffolded with `cyrius port`, which moved the Rust tree to `rust-old/`. That tree was removed in
3.1.4 (roadmap M3); it lives on at tag `3.1.3` — read a file with `git show 3.1.3:rust-old/<path>` —
and comments cite it as `3.1.3:rust-old/<path>`. It is the **functionality reference**: it says what
the port must be able to do, not whether a result is right. Do not restore it to the tree.

## Quick Start

```sh
cyrius deps                                   # resolve dependencies
cyrius build src/main.cyr build/shabdakosh    # compile
cyrius test tests/<mod>.tcyr                   # run one suite
cyrius test tests                              # run all .tcyr
```

## Work Loop

1. For a capability the Rust had, read its module at tag 3.1.3 (`git show 3.1.3:rust-old/<path>`) first —
   it defines the capability to cover
2. Port the module; keep `src/main.cyr` including it and building at every step
3. `cyrius fmt <file>`
4. `cyrius lint <file>`
5. `cyrius test tests/<mod>.tcyr` and `cyrius test tests`
6. `cyrius doctest <file>` (if it has doc examples)
7. `cyrius bench` (if performance-relevant — **never skip**)
8. Update `CHANGELOG.md` if user-facing; update `docs/development/state.md` progress

## Key Principles (durable)

- **The Rust original (`3.1.3:rust-old/`) is functionality coverage, not the oracle — accuracy is the
  oracle.** Every capability the
  Rust had must exist in the port; whether a result is *right* is judged by accuracy (the linguistics, the
  spec — CMUdict / IPA / XML / IEEE — and what a consumer would expect), never by "matches what Rust did".
  Do not bring over broken Rust: where the Rust is wrong, the port is right, and the CHANGELOG says so.
- **Correctness over cleverness** — a consumer-visible output change is recorded (CHANGELOG; an ADR when
  it is a real design choice), never silent.
- Test after every change; ONE change at a time.
- Build with `cyrius build` (the manifest auto-resolves deps), not `cat | cc5`.
- Source files only need project `include`s — stdlib auto-resolves from `cyrius.cyml`.
- `var buf[N]` = N **bytes**, not N entries.
- **Prefix everything** `shbdk_`/`SHBDK_`/`SHBDK_`/`Shbdk` — the distlib links flat (coexists with svara/varna, and with **shabda** which owns `shabda_`). shabdakosh must NOT use the `shabda_` prefix — that belongs to the G2P engine **shabda**, its first downstream consumer.

## Port Invariants (carried from the Rust crate)

- `#[non_exhaustive]` on all public enums → keep additive; give every `match` a `_ =>` catch-all arm.
- `#[must_use]` on pure functions → `#must_use`.
- Every type Serialize+Deserialize with roundtrip tests — see the serialization stance in `docs/development/state.md`.
- **Zero unwrap/panic** in library code → errors build on **sakshi** (packed i64 `[ctx][category][code]`, `0 == ok`, non-zero == error). A fallible fn returns a packed shabda error (`0 == ok`, test with `shabda_is_err`) and writes its payload to an out-param — or returns a payload pointer with `0` meaning none. No thiserror/Display; `shabda_err_name()` gives diagnostic text.
- Phoneme output compatible with svara's `PhonemeEvent` — a pronunciation is a sequence of svara `SVARA_PH_*` ordinals.
- **O(1)** base-dictionary lookup (`lib/hashmap.cyr`, the hashbrown replacement).
- User overlay is sorted for export — no Cyrius BTreeMap, so hashmap + sort-on-export.
- **Never skip benchmarks** before claiming a performance change.

## Data File

`data/cmudict-5k.txt` (300 KB, 10,692 lines) is the base-dictionary source of truth. Format:

- `WORD  PH1 PH2` — entry (two-space separator)
- `WORD(n)  …` — variant pronunciation (folded under the lowercased base word)
- `;;; @freq=0.85` — frequency annotation for the next entry
- `;;; @region=GA` — region annotation for the next entry

`programs/gen_cmudict.cyr` reads it and emits the checked-in
`src/dictionary/_cmudict_data.cyr` (packed-string pieces; the Cyrius replacement for the Rust
`build.rs`) and `src/dictionary/_static_phf_data.cyr` (the static dictionary's perfect hash, which
points into those pieces; scheme in `src/dictionary/phf.cyr`). One module — `cyrius distlib`'s per-module read cap is 1 MB since toolchain 6.4.10,
so the ~283 KB data no longer needs sharding. It `include`s `src/arpabet.cyr` and reuses that
mapping — one source of truth (no Rust-style table duplication). `src/dictionary/cmudict.cyr`
loads the pieces into a `lib/hashmap` at runtime. The generator reads the data by the runtime
reader's rules (`shbdk_parse_cmudict`), and a bad line stops it with its line number, as `build.rs`
panicked. **Regenerate after editing the data:**
`cyrius build programs/gen_cmudict.cyr build/gen_cmudict && ./build/gen_cmudict`.

## Rules (Hard Constraints)

- **Do not commit or push** — the user handles all git operations.
- **Never use `gh` CLI** — use `curl` to the GitHub API if needed.
- Do not modify `lib/` (vendored stdlib/deps). Do not restore `rust-old/`: read it at tag `3.1.3`.
- Do not add unnecessary dependencies.
- Do not hardcode toolchain versions in CI YAML — `cyrius = "X.Y.Z"` in `cyrius.cyml` is the source of truth.

## CHANGELOG Format (Keep a Changelog + SemVer)

```
## [version] — YYYY-MM-DD

Description.

- **Feature**: description
- **Breaking**: description
```

## Documentation

- [`docs/adr/`](docs/adr/) — Architecture Decision Records (*why X over Y?*)
- [`docs/architecture/`](docs/architecture/) — non-obvious constraints
- [`docs/guides/`](docs/guides/) — task-oriented how-tos
- [`docs/examples/`](docs/examples/) — runnable examples
- [`docs/development/state.md`](docs/development/state.md) — live port state
- [`docs/development/roadmap.md`](docs/development/roadmap.md) — milestones
