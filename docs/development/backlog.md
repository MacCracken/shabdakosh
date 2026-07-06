# Backlog — shabdakosh v3.0.0

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

- **[medium/small] `merge` / `merge_conservative` share entry pointers where Rust deep-clones.**
  `src/dictionary/mod.cyr:263`/`284` do `map_set(dst, k, map_get(src, k))` (shared `ShbdkDictEntry*`);
  Rust inserts `entry.clone()`. Observable: after `A.merge(B)`, mutating a shared entry affects both.
  Fix needs a `shbdk_dict_entry_clone` on the merge path.
- **[low/small] `to_cmudict` emits `@freq` with 6 fixed decimals** vs Rust's shortest round-trip f32
  (`0.5` → `0.500000`). Re-parses identically; cosmetic (`src/dictionary/format/mod.cyr:332`).
- **[low/small] `parse_cmudict` scans `@freq`/`@region` anywhere on the comment line**, not only at
  whitespace-token boundaries like Rust's `split_whitespace` (`src/dictionary/format/mod.cyr:189`).
  Slightly more lenient than the oracle.

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

- **[medium/small] `load_binary_file` blob-fallback path is not exercised on Linux** — `lazy_open`
  always takes the mmap branch there, so the `file_read_all` fallback (`binary.cyr:308`) is untested.
  Needs an AGNOS run or a direct `load_binary_file` test.
- **[low/small] `from_simple_entries` constructor untested** (`src/dictionary/mod.cyr:132`).
- **[low/small] Unknown-notation fallthrough arms untested** — `shbdk_notation_name` /
  `shbdk_notation_phoneme_to_str` return 0 for an out-of-range notation tag (`src/notation.cyr:146`).

## Code quality / tech-debt

- **[low/small] Duplicated cstr-compare helper** — `_shbdk_cstr_cmp` (`detect.cyr:66`) and
  `_shbdk_wordcmp` (`validate.cyr:31`) are byte-identical. Coexist only because they're differently
  named; consolidating would couple the two modules (low value).
- **[low/medium] Scattered NUL-terminated byte-copy helpers** — `_shbdk_dupz`, `_shbdk_bin_dup`, and
  two range-copy variants in `format/mod.cyr` differ only in source addressing. Candidate for one shared
  primitive.
- **[low/small] Magic buffer sizes / `SHBD` magic bytes are unnamed literals** — `8388608`/`8388607`
  (three sites), `2097152` (now the size fn), and `83/72/66/68` in `to_binary`. Name them consts.

## Docs

- **[medium/medium] `docs/examples/` is empty** (`.gitkeep` only) despite CLAUDE.md advertising runnable
  examples. Add an end-to-end CYRIUS example (build dict → lookup → user override → format roundtrip).
  Partially covered by [consuming-the-distlib.md](../guides/consuming-the-distlib.md).

## Resolved during the review (not open)

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
