# Backlog — shabdakosh (last swept for v3.0.5)

Non-blocking improvements surfaced by the post-release backlog review (2026-07-05). The v3.0.0
release gates are all met (see [state.md](state.md)); these are tracked for a follow-up. Format:
**[value/effort]**.

## Gated on upstream (cyrius const-eval) — the path to retiring `rust-old/`

Sequencing (owner: the CYRIUS author): **(1)** finish the remaining SIMD work for aarch64 →
**(2)** the **const-eval / comptime arc** (proposal `2026-07-05-const-eval-comptime.md`). When
const-eval lands:

- **[high/medium] Bring `static_dict` up to a true compile-time perfect hash (phf).** Today it's a
  lazy cached singleton over `shbdk_dict_english()` (no const-eval to bake the table — one-time
  ~9.6 ms load; surface + lookup already match Rust). Once const-eval exists, replace the singleton
  with a compile-time-baked perfect hash to reclaim the zero-load property the Rust `phf` had. This
  is the **only** feature the port intentionally left at a lesser fidelity; everything else is at
  parity or consciously dropped (ffi).
- **[—] Final `rust-old/` parity sweep, then drop the oracle.** With phf closed, do one last
  function-for-function review of the port against `rust-old/`, confirm nothing else diverges, then
  **remove `rust-old/`** — the port becomes self-standing and the Rust source is retired.

## Parity gaps (vs `rust-old/`)

_None open._ The two text-format gaps listed here through 3.0.4 were closed in 3.0.5 (see
Resolved). One **residual**, documented rather than fixed: `@freq` values needing more than six
decimal places still print rounded to six (`1/3` → `0.333333`) where Rust prints the shortest f32
form (`0.33333334`). The port stores frequencies as f64 and the documented contract is 0.0–1.0, so
this is out of contract in practice; closing it would need an f32-precision shortest-round-trip
formatter that neither the stdlib (`fmt_float_buf` is fixed-decimals) nor bayan (Grisu2 is
shortest-or-*near*-shortest, and pairs badly with `f64_parse`'s 1-ULP error) currently provides.

## Cross-crate: the phoneme↔IPA bridge

- **[high/medium — needs an ADR] `dict.validate()` reports 5,468 of 10,617 built-in English entries
  as invalid.** Fully measured and written up in
  [architecture note 001](../architecture/001-phoneme-ipa-bridge-and-validation.md). Not a port
  defect — `rust-old/` behaves identically. Three causes: varna spells affricates with the tie bar
  (`t͡ʃ`) while we emit the bare `tʃ` (1,070 entries); ARPABET `IY` maps to `SVARA_PH_VOWEL_E` → IPA
  `e` rather than `iː` (2,306); and varna's `en` inventory has no NURSE vowel and only two of five
  English diphthongs (3,358). Only the first is ours to fix, and doing so is a deliberate divergence
  from the oracle — hence the ADR. Decide before any consumer relies on `validate()`.

### NOT gaps — intentional AGNOS-native design (do not "fix")

The review flagged the JSON and binary formats as "not wire-compatible with the Rust crate." That
is **by design, not a defect** — the whole AGNOS stack is CYRIUS, and nothing consumes the old
Rust crate's output:

- **JSON is bayan.** The codec is built on **bayan** (`json_v_*`), the AGNOS-standard JSON DOM (a
  `[deps].stdlib` fold). Phonemes serialize as `SVARA_PH_*` ints — the CYRIUS-native schema. It is
  valid, standard JSON; it simply isn't the Rust crate's enum-string schema, and no AGNOS consumer
  wants that. Recorded in [ADR 004](../adr/004-cyrius-port-decisions.md).
- **Binary is the CYRIUS format** (hand-rolled, replacing postcard). Reuses the `SHBD` magic +
  version 1 by design. There are no Rust `.bin` blobs in the AGNOS world to collide with; the
  decoder is bounds-checked either way. Recorded in ADR 004.

## Test coverage

- **[low/small] `lazy_open`'s blob-fallback branch is still not exercised on Linux** — mmap always
  succeeds there. 3.0.5 probed the obvious lever and it does not work either: opening a DIRECTORY
  bails earlier, at the size check, because `xlseek` on a directory fd returns -22 (EINVAL). Reaching
  the branch needs an AGNOS run. What 3.0.5 *did* add is the contract the fallback must preserve —
  `lazy_open` and `load_binary_file` are asserted to decode the same file to the same dictionary —
  plus coverage of every reachable `lazy_open` failure path.

## Code quality / tech-debt

_None open._

Note for the next sweep: detect.cyr keeps its own insertion sort rather than the keystone's shared
`_shbdk_msort`, and that is deliberate — it sorts a handful of language codes, and the module is
self-contained by design so its suite can link it without the dictionary tier.

## Docs

_None open._

## Resolved during the review (not open)

- ~~`to_cmudict` emits `@freq` with 6 fixed decimals~~ — **fixed in 3.0.5**: trailing zeros and a bare
  trailing `.` are trimmed, so `0.5`/`0.85`/`1`/`0` come out in Rust's Display shape.
- ~~`parse_cmudict` scans `@freq`/`@region` anywhere on the comment line~~ — **fixed in 3.0.5**: the
  comment is tokenized on whitespace and each token prefix-matched, as Rust's
  `split_whitespace().strip_prefix()` does. `;;; note@freq=0.5` is no longer an annotation.
- ~~`from_simple_entries` untested~~ / ~~unknown-notation fallthrough arms untested~~ — **covered in
  3.0.5** (`tests/dict.tcyr`, `tests/notation.tcyr`), including the parity detail that
  `from_simple_entries` stores keys verbatim while `insert` lowercases.
- ~~Scattered NUL-terminated byte-copy helpers~~ — **collapsed in 3.0.5** onto one L0 primitive,
  `shbdk_copyz` (`src/error.cyr`); the five module-local helpers are now one-line wrappers and no
  longer disagree about the allocation-failure sentinel.
- ~~`SHBD` magic bytes are unnamed literals~~ — **named in 3.0.5** (`SHBDK_BIN_MAGIC_S/H/B/D`).
- ~~Duplicated cstr-compare helper~~ — **consolidated in 3.0.5** onto `shbdk_cstrcmp`
  (`src/error.cyr`). The original objection (it would couple two sibling modules) no longer
  applies now that L0 hosts the shared primitives: detect.cyr and validate.cyr both depend on
  L0, not on each other. Cost was one added include in `tests/detect.tcyr`.
- ~~`docs/examples/` is empty~~ — **filled in 3.0.5**: `quickstart.cyr` (look up a word, render three
  notations) and `roundtrip.cyr` (build → lookup → user override → CMUdict/IPA/JSON/PLS/SSML/binary
  round-trip → prefix search + coverage), both building and running from the repo root, plus a README
  with the build lines.

- ~~`merge` / `merge_conservative` share entry pointers where Rust deep-clones~~ — **fixed in
  3.0.4**: `shbdk_dict_entry_clone` / `shbdk_pronunciation_clone` added and used on both merge
  paths; regression in `tests/dict.tcyr` proves mutating the merged copy leaves the source alone.
  See [the P1 sweep](../audit/2026-09-01-p1-sweep.md).
- ~~`load_binary_file` blob-fallback path is not exercised on Linux~~ — still true for the AGNOS
  branch specifically, but `shbdk_load_binary_file` itself is now directly covered
  (`tests/binary.tcyr`: repeated small loads + a missing-file case).

- ~~Docs / memories still described the Rust crate (Rustisms + stale sharding/path-deps)~~ —
  **audited + cleaned** (2026-07-06): CONTRIBUTING rewritten to CYRIUS tooling, ADR-001/002/003
  given pre-port banners, sharding/path-dep/count references corrected across README, docs/, and
  memories (git+tag deps, single `_cmudict_data.cyr`, 26 suites / 689 assertions).
- ~~cmudict data sharding~~ — **reverted to a single `_cmudict_data.cyr`** (2026-07-06): the distlib
  256 KB → 1 MB cap fix shipped in toolchain 6.4.10 (from our proposal), so the 283 KB module fits
  again. Generator, 19 includers, and `[lib].modules` collapsed back to one file.
- ~~Six varna-lexicon dict constructors unported~~ — **ported** (`src/dictionary/lexicon.cyr`):
  `shbdk_from_lexicon` + `shbdk_dict_spanish/hindi/german/sanskrit` over varna's Swadesh API
  (12 assertions in `tests/lexicon.tcyr`). Was the top parity gap.
- ~~`to_binary` fixed-2 MB write buffer (heap overflow)~~ — **fixed** this pass (`_shbdk_bin_total_size`
  sizes the buffer to the dict). See [audit addendum](../audit/2026-07-05-audit.md).
- `bench` in `[deps].stdlib` — **intentional, not debt**: `cyrius bench` resolves `bench_*` from the
  project stdlib list, so `benches/hotpath.bcyr` needs it. It does not leak into the shipped bundle
  (built from `[lib].modules`).
