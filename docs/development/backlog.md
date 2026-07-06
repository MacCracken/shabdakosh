# Backlog — shabdakosh v3.0.0

Non-blocking improvements surfaced by the post-release backlog review (2026-07-05). The v3.0.0
release gates are all met (see [state.md](state.md)); these are tracked for a follow-up. Format:
**[value/effort]**.

## Parity gaps (vs `rust-old/`)

- **[high/medium] Six varna-lexicon dict constructors are not ported.** `from_lexicon`, `spanish`,
  `hindi`, `german`, `sanskrit` (and the lexicon-backed builder) exist in `rust-old/src/dictionary/mod.rs`
  but have no `shabda_dict_*` twin. A consumer expecting the Rust surface to seed a language dict from
  a varna lexicon will not find them. (Comment at `src/dictionary/mod.cyr:11` now flags this.)
- **[medium/small] `merge` / `merge_conservative` share entry pointers where Rust deep-clones.**
  `src/dictionary/mod.cyr:263`/`284` do `map_set(dst, k, map_get(src, k))` (shared `ShDictEntry*`);
  Rust inserts `entry.clone()`. Observable: after `A.merge(B)`, mutating a shared entry affects both.
  Fix needs a `shabda_dict_entry_clone` on the merge path.
- **[medium/medium] JSON codec is a CYRIUS-native integer-ordinal schema, not `serde_json`-wire-compatible.**
  Rust serializes phonemes as svara enum names ("PlosiveK"); the port uses `SVARA_PH_*` ints
  (`src/dictionary/format/json.cyr:13`). Roundtrips within CYRIUS; does not interop with Rust JSON.
  (Accepted — see [ADR 004](../adr/004-cyrius-port-decisions.md).)
- **[medium/small] Binary format reuses `SHBD`+version-1 but the payload is not postcard.** Same magic,
  incompatible wire encoding (`src/dictionary/format/binary.cyr:16`). Consider bumping the version byte
  or a distinct magic to avoid a Rust blob being mis-accepted. (Accepted — ADR 004.)
- **[low/small] `to_cmudict` emits `@freq` with 6 fixed decimals** vs Rust's shortest round-trip f32
  (`0.5` → `0.500000`). Re-parses identically; cosmetic (`src/dictionary/format/mod.cyr:332`).
- **[low/small] `parse_cmudict` scans `@freq`/`@region` anywhere on the comment line**, not only at
  whitespace-token boundaries like Rust's `split_whitespace` (`src/dictionary/format/mod.cyr:189`).
  Slightly more lenient than the oracle.

## Test coverage

- **[medium/small] `load_binary_file` blob-fallback path is not exercised on Linux** — `lazy_open`
  always takes the mmap branch there, so the `file_read_all` fallback (`binary.cyr:308`) is untested.
  Needs an AGNOS run or a direct `load_binary_file` test.
- **[low/small] `from_simple_entries` constructor untested** (`src/dictionary/mod.cyr:132`).
- **[low/small] Unknown-notation fallthrough arms untested** — `shabda_notation_name` /
  `shabda_notation_phoneme_to_str` return 0 for an out-of-range notation tag (`src/notation.cyr:146`).

## Code quality / tech-debt

- **[low/small] Duplicated cstr-compare helper** — `_shabda_cstr_cmp` (`detect.cyr:66`) and
  `_shabda_wordcmp` (`validate.cyr:31`) are byte-identical. Coexist only because they're differently
  named; consolidating would couple the two modules (low value).
- **[low/medium] Scattered NUL-terminated byte-copy helpers** — `_shabda_dupz`, `_shabda_bin_dup`, and
  two range-copy variants in `format/mod.cyr` differ only in source addressing. Candidate for one shared
  primitive.
- **[low/small] Magic buffer sizes / `SHBD` magic bytes are unnamed literals** — `8388608`/`8388607`
  (three sites), `2097152` (now the size fn), and `83/72/66/68` in `to_binary`. Name them consts.

## Docs

- **[medium/medium] `docs/examples/` is empty** (`.gitkeep` only) despite CLAUDE.md advertising runnable
  examples. Add an end-to-end CYRIUS example (build dict → lookup → user override → format roundtrip).
  Partially covered by [consuming-the-distlib.md](../guides/consuming-the-distlib.md).
- **[medium/small] ADR-001/002/003 describe Rust-crate mechanics the port replaced.** The index note
  ([adr/README.md](../adr/README.md)) flags this; a fuller reframe or `Superseded` status would be cleaner.

## Resolved during the review (not open)

- ~~`to_binary` fixed-2 MB write buffer (heap overflow)~~ — **fixed** this pass (`_shabda_bin_total_size`
  sizes the buffer to the dict). See [audit addendum](../audit/2026-07-05-audit.md).
- `bench` in `[deps].stdlib` — **intentional, not debt**: `cyrius bench` resolves `bench_*` from the
  project stdlib list, so `benches/hotpath.bcyr` needs it. It does not leak into the shipped bundle
  (built from `[lib].modules`).
