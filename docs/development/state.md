# shabdakosh — Current State

> Refreshed as the port progresses. CLAUDE.md is preferences/process/procedures
> (durable); this file is **state** (volatile).

## Version

**3.0.0** (in progress) — Rust→Cyrius port. Targeting full behavioral parity with
the Rust 2.0.0 surface. Started 2026-07-05 via `cyrius port`. 7,085 lines of Rust
preserved at `rust-old/` as the parity oracle.

## Toolchain

- **Cyrius pin**: `6.4.6` (in `cyrius.cyml [package].cyrius`) — bumped from 6.4.5 when
  the installed cycc drifted to 6.4.6; `lib/` re-synced, all suites still green.

## Port decisions (locked 2026-07-05)

- **Base-dictionary data**: generated `.cyr`, NOT runtime-parsed. A `gen_cmudict.cyr`
  generator reads `data/cmudict-5k.txt` (300 KB / 10,692 lines) and emits the
  checked-in `src/dictionary/_cmudict_data.cyr` (chunked string globals + interned
  phoneme ints), loaded into `lib/hashmap.cyr` at startup. Faithful port of the Rust
  `build.rs`; matches the varna & cyrius-unicode precedent. Runtime `.cyml`/`.txt`
  parse was rejected (256 KB / 256-entry parser caps, no asset-path story).
- **Scope**: MAXIMAL — attempt phf / binary / mmap / ffi equivalents, not just
  core+varna. **wasm** is the one gap (Cyrius has no wasm target) — expose the API
  surface, document the gap.
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
| L2 | build.rs → gen | gen_cmudict.cyr + _cmudict_data.cyr | ⏳ next | — | codegen the base dict |
| L3 | dictionary/mod.rs | src/dictionary/mod.cyr | ⬜ | — | keystone: PronunciationDict, diff |
| L4 | dictionary/coverage.rs | … | ⬜ | — | |
| L4 | dictionary/stream.rs | … | ⬜ | — | |
| L4 | dictionary/trie.rs | … | ⬜ | — | HashMap<char,node> |
| L4 | dictionary/heteronym.rs | … | ⬜ | — | `&dyn` resolver |
| L4 | dictionary/g2p.rs | … | ⬜ | — | FallbackDict / G2PModel / FstModel stub |
| L4 | dictionary/static_dict.rs | … | ⬜ | — | phf variant (maximal scope) |
| L5 | dictionary/format/mod.rs | … | ⬜ | — | CMUdict/IPA/JSON |
| L5 | dictionary/format/pls.rs | … | ⬜ | — | W3C PLS XML |
| L5 | dictionary/format/ssml.rs | … | ⬜ | — | SSML phoneme tag |
| L5 | dictionary/format/binary.rs | … | ⬜ | — | postcard equiv (maximal scope) |
| L6 | dictionary/validate.rs | … | ⬜ | — | varna-gated |
| L6 | dictionary/detect.rs | … | ⬜ | — | varna-gated |
| L6 | dictionary/lazy.rs | … | ⬜ | — | mmap (lib/mmap.cyr) |
| L6 | ffi.rs | … | ⬜ | — | C ABI via cyrius header |
| L6 | wasm.rs | … | ⬜ | — | no Cyrius wasm target — surface + doc gap |

**7 of ~24 modules ported** — L1 leaf tier + L2 notation complete (L0 error; L1 arpabet, ipa,
entry, morphology, syllable; L2 notation). Build + smoke + tests green (273 assertions across
8 suites). `SHABDA_PH_NONE` sentinel lives in `error.cyr` (L0 base).

**Parity audit (7 auditors + adversarial verify) — 5 modules clean, 2 low-severity divergences
found & fixed + regression-tested:** (a) entry's freq sort bubbled a NaN frequency to primary
(Rust keeps it in place) → NaN-safe predicate; (b) notation's whitespace tokenizer missed VT/FF
→ added. Next: the CMUdict codegen (`build.rs` port), then the L3 `dictionary/mod` keystone.

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
