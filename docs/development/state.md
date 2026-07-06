# shabdakosh — Current State

> Refreshed as the port progresses. CLAUDE.md is preferences/process/procedures
> (durable); this file is **state** (volatile).

## Version

**3.0.0** (in progress) — Rust→Cyrius port. Targeting full behavioral parity with
the Rust 2.0.0 surface. Started 2026-07-05 via `cyrius port`. 7,085 lines of Rust
preserved at `rust-old/` as the parity oracle.

## Toolchain

- **Cyrius pin**: `6.4.7` (in `cyrius.cyml [package].cyrius`) — bumped 6.4.5→6.4.6→6.4.7 as
  the installed cycc drifted; `lib/` re-synced each time, all suites stay green.

## Port decisions (locked 2026-07-05)

- **Base-dictionary data**: generated `.cyr`, NOT runtime-parsed. A `gen_cmudict.cyr`
  generator reads `data/cmudict-5k.txt` (300 KB / 10,692 lines) and emits the
  checked-in `src/dictionary/_cmudict_data.cyr` (chunked string globals + interned
  phoneme ints), loaded into `lib/hashmap.cyr` at startup. Faithful port of the Rust
  `build.rs`; matches the varna & cyrius-unicode precedent. Runtime `.cyml`/`.txt`
  parse was rejected (256 KB / 256-entry parser caps, no asset-path story).
- **Scope**: MAXIMAL — attempt phf / binary / mmap equivalents, not just
  core+varna. **FFI dropped** (2026-07-05): `ffi.rs` is NOT ported — FFI is dead
  in the CYRIUS stack (no C-ABI consumers). **cx** (bytecode backend) is the
  intended portable/browser target path, but it needs indirect-call (`callptr`)
  support first — shabdakosh's G2P fallback uses fn-pointers, which the cx backend
  currently rejects; the language author is adding cx support later. **wasm**
  (verified 2026-07-05): the WasmDict binding surface is
  ported as a normal `.cyr` module (`src/wasm.cyr`, `shabda_wasm_dict_*`), thin
  wrappers over the dict with the same 12 methods and the same JSON boundary
  shapes. How that surface reaches a browser is the toolchain's concern, not this
  crate's — not a "gap" to design around here. (Toolchain note for reference:
  `cyrius build` targets are `--aarch64` / `--win` / `--agnos` / `--target=js`;
  `--target=js` is a TS/TSX→JS frontend — `hello.cyr` → "ts parse error" — and
  there is no `--target=wasm` backend in this checkout. The only "wasm" strings
  ecosystem-wide are Wasmtime CVE citations.)
- **Serialization** (locked 2026-07-05): text formats (CMUdict/IPA/PLS/SSML) are
  hand-written permanent code (as in Rust — not serde). Scalar records use
  `#derive(Serialize)`. JSON dict I/O is hand-written in the format layer as
  permanent code (NOT a throwaway per-type derive stub — honors svara's precedent).
  No per-type `#derive` on container types.
- **Errors** (locked 2026-07-05): built on **sakshi** (`lib/sakshi.cyr`, the AGNOS
  error/tracing substrate) — packed i64 `[ctx][category][code]`, `0 == ok`. The Rust
  `ShabdakoshError` variants → `SHABDA_ERR_*` codes + `shabda_err_name()`.
- **Collections**: base dict → `lib/hashmap.cyr` (hashbrown replacement). User overlay
  BTreeMap → hashmap + sort-on-export (no Cyrius BTreeMap).
- **Naming**: every symbol prefixed `shabda_`/`SHABDA_`/`SH_`/`Sh` (flat link
  namespace, coexists with svara/varna).

## Port progress (module by module)

Order: leaves → notation + CMUdict codegen → dictionary keystone → extensions →
formats → gated/optional.

| Tier | Rust module | Cyrius | Status | Tests | Notes |
|------|-------------|--------|--------|-------|-------|
| — | (scaffold) | src/main.cyr | ✅ green | 2 | `cyrius build` OK; smoke 2/2 |
| L0 | error.rs | src/error.cyr | ✅ ported | 15 | ShabdakoshError → sakshi packed codes + `shabda_err_name`; fmt/lint clean |
| L1 | arpabet.rs | src/arpabet.cyr | ✅ ported | 61 | ARPABET ↔ `SVARA_PH_*`; svara path-dep wired; fmt/lint clean |
| L1 | ipa.rs | src/ipa.cyr | ✅ ported | 74 | IPA ↔ `SVARA_PH_*`; greedy longest-byte-match parser (tie-bar affricates), phonemes↔string; fmt/lint clean |
| L1 | dictionary/entry.rs | src/dictionary/entry.cyr | ✅ ported | 27 | Pronunciation/Region/DictEntry; sentinels for Option<freq/region>; NaN-safe freq-desc insertion sort (parity-audited); container serde in L5 |
| L1 | dictionary/morphology.rs | src/dictionary/morphology.cyr | ✅ ported | 20 | MorphemeKind/Morpheme/Decomposition; composite/root/prefixes/suffixes; self-contained |
| L1 | dictionary/syllable.rs | src/dictionary/syllable.cyr | ✅ ported | 23 | StressLevel/Syllable + syllabify (Maximal Onset); is_nucleus = ordinal 0..19; self-contained |
| L2 | notation.rs | src/notation.cyr | ✅ ported | 51 | PhonemeNotation trait → notation-tag dispatch; ARPABET/IPA/X-SAMPA; new X-SAMPA table; parse/render; ASCII-whitespace parity-audited |
| L2 | build.rs → gen | programs/gen_cmudict.cyr + _cmudict_data.cyr + cmudict.cyr | ✅ done | 15 | generator reuses arpabet.cyr (no table dup); emits 283KB/12-piece packed data (10,617 words, 0 unknown); cmudict.cyr loader → lib/hashmap |
| L3 | dictionary/mod.rs | src/dictionary/mod.cyr | ✅ ported | 39 | PronunciationDict (base+overlay maps), lookup (overlay→base, lowercase fast-path), merge/merge_conservative, diff/DictDiff, english()/english_minimal(); g2p/trie/varna/serde methods deferred to their tiers |
| L4 | dictionary/coverage.rs | src/dictionary/coverage.cyr | ✅ ported | 22 | CoverageReport + coverage(): tokenize/strip-punct/lowercase, dedup+sort uncovered; coverage_pct as f64 |
| L4 | dictionary/stream.rs | src/dictionary/stream.cyr | ✅ ported | 12 | LookupStream iterator → stateful cursor (zero-alloc per step); next/word/phonemes/size_hint |
| L4 | dictionary/trie.rs | src/dictionary/trie.cyr | ✅ ported | 30 | PrefixTrie (byte-keyed vec-of-pairs nodes, recursive collect); from_dict; wires keystone prefix_search |
| L4 | dictionary/heteronym.rs | src/dictionary/heteronym.cyr | ✅ ported | 17 | HeteronymContext + resolver as fn-ptr (fncall2); lookup_with_context variant selection |
| L4 | dictionary/g2p.rs | src/dictionary/g2p.cyr | ✅ ported | 29 | G2PResult, LookupSource, FallbackDict (model = (predict_fp,state) pair), promote*, FstModel stub; wires with_fallback |
| L4 | dictionary/static_dict.rs | … | ⬜ | — | phf variant (maximal scope) |
| L5 | dictionary/format/mod.rs | src/dictionary/format/mod.cyr + format/json.cyr | ✅ ported | 45 | CMUdict + IPA parse/emit, XML, file I/O; **hand-written PronunciationDict JSON codec** (bayan DOM both ways) — the serde-stance deliverable |
| L5 | dictionary/format/pls.rs | src/dictionary/format/pls.cyr | ✅ ported | 10 | W3C PLS XML parse/emit (ipa alphabet), hand-rolled scan; to_pls/to_pls_with_user |
| L5 | dictionary/format/ssml.rs | src/dictionary/format/ssml.cyr | ✅ ported | 13 | SSML <phoneme alphabet ph>word</phoneme> parse/emit; reuses pls XML scan helpers |
| L5 | dictionary/format/binary.rs | src/dictionary/format/binary.cyr | ✅ ported | 20 | hand-rolled compact binary format (SHBD magic+version, LE, 1-byte phonemes); to/from_binary + file I/O |
| L6 | dictionary/validate.rs | src/dictionary/validate.cyr | ✅ ported | 36 | inventory + phonotactics validation vs varna; 4 report structs w/ hand-written bayan JSON codecs; ː length-normalization; is_consonant parity quirk (omits VOWEL_LONG_I); dict.validate[_phonotactics] convenience |
| L6 | dictionary/detect.rs | src/dictionary/detect.cyr | ✅ ported | 24 | script + language detection vs varna script ranges; UTF-8 code-point decoder; majority vote; language-hint filter+sort; script-name out-params |
| L6 | dictionary/lazy.rs | src/dictionary/lazy.cyr | ✅ ported | 16 | LazyDict: mmap-open a binary dict (lib/mmap.cyr, real mmap on linux) + file_read_all fallback (AGNOS); eager decode like Rust; handle IS a dict; debug string |
| L6 | ffi.rs | — | ❌ dropped | — | FFI is dead in the CYRIUS stack (no C-ABI consumers) — not ported, by decision 2026-07-05 |
| L6 | wasm.rs | src/wasm.cyr | ✅ ported | 23 | WasmDict handle over inner dict; 12 methods; lookup/prefix_search/coverage cross as JSON (bayan); JSON IPA roundtrip test |

**7 of ~24 modules ported** — L1 leaf tier + L2 notation complete (L0 error; L1 arpabet, ipa,
entry, morphology, syllable; L2 notation). Build + smoke + tests green (273 assertions across
8 suites). `SHABDA_PH_NONE` sentinel lives in `error.cyr` (L0 base).

**Parity audit (7 auditors + adversarial verify) — 5 modules clean, 2 low-severity divergences
found & fixed + regression-tested:** (a) entry's freq sort bubbled a NaN frequency to primary
(Rust keeps it in place) → NaN-safe predicate; (b) notation's whitespace tokenizer missed VT/FF
→ added.

**CMUdict codegen done (the `build.rs` port):** `programs/gen_cmudict.cyr` reads
`data/cmudict-5k.txt`, folds variants via a hashmap, and emits the checked-in
`src/dictionary/_cmudict_data.cyr` (283 KB, 12 packed-string pieces, 10,617 words, 0 unknown
symbols) — reusing the ported `arpabet.cyr` mapping (no Rust-style table duplication).
`src/dictionary/cmudict.cyr` parses the pieces into a `lib/hashmap` map at load. Verified:
`shabda_cmudict_english()` loads 10,617 words; `cat`→[K,AE,T], `a`→[schwa], heteronym `bass`→2
prons. Regen: `cyrius build programs/gen_cmudict.cyr build/gen_cmudict && ./build/gen_cmudict`.
Total 327 assertions / 10 suites, all green.

**L3 keystone done** (`dictionary/mod.cyr`): `PronunciationDict` = base hashmap + user-overlay
hashmap + language; `lookup`/`lookup_entry`/`lookup_all` (overlay→base, lowercase fast-path),
`insert*`/`remove_user`, `merge`/`merge_conservative`, `english()` (loads the generated dict) /
`english_minimal()` (29 words), and `diff`/`DictDiff` (sorted, deep entry equality). Methods
deferred to their tiers: `with_fallback` (g2p, L4), `prefix_search` (trie, L4), varna methods
(L6), JSON serde (format L5) — these get wired into `mod.cyr` as those modules land.
**Known divergence to revisit**: `merge` shares entry pointers where Rust deep-clones — observable
only if the merged-from dict is mutated afterward; flag for a future clone or an ADR.
**L4 extension tier COMPLETE** (coverage, stream, trie, heteronym, g2p) → 437 assertions /
15 suites. `prefix_search` + `with_fallback` now wired into the keystone. (Cleaned a stray
`deferred`-keyword lint warning in the committed trie.cyr.) A parity audit over L3+L4 is
planned before release.

**L5 format layer started** — `format/mod` TEXT formats done (CMUdict + IPA parse/emit, XML
escape/unescape, file I/O) → 465 assertions / 16 suites. Fixing this bite surfaced a **latent
bug in cmudict.cyr**: `f64_parse` takes a cstr byte-pointer (not a `str_new` Str), so heteronym
frequencies were silently parsing to 0 — now fixed + regression-tested (bass @freq=0.5). Next
sub-bites: the hand-written **PronunciationDict JSON codec** (the serde-stance deliverable),
then format/pls, format/ssml, format/binary.

**JSON codec done** (`format/json.cyr`, 19 tests → 484 assertions / 17 suites): hand-written
`to_json`/`from_json` for `PronunciationDict` via bayan's `json_v_*` DOM — the permanent
serde-replacement per the locked stance (round-trips words/phonemes/frequency/region/language/
user-overlay; invalid JSON → 0). Debugging it pinned down the **bayan cstr/Str contract**
(see memory). Next L5: format/pls, format/ssml, format/binary.

**format/pls + format/ssml done** → 507 assertions / 19 suites.

**L5 FORMAT TIER COMPLETE** (format/mod text I/O, JSON codec, pls, ssml, binary).

**L3–L5 parity audit (11 auditors + adversarial verify) — 7 clean, 4 modules had issues; the
real bugs are FIXED + regression-tested** (533 assertions / 20 suites):
- parse_cmudict silently accepted a malformed `@freq` / unknown `@region` → now aborts
  (returns 0), matching Rust's Err (`f64_parse_ok` validates the freq token).
- to_cmudict emitted a stray leading/double space for phonemes with no ARPABET mapping →
  separator now only BETWEEN emitted symbols (same class as the earlier notation-render fix).
- diff resolved words by a direct map lookup → now routes through `lookup_entry` (re-lowercase)
  so non-lowercased `from_entries` keys classify like Rust.
- trie `search_prefix` collect buffer could overflow on a pathological >512-byte word →
  bounded buffer (plen+64K) + depth clamp.
- file-load buffers bumped 1–2 MB → 8 MB (no silent truncation for realistic dict files).
- freq precision: 6 decimals in both JSON and CMUdict emit (ample for a 0–1 score).

**Accepted divergences (documented, not "fixed"):** (a) case-folding is ASCII-only (like the
whitespace tokenizers) — accented capitals aren't lowercased; English dict words are ASCII.
(b) `xml_unescape` is single-pass — Rust's sequential `.replace()` OVER-decodes double-encoded
entities (`"&amp;lt;"`→`"<"`); the single pass (`"&amp;lt;"`→`"&lt;"`) is the CORRECT behavior.

**wasm.cyr done (2026-07-05)**: WasmDict surface ported as a `.cyr` module — 12 thin
wrappers over the dict, JSON boundary shapes preserved (lookup → IPA array, prefix_search →
array, coverage → `{total_tokens,covered_tokens,uncovered_words}` via bayan, key order
preserved). 23 assertions; full tree now **21 suites / 556 assertions** green. The earlier
"no wasm target — doc the gap" plan was corrected after verifying the toolchain (see Scope
note): the binding is just another `.cyr` surface; browser delivery is the toolchain's job.

**L6 complete (2026-07-05)**: lazy (16), detect (24), validate (36) all ported + green.
- **lazy.cyr** — LazyDict mmap-opens a binary dict (real mmap on linux via lib/mmap.cyr, lseek
  for size) with a file_read_all fallback for AGNOS; eager decode like Rust, so a lazy handle IS
  a dict handle. varna NOT needed.
- **detect.cyr** — script/language detection over varna's script ranges; adds a UTF-8 code-point
  decoder (CYRIUS has cstr byte pointers, no `char` iterator); majority vote, language-hint
  filter+sort, script-name out-params.
- **validate.cyr** — inventory + phonotactics validation against varna; 4 report structs each
  with a hand-written bayan JSON codec (serde-roundtrip invariant preserved); the ː
  length-normalization membership check (ɔ↔ɔː); faithful `is_consonant` parity quirk (Rust omits
  VOWEL_LONG_I → reads as consonant); dict.validate / dict.validate_phonotactics convenience.
- **varna** wired as a path dep (`lib/varna.cyr`, self-contained bundle, bare `phoneme_*`/`script_*`
  symbols — links cleanly alongside svara, no collision); phonemes bridge svara SVARA_PH_* ordinals
  → varna IPA strings via shabda_phoneme_to_ipa.

Full tree now **24 suites / 632 assertions** green. All Rust modules are ported (ffi dropped;
wasm ported as a `.cyr` surface). **Next: release prep** — distlib bundle (`cyrius distlib` →
dist/shabdakosh.cyr), benchmarks, CHANGELOG/roadmap finalization.

## Dependencies

- **stdlib** (declared): syscalls, string, alloc, str, fmt, vec, io, args, assert.
  Grows: `hashmap` (dict), `bayan` (serde), `tagged` (Option), `math` at their tiers.
- **svara** (2.0.0 Rust → 3.0.1 Cyrius, `dist/svara.cyr`) — `SVARA_PH_*` phoneme
  source. **Wired** (L1) as a path dep (`[deps.svara] path = "../svara"`); pulls
  the transitive stack (hisab/naad/goonj/sakshi). Entry includes
  `lib/{hisab,goonj,naad,svara}.cyr` before src modules. NOTE: the full bundle
  adds ~1 MB of unreachable code (DCE-eligible, `CYRIUS_DCE=1`) — a phoneme-only
  svara sub-bundle would lighten dictionary-only consumers; possible later.
- **varna** (2.0.0 Cyrius, `dist/varna.cyr`) — optional validate/detect. Added at the
  varna-gated tier.

## Consumers

shabda (G2P), dhvani (audio), vansh (voice shell) — will pull `dist/shabdakosh.cyr`.
