# Architecture Overview

> **shabdakosh** (Sanskrit: *dictionary*) — pronunciation dictionary for AGNOS.
> A **CYRIUS** (`.cyr`) library, not a Rust crate. It owns pronunciation lookup:
> a dictionary-first grapheme→phoneme store mapping words to sequences of
> svara `SVARA_PH_*` phonemes, with ARPABET/IPA/X-SAMPA bridges and
> CMUdict/PLS/SSML/JSON/binary I/O.

This is a Cyrius port of a 7,085-line Rust library (preserved at `rust-old/` as
the parity oracle). The surface is **flat, C-style, `shabda_`-prefixed
functions** — no methods, traits, generics, `use`, `Cargo.toml`, or crates.io.
Consumers pull the bundled `dist/shabdakosh.cyr`.

## Module Map

`src/main.cyr` is the smoke entry: it never lets modules include each other —
it orders the ~26 `.cyr` modules (stdlib + sakshi auto-resolve from
`cyrius.cyml [deps]`). The same list, in dependency order, is the distlib bundle
manifest (`cyrius.cyml [lib].modules`). Grouped by port tier:

```
shabdakosh/
├── lib/                            — vendored deps (do not edit); the svara chain
│   ├── hisab.cyr  goonj.cyr  naad.cyr  svara.cyr   — SVARA_PH_* phoneme source + backends
│   └── varna.cyr                   — phoneme inventories + script ranges (validate/detect)
│
├── src/
│   ├── main.cyr                    — smoke entry: include order + real-usage smoke calls
│   │
│   │  ── L0 error ──
│   ├── error.cyr                   — sakshi-packed error codes + shabda_err_name; SHABDA_PH_NONE sentinel
│   │
│   │  ── L1 leaves (notation bridges + record types) ──
│   ├── arpabet.cyr                 — ARPABET ↔ SVARA_PH_* (with/without stress)
│   ├── ipa.cyr                     — IPA ↔ SVARA_PH_*, greedy longest-byte-match parser
│   ├── notation.cyr                — ARPABET/IPA/X-SAMPA via a notation-TAG (not a trait)
│   └── dictionary/
│       ├── entry.cyr               — Pronunciation / Region / DictEntry (freq-desc sorted)
│       ├── morphology.cyr          — Morpheme / MorphemeKind / Decomposition
│       ├── syllable.cyr            — Syllable / StressLevel + syllabify (Maximal Onset)
│       │
│       │  ── L2 generated base data ──
│       ├── _cmudict_data_0.cyr     — packed-string shard, pieces 0–6 (172 KB)
│       ├── _cmudict_data_1.cyr     — packed-string shard, pieces 7–11 + count + accessor (110 KB)
│       ├── cmudict.cyr             — loads the shards into a lib/hashmap at runtime
│       │
│       │  ── L3 keystone ──
│       ├── mod.cyr                 — PronunciationDict: base map + user overlay + language;
│       │                            lookup/insert/merge/diff/english/english_minimal
│       │
│       │  ── L4 extensions ──
│       ├── coverage.cyr            — coverage report over tokenized text
│       ├── stream.cyr             — LookupStream: stateful zero-alloc cursor
│       ├── trie.cyr                — PrefixTrie: O(k) prefix search / autocomplete
│       ├── heteronym.cyr           — HeteronymContext + fn-pointer resolver
│       ├── g2p.cyr                 — FallbackDict (fn-pointer model), FstModel stub, promote*
│       ├── static_dict.cyr         — lazy cached singleton (phf replacement)
│       │
│       │  ── L5 formats ──
│       └── format/
│           ├── mod.cyr             — CMUdict + IPA parse/emit, XML escape, file I/O
│           ├── json.cyr            — hand-written PronunciationDict JSON codec (bayan DOM)
│           ├── pls.cyr             — W3C PLS XML parse/emit
│           ├── ssml.cyr            — SSML <phoneme> tag parse/emit
│           └── binary.cyr          — compact hand-rolled binary (SHBD magic, LE)
│       │
│       │  ── L6 gated / optional ──
│       ├── lazy.cyr                — LazyDict: mmap-open a binary dict (+ file_read_all fallback)
│       ├── detect.cyr              — script/language detection vs varna (UTF-8 decoder)
│       └── validate.cyr            — inventory + phonotactics validation vs varna
│   └── wasm.cyr                    — WasmDict: 12 thin JSON-boundary wrappers over the dict
│
├── data/cmudict-5k.txt            — base-dictionary source of truth (300 KB, 10,692 lines)
├── programs/gen_cmudict.cyr        — codegen: data → _cmudict_data_N.cyr shards (the build.rs port)
├── benches/hotpath.bcyr           — cyrius bench (criterion replacement)
├── tests/*.tcyr                    — per-module suites (cyrius tests)
└── dist/shabdakosh.cyr (+ .deps)   — the shipped distlib bundle
```

`ffi.rs` is **not** ported — C FFI is dead in the CYRIUS/AGNOS stack (no C-ABI
consumers). `wasm.rs` and `static_dict.rs` (phf) are ported as ordinary `.cyr`
modules.

## Data Flow

A pronunciation is a `vec` of svara `SVARA_PH_*` integer ordinals — compatible
with svara's `PhonemeEvent`. Lookup is dictionary-first: the user overlay shadows
the base map, both are `lib/hashmap` for O(1) access.

```
word ("hello")
  │
  └─→ shabda_dict_lookup(d, word)            (overlay → base, lowercase fast-path)
        │
        ├─ user overlay  (lib/hashmap)       ← application overrides, checked first
        └─ base dict     (lib/hashmap)        ← 10,617 generated entries
              │
              └─→ ShDictEntry
                    └─→ Pronunciation[]        (freq-desc, NaN-safe insertion sort)
                          ├── phonemes: vec<SVARA_PH_*>   → shabda_dict_entry_primary_phonemes
                          ├── frequency: f64 (sentinel = none)
                          └── region:    code (sentinel = none)
                    │
  notation out ─────┴─→ shabda_notation_render(SHABDA_NOTATION_XSAMPA, phonemes)  → "k { t"
                       shabda_phonemes_to_ipa(phonemes)          → "hɛloʊ"
                       shabda_phoneme_to_arpabet(ph)             → "AH0"

Fallible calls return a payload pointer (0 == none) or write to an out-param;
packed-error returns are tested with shabda_is_err / shabda_is_ok.

import paths (all return a PronunciationDict handle, 0 on parse failure):
  shabda_parse_cmudict(text)                  ← CMUdict text
  shabda_parse_ipa(text) / shabda_parse_ipa_word
  shabda_parse_pls(xml)                       ← W3C PLS
  shabda_parse_ssml_phoneme(xml)              ← SSML <phoneme>
  shabda_from_json(text) / shabda_from_binary(bytes)

extension surface (on a dict handle):
  shabda_dict_coverage(d, text)               → coverage report
  shabda_dict_lookup_stream(d, words)         → streaming cursor
  shabda_dict_prefix_search(d, "he")          → ["he","hello"]   (via trie)
  shabda_dict_lookup_with_context(...)        → heteronym resolution
  shabda_dict_with_fallback(d, &predict_fp, model)  → G2P fallback chain

varna-gated:
  shabda_detect_script("hello")               → "Latn"
  shabda_validate_inventory(d, phoneme_spanish())   → validation report
```

## Generated-Data Pipeline (the `build.rs` port)

Rust's `build.rs` emitted compile-time Rust. CYRIUS has no `build.rs`, so the
base dictionary is **checked-in generated `.cyr`** (matching the varna /
cyrius-unicode precedent). Runtime `.txt`/`.cyml` parse was rejected (256 KB /
256-entry parser caps, no asset-path story — see `state.md`).

```
data/cmudict-5k.txt          10,692 lines: "WORD  PH1 PH2", "WORD(n)  …" variants,
  │                          ";;; @freq=" / ";;; @region=" annotations
  │
programs/gen_cmudict.cyr     includes src/arpabet.cyr and REUSES its mapping —
  │                          one source of truth, no Rust-style table duplication.
  │                          Folds variants via a hashmap.
  │
  ├─→ src/dictionary/_cmudict_data_0.cyr   pieces 0–6   (packed string globals)
  └─→ src/dictionary/_cmudict_data_1.cyr   pieces 7–11 + word count + accessor
        │                                  (sharded: distlib caps per-module reads at 256 KB)
        │
        └─→ src/dictionary/cmudict.cyr
              ├─ shabda_cmudict_load(map)   → parses the pieces into a lib/hashmap (returns count)
              └─ shabda_cmudict_english()   → the loaded 10,617-word base map
                    │
                    └─→ shabda_dict_english()   (the L3 keystone reads this)
```

Regenerate after editing the data:
`cyrius build programs/gen_cmudict.cyr build/gen_cmudict && ./build/gen_cmudict`.

## Dependency Stack

Named deps need explicit `include`s in dependency order before the modules that
use them; stdlib folds (hashmap, bayan, sakshi, tagged, …) auto-resolve from
`cyrius.cyml [deps].stdlib`.

```
shabdakosh
  │
  ├── svara chain (path dep [deps.svara] = "../svara", pulls dist/svara.cyr)
  │     lib/hisab.cyr → lib/goonj.cyr → lib/naad.cyr → lib/svara.cyr
  │     └── SVARA_PH_* phoneme identities (the PhonemeEvent-compat contract) +
  │         the transitive backend chain (sakshi rides along)
  │
  ├── varna (path dep [deps.varna] = "../varna", pulls dist/varna.cyr)
  │     lib/varna.cyr — phoneme inventories + phonotactics + script ranges.
  │     Self-contained bundle; bare module-prefixed symbols (phoneme_*/script_*),
  │     NOT varna_-prefixed — links cleanly alongside svara, no collision.
  │     Bridged: SVARA_PH_* ordinals → varna IPA strings via shabda_phoneme_to_ipa.
  │
  └── stdlib folds ([deps].stdlib):
        syscalls, string, alloc, str, fmt, vec, io, args, assert, fnptr, atomic,
        sakshi (errors), math, ganita, tagged (Option-shaped returns),
        hashmap (the hashbrown replacement — base + overlay maps),
        bayan (JSON DOM codecs), mmap (LazyDict), bench.
```

There are no Rust-style feature flags. What Rust gated behind `std` / `json` /
`varna` / `phf` / `binary` / `mmap` features is here either always-on (`.cyr`
modules in the bundle) or resolved by the presence of a path dep. Unreachable
svara code is DCE-eligible (`CYRIUS_DCE=1`).

## Distlib Bundle

`cyrius distlib` concatenates the `[lib].modules` list (deps before dependents)
into a single shipped artifact:

```
dist/shabdakosh.cyr    458 KB, v3.0.0 — the whole library in one file
dist/shabdakosh.deps   sidecar listing the stdlib folds this bundle needs (hisab/goonj/naad)
```

Sharding the generated cmudict data into `_cmudict_data_0/1.cyr` was **required**
by distlib's 256 KB per-module read cap. The auto-generated `.deps` sidecar lists
only the hisab/goonj/naad leaves (a distlib heuristic); consumers therefore
declare `shabdakosh + svara + varna` deps plus the stdlib folds explicitly. A
consumer-side smoke (svara chain + varna + `dist/shabdakosh.cyr`) links and runs:
`shabda_dict_english()` loads all 10,617 entries, detect → `Latn`, wasm lookup →
JSON IPA.

## Design Principles (CYRIUS port invariants)

- **Flat prefixed namespace** — every symbol is `shabda_`/`SHABDA_`/`SH_`/`Sh`.
  The distlib links flat, so shabdakosh coexists with svara and varna without
  collision. No modules, no `use`, no re-exports.
- **sakshi errors, not `thiserror`** — errors are packed-i64 codes
  (`[ctx][category][code]`, `0 == ok`). The Rust `ShabdakoshError` variants map to
  `SHABDA_ERR_DICT_PARSE` / `SHABDA_ERR_UNKNOWN_SYMBOL` /
  `SHABDA_ERR_PHONEME_NOT_IN_INV` / `SHABDA_ERR_UNKNOWN_LANGUAGE`, with
  `shabda_err_name()` for diagnostic text. **Zero unwrap/panic** in library code.
- **Result / Option → sentinels & pointers** — `Result<T>` becomes a payload
  pointer (`0` == error/none) or a packed error written to an out-param, tested
  with `shabda_is_err` / `shabda_is_ok`. `Option<T>` becomes a sentinel value
  (e.g. `SHABDA_PH_NONE`, a `0` cstr, or a NaN/none frequency).
- **Fn-pointer & enum-tag dispatch instead of traits** — Rust's
  `PhonemeNotation` trait is a `SHABDA_NOTATION_*` tag switched inside
  `shabda_notation_render` / `shabda_notation_parse`. Rust's `G2PModel` trait and
  heteronym resolver are **function pointers** (`&shabda_fst_model_predict` passed
  to `shabda_dict_with_fallback`); the fallback model is a `(predict_fp, state)`
  pair. Lookup provenance is a `LookupSource` tag, not a trait object.
- **Hand-written codecs, no serde** — text formats (CMUdict/IPA/PLS/SSML) are
  permanent hand-written code; the PronunciationDict JSON codec
  (`shabda_to_json` / `shabda_from_json`) is hand-written over the bayan JSON DOM;
  the binary format (`shabda_to_binary` / `shabda_from_binary`) is a hand-rolled
  `SHBD`-magic little-endian layout replacing Rust's postcard. Every type
  round-trips; the binary deserializer bounds-checks all attacker-controlled
  length/count fields.
- **Dictionary-first, O(1) base lookup** — known-correct entries over algorithmic
  guessing; `lib/hashmap` for the base dict (~135 ns/hit, alloc-free fast-path).
  The user overlay is a hashmap too (CYRIUS has no ordered map); sorted export
  lives in the format layer (sort-on-export).
- **Additive enums** — the Rust `#[non_exhaustive]` invariant carries over: every
  tag `match`/`switch` keeps a `_ =>` catch-all arm.
- **cross-checked against `rust-old/`** — the correctness bar is "matches what
  Rust did"; documented divergences (ASCII-only case-folding, single-pass
  `xml_unescape`, pointer-sharing `merge`) live in `state.md`, not silent.

## Key Handles & Constants

| Name | Module | Purpose |
|------|--------|---------|
| `ShPronunciationDict` (24-byte handle) | `dictionary/mod.cyr` | base map + user overlay + language code |
| `ShDictEntry` | `dictionary/entry.cyr` | one or more pronunciations of a word |
| `Pronunciation` | `dictionary/entry.cyr` | `vec<SVARA_PH_*>` + frequency + region |
| `SHABDA_NOTATION_ARPABET/IPA/XSAMPA` | `notation.cyr` | notation-tag dispatch |
| `SHABDA_MORPH_PREFIX/ROOT/SUFFIX/INFIX` | `dictionary/morphology.cyr` | morpheme-kind tags |
| `SHABDA_FST_NOTATION_ARPABET/IPA` | `dictionary/g2p.cyr` | FST model notation tag |
| `SHABDA_ERR_*`, `shabda_err_name()` | `error.cyr` | sakshi error codes + text |
| `SHABDA_PH_NONE` | `error.cyr` | phoneme Option sentinel (L0 base) |

## Downstream Consumers

```
dist/shabdakosh.cyr
  ├─→ shabda   — G2P engine (dictionary lookup + rules fallback)
  ├─→ dhvani   — audio engine (pronunciation for TTS)
  └─→ vansh    — voice AI shell (user-facing pronunciation overrides)
```

## Quick Start

```sh
cyrius deps                                   # resolve dependencies
cyrius build src/main.cyr build/shabdakosh    # compile the smoke entry
cyrius test tests/<mod>.tcyr                   # run one suite
cyrius tests tests                            # run all .tcyr
cyrius bench                                   # benches/hotpath.bcyr
cyrius distlib                                 # rebuild dist/shabdakosh.cyr
```
