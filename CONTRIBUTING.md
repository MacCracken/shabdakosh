# Contributing to shabdakosh

Thank you for your interest in contributing to shabdakosh — the pronunciation
dictionary for AGNOS, written in [CYRIUS](https://github.com/MacCracken/cyrius).

## Getting Started

1. Fork the repository
2. Clone your fork
3. Create a feature branch: `git checkout -b feature/your-feature`
4. Make your changes following the guidelines below
5. Submit a pull request

## Development Requirements

- The **CYRIUS toolchain** (`cyrius`). The exact version is pinned in
  [`cyrius.cyml`](cyrius.cyml) under `[package].cyrius` — do not hardcode it elsewhere.
- `cyrius deps` to resolve the git+tag dependencies (svara, varna) into `lib/`.

There is no Rust/cargo toolchain. The Rust original was removed after 3.1.3 and lives at that tag
as the functionality reference: `git show 3.1.3:rust-old/<path>`.

## Code Quality Requirements

Before submitting a PR, run the project sweep (fmt + lint + docs + tests + bench):

```sh
cyrius audit
```

Or the individual checks:

```sh
cyrius fmt <file.cyr>          # format (add --check to verify without writing)
cyrius lint <file.cyr>         # static analysis (warnings + untracked deferrals)
cyrius test tests              # run every tests/*.tcyr suite
cyrius doctest <file.cyr>      # run doc examples, if the file has any
cyrius bench                   # benchmarks (auto-discovers benches/)
```

## Code Standards

- **The Rust original (`3.1.3:rust-old/`) is the capability list; accuracy is the oracle**
  ([ADR 005](docs/adr/005-accuracy-over-rust-parity.md)). Every Rust capability must exist here; where the Rust is wrong the port is right, and a
  consumer-visible output change goes in the CHANGELOG (an ADR when it is a real design choice).
- **Prefix everything** `shbdk_` / `SHBDK_` (the distlib links flat). Never `shabda_`: that prefix
  belongs to the G2P engine shabda.
- **Additive enums**: give every `match` a `_ =>` catch-all arm (the CYRIUS form of the Rust
  `#[non_exhaustive]` invariant).
- **`#must_use`** on pure functions.
- **Zero `unwrap`/`panic`** — errors are **sakshi** packed-i64 codes (`0 == ok`, test with
  `shbdk_is_err`; the reason a parse failed is `shbdk_last_error()`). A fallible function returns
  a payload pointer (`0` == none/error) or writes its payload to an out-param. String-returning functions must be annotated `: cstring`.
- **Serialize + Deserialize round-trip** for every type carrying state — hand-written codecs (JSON
  via **bayan**), each with a round-trip test.
- Dictionary-first, accuracy over speed. O(1) base-dictionary lookup.

## Adding Dictionary Entries

1. Add entries to `data/cmudict-5k.txt` in CMUdict ARPABET format.
2. For heteronyms, use the `WORD(n)` variant convention with `;;; @freq=` annotations.
3. Ensure every vowel has a stress digit (0, 1, or 2); consonants never have stress digits.
4. **Regenerate** the checked-in data module (the CYRIUS replacement for the Rust `build.rs`):

   ```sh
   cyrius build programs/gen_cmudict.cyr build/gen_cmudict && ./build/gen_cmudict
   ```

   This overwrites `src/dictionary/_cmudict_data.cyr`.
5. Add or extend a `.tcyr` test under `tests/` covering the new words.

## Benchmarks

Performance-relevant changes must include benchmark results — never skip them. Run
`cyrius bench` (it auto-discovers `benches/hotpath.bcyr`); see [`docs/benchmarks.md`](docs/benchmarks.md)
and `benches/history.csv`.

## License

By contributing, you agree that your contributions will be licensed under GPL-3.0.
