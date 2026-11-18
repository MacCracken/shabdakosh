# shabdakosh — Roadmap

> What ships next, in what order, against which gates. Live state is in [`state.md`](state.md).
> **Every open item lives here.** There is no separate backlog: `backlog.md` was folded into this
> file in 3.1.2, and an item leaves this file when it ships (its record is the CHANGELOG).

The port shipped straight to **v3.0.0** (full parity with the Rust 2.x surface), so the original
v0.x/v1.0 sequencing collapsed into a single parity milestone.

## Release criteria (v3.0.0)

- [x] Rust → Cyrius surface parity verified (function-level against `rust-old/`; every module ✅ or consciously dropped)
- [x] Test coverage adequate for the surface area (689 assertions at the 3.0.0 gate; 997 / 26 suites at 3.1.2)
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

### M3 — Retire `rust-old/` — next

Unblocked by M2: the perfect hash was the last gate the old backlog put in front of this.

1. **A final function-for-function parity sweep** of every ported module against `rust-old/`.
   Anything that diverges either gets fixed or joins *Open decisions* below.
2. **Settle the open decisions** below. Each one is a place the oracle itself is wrong or
   limited; once `rust-old/` is gone, the ADR is the only record of why the port differs.
3. **Remove `rust-old/`.** It needs the maintainer's go-ahead: CLAUDE.md protects it as the
   parity oracle. CLAUDE.md, ADR 004, `state.md` and many test comments cite `rust-old/` paths;
   re-point them at a tag of this repository that still has it.

## Open decisions

Each would be a deliberate divergence from `rust-old/`, so each needs an ADR before code changes.
The fuzz harness tolerates the two it can reach (2 and 3), with a comment pointing here.

1. **`dict.validate()` reports 5,468 of 10,617 built-in English entries as invalid.** Measured and
   written up in [architecture note 001](../architecture/001-phoneme-ipa-bridge-and-validation.md);
   `rust-old/` behaves identically, and the count is unchanged on varna 2.6.0 (measured in 3.1.2).
   Three causes: varna spells affricates with the tie bar (`t͡ʃ`) while the port emits bare `tʃ`
   (1,070 entries); ARPABET `IY` maps to `SVARA_PH_VOWEL_E` → IPA `e` rather than `iː` (2,306);
   and varna's `en` inventory has no NURSE vowel and only two of five English diphthongs (3,358).
   Only the first is this crate's to fix. Decide before any consumer relies on `validate()`.
2. **The ARPABET notation drops every vowel on a round trip** (found by the 3.1.2 fuzz harness).
   `shbdk_notation_render(ARPABET, …)` writes stress digits (`OW1`, `AH0`) and
   `shbdk_notation_parse(ARPABET, …)` reads through `arpabet_to_phoneme`, which takes none, so
   `parse(render(v))` keeps only the consonants. The Rust's `Arpabet::str_to_phoneme` does the
   same. The fix is to parse through `shbdk_arpabet_to_phoneme_with_stress`, as CMUdict text does.
3. **Some CMUdict words do not survive a round trip** (found by the 3.1.2 fuzz harness). The word
   is what comes before `(` after the line is trimmed, so `(x  AH0` stores the word `""` and
   `read (1)  R EH1 D` stores `"read "`. Their export is a line whose word is gone or re-trimmed,
   so it reads back as a different dictionary. The Rust stores both the same way. The fix is to
   trim after stripping the `(n)` suffix and refuse an empty word.
4. **Phonotactic validation covers English, Sanskrit and Japanese only**, the Rust's three
   profiles. varna 2.6.0 has `phonotactics_by_code` for twelve languages and more than one
   inventory per language (`registry_inventories`, e.g. Brazilian Portuguese as `pt-BR`).
   Extending `shbdk_dict_validate_phonotactics` would be additive, but still a divergence.
5. **A frequency of exactly -1.0 reads back as no frequency.** `SHBDK_FREQ_NONE` is the f64 -1.0
   bit pattern, chosen because the Rust sorts a missing frequency as `unwrap_or(-1.0)`. -1.0 is
   outside the documented 0.0–1.0 contract. Moving the sentinel to a NaN payload would change what
   `shbdk_pronunciation_frequency` returns for an unset frequency, which a consumer may sort on,
   so it is not a patch-release change.

## Gated on something outside this repository

- **`@freq` precision.** A value needing more than six decimals is written rounded to six
  (`1/3` → `0.333333`, and `1e-9` → `0`), where the Rust writes the shortest f32 form
  (`0.33333334`). Closing it needs an f32 shortest-round-trip formatter, which neither the stdlib
  (`fmt_float_buf` is fixed-decimal) nor bayan (Grisu2 is shortest-or-*near*-shortest, and pairs
  badly with `f64_parse`'s 1-ULP error) provides. Out of contract in practice.
- **`lazy_open`'s blob fallback is not exercised on Linux**, where mmap always succeeds. Opening a
  directory bails earlier (`xlseek` gives EINVAL), so reaching the branch needs an AGNOS run. The
  contract it must keep (`lazy_open` and `load_binary_file` decode a file to the same dictionary)
  is tested.
- **Lighter bundles for dictionary-only consumers.** A cross-crate question with two parts:
  - **bayan**: shabdakosh folds the 995 KB `bayan` monolith (with 353 KB of unicode tables at
    cyrius 6.7.18), where `dist/bayan-json.cyr` would do. shabda folds the monolith too, and a
    consumer must not hold both, so it is a stack decision, not a pin bump. varna 2.6.0's
    optional JSON support already uses `bayan-json`.
  - **svara**: shabdakosh uses only svara's `SVARA_PH_*` ordinals, but folds the whole synthesis
    bundle. A phoneme-only svara sub-bundle would be svara's to ship.

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

## Out of scope

- **C FFI** (`ffi.rs`) — dropped as redundant: CYRIUS interops with C directly, so a dedicated FFI
  shim buys nothing, and there are no C-ABI consumers today. A backdoor exists to add one if needed.
