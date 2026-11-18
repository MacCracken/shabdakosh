# shabdakosh — Roadmap

> What ships next, in what order, against which gates. Live state is in [`state.md`](state.md).
> **Every open item lives here.** There is no separate backlog: `backlog.md` was folded into this
> file in 3.1.2, and an item leaves this file when it ships (its record is the CHANGELOG).

The port shipped straight to **v3.0.0** (full parity with the Rust 2.x surface), so the original
v0.x/v1.0 sequencing collapsed into a single parity milestone.

## Release criteria (v3.0.0)

- [x] Rust → Cyrius surface parity verified (function-level against `rust-old/`; every module ✅ or consciously dropped)
- [x] Test coverage adequate for the surface area (689 assertions at the 3.0.0 gate; 1,426 / 28 suites after M3)
- [x] Benchmarks captured in [`docs/benchmarks.md`](../benchmarks.md)
- [x] At least one downstream consumer green (`dist/shabdakosh.cyr` linked + exercised by a consumer smoke)
- [x] CHANGELOG complete (3.0.0 entry)
- [x] Security audit pass ([`docs/audit/2026-07-05-audit.md`](../audit/2026-07-05-audit.md) — 9 findings, all fixed + regression-tested)

## Milestones

### M0 — Port scaffold (v0.1.0) — ✅ shipped 2026-07-05

- `cyrius port` scaffold landed
- Rust source moved to `rust-old/`
- Doc-tree per [first-party-documentation.md](https://github.com/MacCracken/agnosticos/blob/main/docs/development/applications/first-party-documentation.md)

### M1 — Full parity port (v3.0.0) — ✅ shipped 2026-07-05

Every Rust module ported function-for-function to CYRIUS and cross-checked against `rust-old/`:
error/arpabet/ipa/entry/morphology/syllable/notation, the dictionary keystone + coverage/stream/
trie/heteronym/g2p/static_dict, all format codecs (CMUdict/IPA/PLS/SSML/JSON/binary), lazy (mmap),
detect + validate (varna), and the WASM surface. Base CMUdict data generated as a single `.cyr` module (`_cmudict_data.cyr`).
distlib bundle built + consumer-verified. See [`state.md`](state.md) for the per-module ledger.

### M2 — The perfect-hash static dictionary (v3.1.2) — ✅ shipped 2026-11-17

The one feature the port had left below the Rust's fidelity. `static_dict` was a lazily built
singleton over `shbdk_dict_english()` because CYRIUS had no const-eval to bake a `phf` table. The
gate turned out to be answered upstream: cyrius archived the const-eval proposal at 6.7.2, when
`const fn` landed, and its roadmap keeps a `#phf` builtin only "if a filed need remains that
`const fn` + the generated-`.cyr` idiom do not cover". The generated-`.cyr` idiom covers it, exactly
as `build.rs` + `phf_codegen` did for the Rust: `programs/gen_cmudict.cyr` now also writes a CHD
perfect hash (`_static_phf_data.cyr`, scheme in `src/dictionary/phf.cyr`). The first lookup went
from ~8.7 ms to ~33 µs.

Shipped alongside: `fuzz/parsers.fcyr`, a real fuzz harness over every untrusted-input parser (it
replaced the scaffold stub and runs in CI), which found and fixed one round-trip defect.

### M3 — Retire `rust-old/` — ✅ done 2026-11-17 (sweep in 3.1.3, tree removed in 3.1.4)

The bar changed for this milestone: **the Rust original is the list of capabilities the port must
have; accuracy decides whether a result is right** ([ADR 005](../adr/005-accuracy-over-rust-parity.md)).
It now lives only at tag `3.1.3`: `git show 3.1.3:rust-old/<path>`.

1. ✅ **The function-for-function sweep** (2026-11-17). Seven parallel reviews ran the same inputs
   through a built copy of the Rust crate and through the port: the full English dump, every
   export, the formats' edge cases, the API surface, and the Rust's 178 unit, 53 integration and
   27 doc tests. Every missing capability was ported, every port defect fixed, and every Rust
   inaccuracy corrected (the CHANGELOG's 3.1.3 entry lists them). The 53 Rust integration tests are
   `tests/integration.tcyr`, one test group each.
2. ✅ **The open decisions are settled**, each by what is accurate:
   - `validate()` affricates and `IY` are fixed; the rest is varna's inventory (see below).
   - ARPABET notation parse reads stress digits.
   - CMUdict words are trimmed after their `(n)` marker and an empty word is refused.
   - Phonotactics covers every language varna profiles.
   - A frequency must be 0.0–1.0, so -1.0 is no longer a frequency that collides with "none".
3. ✅ **`rust-old/` removed** in 3.1.4 (after the 3.1.3 tag, with the maintainer's go-ahead). Source, test and
   bench comments cite the Rust files as `3.1.3:rust-old/<path>`, git's own spelling of a file at a
   tag, and CLAUDE.md, CONTRIBUTING, the ADRs and the guides point there too. The historical records
   (CHANGELOG entries, the 2026-07-05 audit) keep the paths they were written with.

## Gated on something outside this repository

- **varna's English inventory lacks four phonemes**: the NURSE vowel `ɜ` (or `ɜː`) and the
  diphthongs `aɪ`, `aʊ`, `ɔɪ`. They are the whole of `validate()`'s remaining 3,113 flags on the
  English dictionary ([architecture note 001](../architecture/001-phoneme-ipa-bridge-and-validation.md)).
  This is varna's data to add. **Filed upstream** (2026-11-17) as varna
  `docs/development/issues/2026-11-17-english-inventory-lacks-nurse-and-three-diphthongs.md`, with a
  repro. When varna ships the fix: bump `[deps.varna]`, add `ɜ` ↔ `ɝ` to
  `_shbdk_inventory_has_normalized` if varna spells NURSE `ɝ`, and re-measure (expected: 0 flags).
- **varna's script data.** Both filed upstream (2026-11-17), each with a repro:
  - **Script ranges cover part of each Unicode script**
    (varna `docs/development/issues/2026-11-17-script-ranges-cover-part-of-each-unicode-script.md`).
    Measured against Unicode 17.0 `Scripts.txt`: Latin has 450 of 1,492 code points, so Vietnamese
    (`1E00–1EFF`), IPA (`0250–02B8`) and fullwidth letters detect as no script, while `×` / `÷` count as
    Latin. Han misses 33,047, Kana its halfwidth forms, and Arabic, Cyrillic, Greek and Egyptian
    hundreds each. The proposal is to generate the ranges from `Scripts.txt` + `ScriptExtensions.txt`.
  - **Japanese is registered with Kana only**
    (varna `docs/development/issues/2026-11-17-japanese-registry-lists-only-kana.md`). A kanji word
    never hints `ja`. Also, varna's `Kana` holds Hiragana, where ISO 15924 `Kana` is Katakana only.
  - When varna ships either: bump `[deps.varna]` and add `detect.tcyr` cases for the words in the
    issues (`ở`, `ʃə`, `ＡＢＣ`, `ｶﾀｶﾅ`, `×`; `日本`, `人々`, `ひらがな`).
- **Nasal vowels.** svara has no nasalised-vowel phonemes, so the IPA reader keeps a nasal vowel's
  base vowel and drops the tilde (`ĩ` reads as `i`). A phoneme for each would be svara's to add.
- **`lazy_open`'s blob fallback is not exercised on Linux**, where mmap always succeeds. Opening a
  directory bails earlier (`xlseek` gives EINVAL), so reaching the branch needs an AGNOS run. The
  contract it must keep (`lazy_open` and `load_binary_file` decode a file to the same dictionary)
  is tested.
- **Lighter bundles for dictionary-only consumers.** A cross-crate question with two parts:
  - **bayan**: shabdakosh folds the 995 KB `bayan` monolith (with 353 KB of unicode tables at
    cyrius 6.7.18), where `dist/bayan-json.cyr` would do. shabda folds the monolith too, and a
    consumer must not hold both, so it is a stack decision, not a pin bump. varna 2.6.0's
    optional JSON support already uses `bayan-json`. (Since 3.1.3 the text layer uses the unicode
    casefold and category tables the monolith carries, so a lighter bundle would need those too.)
  - **svara**: shabdakosh uses only svara's `SVARA_PH_*` ordinals and its phoneme classes, but folds
    the whole synthesis bundle. A phoneme-only svara sub-bundle would be svara's to ship.

## By design (not gaps)

- **JSON is bayan, and phonemes serialize as `SVARA_PH_*` ints**, not the Rust crate's enum
  strings. Valid, standard JSON in the CYRIUS-native schema; no AGNOS consumer wants the old one.
  ([ADR 004](../adr/004-cyrius-port-decisions.md))
- **Binary is the CYRIUS format** (hand-rolled, replacing postcard), reusing the `SHBD` magic and
  version 1. There are no Rust `.bin` blobs in the AGNOS world; the decoder is bounds-checked
  either way. (ADR 004)
- **`detect.cyr` keeps its own insertion sort** rather than the keystone's `_shbdk_msort`: it sorts
  a handful of language codes, and the module is self-contained so its suite links without the
  dictionary tier.
- **`bench` is in `[deps].stdlib`**: `cyrius bench` resolves `bench_*` from the project stdlib list.
  It does not reach the shipped bundle, which is built from `[lib].modules`.

- **Inputs are NUL-terminated cstrings.** A byte 0 ends a word or a text, where a Rust `&str`
  could hold one. Every public text input is a cstr by the port's calling convention.
- **ARPABET is an English phoneme set.** A phoneme with no ARPABET symbol is left out of a CMUdict
  export (and a pronunciation with none at all is refused), and ARPABET marks no vowel length, so
  /i/ and /iː/ both write `IY1`.

## Out of scope

- **C FFI** (`ffi.rs`) — dropped as redundant: CYRIUS interops with C directly, so a dedicated FFI
  shim buys nothing, and there are no C-ABI consumers today. A backdoor exists to add one if needed.
- **The `shabdakosh-cli` binary and the criterion harness** — removed at 3.0.0. Every CLI
  subcommand maps onto library calls (import / export: the `shbdk_load_*` / `shbdk_parse_*` and
  `shbdk_to_*` / `shbdk_save_*` functions; `merge`, `diff`, `coverage`, `info`: their `shbdk_dict_*`
  functions), and the benches are `benches/hotpath.bcyr`.
