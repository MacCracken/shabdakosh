# Changelog

## [3.0.6] — 2026-09-01

Toolchain pin to cyrius **6.5.37** and a sidecar correction that unblocks consumers.

### Fixed — `dist/shabdakosh.deps` named three VENDORED deps as stdlib leaves

The sidecar listed `hisab`, `goonj` and `naad` among its stdlib requirements. None of them is
a cyrius stdlib module — they are vendored dependencies pulled in through `src/main.cyr`, the
umbrella that `cyrius distlib` scans for `include "lib/X.cyr"` lines. The scan correctly
captured "these must be in scope" but tagged them as *stdlib*, and a consumer's `cyrius deps`
then hunted for them in the stdlib and failed outright:

```
error: dep shabdakosh requires 'hisab' but it is not in the cyrius stdlib
```

⭐ **Over-reporting is the fatal direction.** An under-reported sidecar fails silently at the
consumer's compile step; an over-reported one is a HARD resolver error before compilation is
reached — the same shape that left `crab` and `puka` unable to resolve `dhancha` at all.

Fixed upstream in cyrius 6.5.37, which validates every captured leaf against the stdlib
snapshot at generation time, so the names are dropped by construction. Regenerating here picks
that up: **24 leaves → 21**, and all 21 now resolve.

⚠ No source change was needed or made. `src/main.cyr`'s includes are correct — they carry the
dependency chain the bundled modules require, and removing them fails the build with 42
undefined functions. Only the published metadata was wrong.

### Changed

- `cyrius` pin **6.5.36 → 6.5.37**.
- `lib/` re-synced from the 6.5.37 snapshot (34 files).
- `dist/shabdakosh.cyr` + `dist/shabdakosh.deps` regenerated.

### Known issue — `lib/sakshi.cyr` stays at 2.4.11

The pinned 6.5.37 snapshot ships sakshi **2.4.12** (which fixes a negative-depth buffer
underflow in `sakshi_span_enter`), and `cyrius lib sync` writes it correctly — but the next
`cyrius deps` reverts it to **2.4.11**, byte-identical to `../svara/lib/sakshi.cyr`. The
resolver is pulling a declared *stdlib* leaf from a named dep's own `lib/` instead of the
pinned snapshot, so the two commands fight and `deps` wins.

This is a cyrius resolver defect, not a shabdakosh one, and is filed upstream. It does not
affect this release: the build is clean and all 83 tests pass.


## [3.0.5] — 2026-09-01

Backlog sweep. Every actionable item on `docs/development/backlog.md` is now
closed — two parity gaps against `rust-old/`, three test-coverage gaps, three
tech-debt items and the empty examples directory — plus one further parity gap
found while closing them. The only entries left open are the two gated on
upstream Cyrius const-eval (`static_dict` perfect hash, then retiring the
oracle), one AGNOS-only branch that Linux cannot reach, and one newly-raised
cross-crate decision that needs an ADR. 26 suites / 873 assertions green
(was 821).

### Fixed

- **`@freq` is emitted in Rust's `Display` shape.** `to_cmudict` wrote six fixed
  decimals (`@freq=0.500000`); Rust writes `{freq}`, the shortest text that
  round-trips (`0.5`, `0.85`, `1`, `0`). Trailing zeros and a bare trailing `.`
  are now trimmed. *(bayan's Grisu2 writer was the obvious alternative and is
  worse here: it is shortest-or-**near**-shortest, and the stdlib's `f64_parse`
  lands 1 ULP off on values like `0.85`, so the pair emits
  `0.8500000000000001`. Fixed-6 rounding absorbs that.)*
- **`@freq` / `@region` are recognized only at whitespace-token starts.** Rust
  does `comment.split_whitespace()` then `token.strip_prefix("@freq=")`; the port
  scanned for `@` anywhere on the comment line, so it accepted
  `;;; note@freq=0.5`, which Rust ignores. The comment is now tokenized first.
- **VT and FF count as whitespace in the CMUdict parser.** `_shbdk_fmt_is_ws`
  covered space/tab/CR but not VT (11) or FF (12), which Rust's
  `trim()` / `split_whitespace()` do treat as whitespace — so `CAT\x0bK AE T`
  tokenized differently from the oracle. Same defect class as the `notation.cyr`
  tokenizer fixed at 3.0.0; this second copy was overlooked then. *(Found while
  closing the item above, not previously on the backlog.)*

### Changed

- **One shared NUL-terminated copy primitive.** `shbdk_copyz` (`src/error.cyr`)
  replaces five near-identical helpers — `_shbdk_fmt_dup`,
  `_shbdk_fmt_lower_dup`, `_shbdk_bin_dup`, `_shbdk_trie_dup_n`, `_shbdk_dupz`
  and `_shbdk_substr_cstr` — which differed only in how they addressed the
  source, and which disagreed about the allocation-failure sentinel (`0` vs
  `""`, where a `0` would have been dereferenced by several callers). They are
  now one-line wrappers over a primitive that clamps a negative length and
  uniformly returns `""`.
- **One shared cstring comparator.** `shbdk_cstrcmp` (`src/error.cyr`) replaces
  the byte-identical `_shbdk_cstr_cmp` (detect.cyr) and `_shbdk_wordcmp`
  (validate.cyr). The backlog had kept them apart because sharing would couple
  two sibling modules; that no longer applies now L0 hosts the shared primitives
  — both modules depend on L0, not on each other.
- **`SHBD` magic bytes are named** — `SHBDK_BIN_MAGIC_S/H/B/D` instead of bare
  `83/72/66/68` literals at both the write and the read site.

### Added

- **`docs/examples/` is no longer empty** (it held only a `.gitkeep` while
  CLAUDE.md advertised runnable examples). Two programs that build and run from
  the repository root, plus a README with the build lines and the five things
  they exist to demonstrate:
  - `quickstart.cyr` — open the built-in dictionary, look a word up, render it
    as IPA / ARPABET / X-SAMPA.
  - `roundtrip.cyr` — build a dictionary, look up, apply and remove a user
    override, round-trip through CMUdict text, IPA text, JSON, PLS, SSML and the
    binary format (in memory and on disk), then prefix-search and coverage over
    the built-in dictionary.
- **Test coverage for the three gaps the backlog listed** (+52 assertions):
  `from_simple_entries` (including the parity detail that it stores keys
  verbatim where `insert` lowercases); the `#[non_exhaustive]` fallthrough arms
  for an unknown notation tag; and every reachable `lazy_open` failure path plus
  the invariant its untestable branch must preserve — `lazy_open` and
  `load_binary_file` decode the same file to the same dictionary.
- **[Architecture note 001](docs/architecture/001-phoneme-ipa-bridge-and-validation.md)**
  — the phoneme↔IPA bridge, and why `shbdk_dict_validate(shbdk_dict_english())`
  reports 5,468 of 10,617 entries as invalid. Measured and attributed: varna
  spells affricates with the tie bar (`t͡ʃ`) while this crate emits the bare
  `tʃ` (1,070 entries); ARPABET `IY` maps to `SVARA_PH_VOWEL_E` → IPA `e` rather
  than `iː` (2,306); and varna's `en` inventory has no NURSE vowel and only two
  of the five English diphthongs (3,358). **Not a port defect** — `rust-old/`
  behaves identically — so closing the one part that is ours would be a
  deliberate divergence from the oracle and needs an ADR first. Tracked on the
  backlog; no behaviour changed in this release.

## [3.0.4] — 2026-09-01

Priority-1 audit sweep: correctness, security hardening, performance, and the two
parity/portability gaps the audit turned up. No API removals; two new public
functions and one new public constant. 26 suites / 821 assertions green (was 689) — 132 of them new
regressions for the defects below.

### Fixed

- **Corrupt text export from any cmudict-backed dictionary (HIGH).** Base-dictionary
  map keys are `Str` VIEWS into the packed `_cmudict_data` pieces: `str_len` is
  right, but the bytes are **not NUL-terminated**. `shbdk_to_cmudict`,
  `shbdk_to_cmudict_with_user`, `shbdk_to_ipa` and both `validate` report builders
  read those keys with `str_data()` and handed them to cstr APIs, which ran to the
  next NUL — the rest of the ~22 KB piece. `shbdk_to_cmudict(shbdk_dict_english())`
  emitted **128,695,456 bytes instead of 299,744**, every word followed by the
  packed tail of its shard. Six call sites now use `str_cstr()`. The tests missed
  it because they only ever exported dictionaries built by `parse_cmudict`, whose
  keys *are* NUL-terminated.
- **Stack overflow in `prefix_search` on a long word (HIGH, DoS).**
  `_shbdk_trie_collect` recursed once per path byte, so its stack depth was the
  word length — attacker-controlled (`from_binary` accepts u16-length words;
  `parse_cmudict` has no limit at all). A ~63 KB word already exhausted the 8 MB
  stack: a corrupt `.bin` plus one `prefix_search` was a reliable SIGSEGV. The walk
  is now iterative over preallocated frame arrays; a 2 MB word is fine.
- **`shbdk_lazy_open` had no file-size ceiling (MEDIUM, DoS).** The non-mmap loaders
  always capped at 8 MB and failed loud; the mmap path did not, so a huge or sparse
  file was mapped whole and fed to a decode loop whose length is proportional to
  the blob. Now capped identically.
- **`lazy.cyr` used a raw `syscall(8, …)` for `lseek` (MEDIUM, portability).** #8 is
  `lseek` only on x86-64 Linux/macOS — on aarch64 Linux it is a different call, and
  on AGNOS `lseek` is #58, so the mmap path misbehaved off x86-64. It now uses
  io.cyr's `xlseek`, whose `SYS_LSEEK` is resolved per target.
- **`merge` / `merge_conservative` aliased entries (MEDIUM, parity).** Both inserted
  the source's `ShbdkDictEntry` pointer where Rust inserts `entry.clone()`, so
  after `a.merge(b)` an `add_pronunciation` through either dictionary mutated both.
  Entries are now deep-cloned. (Was the top open item in `docs/development/backlog.md`.)

### Changed

- **Dictionary-scale sorts are O(n log n) (performance).** `_shbdk_sort_strs`
  (mod.cyr) and `_shbdk_sort_by_word` (validate.cyr) were insertion sorts running
  over every dictionary key — ~56M byte-wise compares for the 10,617-entry built-in
  dict. Both now share one stable bottom-up merge sort, `_shbdk_msort`, in the
  keystone. Ordering and stability are unchanged. `detect.cyr`'s sort stays an
  insertion sort on purpose: it is self-contained by design (its test links it
  without the keystone) and sorts a handful of language codes.
- **File loaders size their buffer to the file.** `shbdk_load_cmudict_file` and
  `shbdk_load_binary_file` each allocated a flat 8 MB read buffer per call out of a
  bump allocator that never frees — a few hundred reloads exhausted memory
  regardless of file size. The 8 MB ceiling is unchanged (and now the named
  `SHBDK_MAX_FILE_BYTES`), but it is checked up front against the real size.
- **`prefix_search` sizes its path buffer to the trie.** It allocated a flat
  `prefix + 65536` bytes per call — 64 KB of permanent arena for every
  autocomplete keystroke — while *still* silently dropping any longer word. The
  trie now tracks its longest word (`PrefixTrie` grew to 24 bytes) and the buffer
  is exact: nothing is dropped, and a normal dictionary needs ~30 bytes.
- **`detect_script` decodes the word once.** It re-walked the word's UTF-8 for each
  of varna's 19 scripts and heap-allocated an out-cell per walk; it now decodes
  once into a vec and the out-cell is a stack slot.

### Added

- **`shbdk_dict_entry_clone` / `shbdk_pronunciation_clone`** — deep copies (Rust's
  derived `Clone`), used by the merge fix and useful to consumers directly.
- **Hardening across the allocation and callback paths** (the zero-panic contract):
  every input-proportional `alloc()` is checked for the 0 that `alloc` returns on
  failure rather than writing through it; the byte-range copy helpers clamp an
  inverted range; `shbdk_to_binary` fails as an empty `Str` and
  `shbdk_save_binary_file` refuses to write it; a 0 heteronym resolver or 0 G2P
  `predict_fp` no longer becomes an indirect call to address 0; and
  `shbdk_dict_lookup_entry(d, 0)` returns "not found" instead of dereferencing null.
- **`to_cmudict` / `to_ipa` benchmarks.** The sorted-export path was unbenched,
  which is how both defects above survived three releases.

### Performance (10,617-entry built-in English dictionary)

| operation | 3.0.3 | 3.0.4 |
|---|---|---|
| `to_cmudict` | 1,780 ms / 539 MB arena / 128 MB output | 18 ms / 2 MB / 300 KB output |
| `to_ipa` | 1,892 ms / 540 MB arena | 18 ms / 2.6 MB |
| `to_pls` | 873 ms | 25 ms |
| `dict.validate()` | 317 ms / 67 MB arena | 56 ms / 3.4 MB |
| `prefix_search` ×100 | 16.1 ms / 7.0 MB arena | 6.3 ms / 0.8 MB |
| whole probe run, total arena | 1,194 MB | 48 MB |

Lookup is unchanged at ~128 ns (hit) / ~153 ns (miss).

## [3.0.3] — 2026-08-31

Toolchain + dependency refresh to the current AGNOS chain. One mechanical source
change (a renamed bayan entry point); no API or behavior change — the
689-assertion suite (26 files) passes unchanged, benchmarks are flat, and the
checked-in CMUdict codegen regenerates byte-identically.

- **Changed**: toolchain pin 6.4.12 → **6.5.36** (current release; clears the
  wrapper/manifest drift warning). `lib/` re-synced from the 6.5.36 snapshot.
- **Changed**: svara pin 3.1.0 → **3.5.4**, which carries its own dependency
  refresh (hisab 2.6.7 → 2.11.2, naad 2.1.1 → 2.2.1, goonj 2.0.0 → 2.0.4) and
  sakshi 2.4.2 → 2.4.11. shabdakosh consumes only svara's `SVARA_PH_*` phoneme
  identities, which are unchanged.
- **Changed**: varna pin 2.0.0 → **2.4.1** (adds `tone` + `features` modules
  upstream). The script-range / inventory / phonotactics / Swadesh surfaces
  `validate.cyr`, `detect.cyr`, and `lexicon.cyr` consume are unchanged.
- **Changed**: `[deps].stdlib` gains `slice` and `result` — varna 2.4.1's
  `dist/varna.deps` sidecar declares both as required leaves.
- **Fixed**: `shbdk_from_json`, `shbdk_validation_report_from_json`, and
  `shbdk_phonotactic_report_from_json` now call `json_v_parse_buf`. bayan renamed
  the cstr+len parse entry `json_v_parse_str` → `json_v_parse_buf` because
  Cyrius routes `X(str, …)` to `X_str`, which silently mis-dispatched the old
  name; without the rename the build fails on an undefined function.
- **Changed**: `dist/shabdakosh.cyr` regenerated at 3.0.3. Its
  `dist/shabdakosh.deps` sidecar now lists the full stdlib leaf set (24 entries)
  rather than just the three git-dep names — a distlib generator improvement in
  the newer toolchain.

## [3.0.2] — 2026-07-06

Dependency + toolchain pin hygiene. No API or behavior change — the 715-assertion
suite passes unchanged and the distlib is regenerated.

- **Changed**: svara pin 3.0.1 → **3.1.0** (its control-rate-glide synthesis
  release), and the dep block gains `path = "../svara"` (canonical path-for-local /
  git+tag-for-CI form). shabdakosh only consumes svara's `SVARA_PH_*` phoneme
  identities — unchanged in 3.1.0 — so this is a pin refresh for a consistently
  versioned chain, not a functional change.
- **Changed**: toolchain pin 6.4.10 → **6.4.12** (current release; removes drift,
  aligns with svara/shabda).

## [3.0.1] — 2026-07-06

Symbol-prefix correction ahead of the first downstream consumer (**shabda**, the G2P engine).
The whole public surface was renamed off the `shabda_` namespace so shabda — which links flat
against this distlib — can own it; the dictionary *shabda-kosh* now owns `shbdk_`. Purely
mechanical: the 689-assertion suite passes unchanged and the distlib is regenerated.

- **Breaking**: public prefix renamed `shabda_` → `shbdk_`, `SHABDA_` → `SHBDK_`, and `Sh<Name>` structs → `Shbdk<Name>` (269 exported symbols). API shape and semantics are unchanged. Done before any external consumer existed, so no real-world break — hence a patch, not a major.
- **Changed**: `dist/shabdakosh.cyr` regenerated with the new prefix; the `_cmudict_data.cyr` codegen symbols became `_shbdk_cmudict_piece` / `_SHBDK_CMUDICT_PIECE_*` (generator + checked-in data + consumer kept in lockstep).
- **Fixed**: `scripts/version-bump.sh` rewritten for the Cyrius toolchain — it edited a removed root `Cargo.toml` and ran `cargo generate-lockfile` (both Rust-era dead paths). It now writes `VERSION` (the `${file:VERSION}` source of truth) and regenerates the distlib bundle via `cyrius distlib` so `dist/shabdakosh.cyr` carries the version.

## [3.0.0] — 2026-07-05

Complete port of shabdakosh from Rust to the **CYRIUS** language (AGNOS toolchain). A
full-parity port: every Rust module reproduced against the preserved `rust-old/` oracle and
cross-checked by a 689-assertion suite across 26 test groups, plus a consumer-verified distlib
bundle (`dist/shabdakosh.cyr`).

- **Breaking**: Language change — shabdakosh is now a CYRIUS (`.cyr`) library, not a Rust crate. The API is flat, `shabda_`-prefixed C-style functions (`shabda_dict_lookup`, `shabda_parse_cmudict`, …) rather than Rust methods/traits/generics. Consumers pull `dist/shabdakosh.cyr`.
- **Breaking**: Errors are **sakshi** packed-i64 codes (`0 == ok`) instead of `thiserror` enums; fallible functions return a payload pointer (`0` == none) or a packed error (test with `shabda_is_err`).
- **Breaking**: Traits → function-pointer dispatch (G2P `FallbackDict`, heteronym resolver) and enum-tag dispatch (notation, lookup source); `Option<T>` → sentinels; `Result<T>` → pointer-or-0.
- **Feature**: full pronunciation surface — base dictionary (ARPABET/IPA↔svara phonemes), user overlay, O(1) hashmap lookup (~135 ns/hit), merge/diff, coverage, streaming lookup, prefix trie, heteronym context, G2P fallback chain.
- **Feature**: I/O formats — CMUdict / IPA / PLS / SSML text codecs (hand-written), JSON via the bayan DOM, and a compact hand-rolled binary format (replacing postcard); `LazyDict` (mmap-backed with a `file_read_all` fallback).
- **Feature**: varna-backed inventory + phonotactics **validation**, script/language **detection** (with a UTF-8 code-point decoder), and **Swadesh seed-dictionary constructors** (`shabda_from_lexicon`, `shabda_dict_spanish`/`hindi`/`german`/`sanskrit`).
- **Feature**: WASM binding surface (`WasmDict`) and the static dictionary (`static_dict`) ported as `.cyr` modules.
- **Feature**: base CMUdict data generated as a single checked-in `.cyr` module (`gen_cmudict.cyr`, the `build.rs` port); it fits under the distlib 1 MB per-module cap.
- **Changed**: `phf` static dict → lazy cached singleton — CYRIUS has no compile-time perfect hash; surface + lookup preserved, one-time ~9.6 ms load instead of a compile-time-baked table.
- **Removed**: C FFI (`ffi.rs`) — dead in the CYRIUS/AGNOS stack (no C-ABI consumers).
- **Removed**: the Rust `cli` binary and criterion harness — replaced by `.tcyr` tests and `benches/hotpath.bcyr` (see `docs/benchmarks.md`).

## [2.0.0] — 2026-04-01

Major release for shabda integration. Adds unified phoneme notation, syllabification,
morphological decomposition, and the complete v1.2–v1.4 feature set.

- **Breaking**: svara dependency bumped from 1.x to 2.0.0
- **Breaking**: `lookup_entry()` fast path skips `to_lowercase()` allocation when input is already lowercase — semantics unchanged, but internal storage relies on lowercase keys
- **Breaking**: `PrefixTrie` children switched from `BTreeMap` to `HashMap` — `search_prefix()` results are still sorted, but serialization order may differ
- **Breaking**: `#[non_exhaustive]` added to all public structs (`CoverageReport`, `HeteronymContext`, `InvalidEntry`, `ValidationReport`, `PhonotacticViolation`, `PhonotacticReport`, `StaticPronunciation`, `StaticEntry`, `Syllable`, `Morpheme`, `Decomposition`)
- **Feature**: `PhonemeNotation` trait — unified abstraction over ARPABET, IPA, and X-SAMPA (`notation` module)
- **Feature**: `Syllable`, `StressLevel`, `syllabify()` — syllable boundary detection using Maximal Onset Principle (`dictionary::syllable`)
- **Feature**: `Morpheme`, `MorphemeKind`, `Decomposition` — morphological decomposition tags for productive pronunciation (`dictionary::morphology`)
- **Feature**: `G2PModel` trait, `G2PResult`, `FallbackDict<M>` — neural/rule-based G2P fallback chain with confidence scores (`dictionary::g2p`)
- **Feature**: `FstModel` / `FstNotation` — Phonetisaurus WFST integration point
- **Feature**: Dictionary learning — `promote_prediction()`, `promote_if_confident()` on `FallbackDict`
- **Feature**: `PrefixTrie` — O(k) prefix search and autocomplete (`dictionary::trie`)
- **Feature**: Binary dictionary format via postcard — `to_binary()`, `from_binary()` (`binary` feature)
- **Feature**: PHF static dictionary — zero-allocation compile-time perfect hash lookups (`phf` feature, `dictionary::static_dict`)
- **Feature**: `LazyDict` — memory-mapped binary dictionary loading (`mmap` feature)
- **Feature**: `LookupStream` — zero-allocation streaming word-to-phoneme iterator
- **Feature**: Phonotactic validation — `validate_phonotactics()` detects forbidden sequences via varna constraints
- **Feature**: `CoverageReport` — text corpus coverage analysis
- **Feature**: `HeteronymResolver` trait — context-aware pronunciation selection for heteronyms
- **Feature**: C FFI — `extern "C"` API with opaque handles (`ffi` feature)
- **Feature**: WASM bindings — `WasmDict` via wasm-bindgen (`wasm` feature)
- **Feature**: `shabdakosh-cli` binary — import/export/merge/validate/diff/coverage/info (`cli` feature)
- **Performance**: Lookup 3x faster (skip allocation for lowercase inputs)
- **Performance**: Binary serialization 35% faster (zero-copy borrowing)
- **Performance**: Trie construction 18% faster (HashMap children)
- New feature flags: `binary`, `phf`, `mmap`, `ffi`, `wasm`, `cli`

## [1.1.0] — 2026-04-01

Multi-language foundation via varna integration.

- **Feature**: `varna` feature flag — optional dependency on varna for multi-language support
- **Feature**: Language-tagged dictionaries — `language()`, `set_language()`, `with_language()` for ISO 639 codes
- **Feature**: Inventory validation — `validate()` checks phonemes against varna's per-language inventories
- **Feature**: Lexicon ingestion — `from_lexicon()` converts `varna::lexicon::Lexicon` into a `PronunciationDict`
- **Feature**: Script detection — `detect_script()` identifies writing system from Unicode code points
- **Feature**: Language detection hint — `detect_language_hint()` suggests candidate languages by script
- **Feature**: Seed dictionaries — `spanish()`, `hindi()`, `german()`, `sanskrit()` constructors from varna's Swadesh lists
- **Feature**: New error variants — `PhonemeNotInInventory`, `UnknownLanguage`
- IPA length mark normalization in validation (handles convention differences)
- Serde backward compatibility: `language` field is optional, absent in v1.0 serialized data

## [1.0.0] — 2026-03-28

Initial stable release. 10K+ entry English dictionary with O(1) lookup, multiple notation
formats, and dictionary operations. Includes all pre-1.0 development (variant pronunciations,
metadata, IPA, PLS, SSML support).

- 10,600+ entry English pronunciation dictionary (CMUdict-derived, compile-time codegen)
- ARPABET-to-svara Phoneme bidirectional mapping (39 symbols)
- IPA module (`ipa.rs`) — bidirectional IPA-Phoneme mapping, `parse_ipa_word()`, `phonemes_to_ipa()`
- `PronunciationDict` with two-layer lookup: user overlay (BTreeMap) over base (hashbrown::HashMap)
- Variant pronunciations for 23 common heteronyms (read, live, wind, etc.)
- `Pronunciation` struct with `phonemes()`, `frequency()`, `region()` metadata
- `DictEntry` struct with multiple `Pronunciation` variants sorted by frequency
- `Region` enum (`GeneralAmerican`, `ReceivedPronunciation`)
- Import/export: CMUdict text, IPA text, W3C PLS XML, SSML `<phoneme>` tags, JSON (`json` feature)
- Dictionary merge (`merge()`, `merge_conservative()`) and diff (`diff()` → `DictDiff`)
- File I/O convenience wrappers (`std` feature)
- `no_std` + `alloc` support
- Serde Serialize + Deserialize on all types
- Criterion benchmarks for construction and lookup
