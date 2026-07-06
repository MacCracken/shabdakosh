# shabdakosh

[![version](https://img.shields.io/badge/version-3.0.0-blue.svg)](VERSION)
[![license](https://img.shields.io/badge/license-GPL--3.0--only-green.svg)](LICENSE)
[![language](https://img.shields.io/badge/language-CYRIUS-orange.svg)](cyrius.cyml)

**shabdakosh** (Sanskrit: *dictionary*) — the pronunciation dictionary for
[AGNOS](https://github.com/MacCracken). A **CYRIUS** (`.cyr`) library: a
dictionary-first grapheme→phoneme store mapping words to sequences of
[svara](https://github.com/MacCracken/svara) `SVARA_PH_*` phonemes, with
ARPABET/IPA/X-SAMPA notation bridges and CMUdict/PLS/SSML/JSON I/O.

> v3.0.0 is a full-parity **CYRIUS port** of a 7,085-line Rust library. It is no
> longer a Rust crate: the API is flat, `shabda_`-prefixed C-style functions
> (`shabda_dict_lookup`, `shabda_parse_cmudict`, …) — no methods, traits,
> generics, `Cargo.toml`, or crates.io. Consumers pull `dist/shabdakosh.cyr`.

## Features

- **Pronunciation lookup** — 10k+ entry base English dictionary (ARPABET/IPA ↔ svara phonemes), a user overlay, O(1) hashmap lookup (~135 ns/hit), plus `merge`/`diff`, coverage analysis, streaming lookup, a prefix trie, heteronym context, and a G2P fallback chain.
- **I/O formats** — CMUdict / IPA / PLS / SSML text codecs (hand-written), JSON via the bayan DOM, and a compact hand-rolled binary format. `LazyDict` gives mmap-backed loading (with a `file_read_all` fallback).
- **Validation & detection** — varna-backed phoneme-inventory and phonotactics validation, plus script/language detection over a UTF-8 code-point decoder.
- **WASM + static dict** — a `WasmDict` binding surface and a lazily-cached static dictionary singleton.
- **Generated data** — the base CMUdict is checked-in as a single `.cyr` module (`src/dictionary/_cmudict_data.cyr`, from `gen_cmudict.cyr`, the port of the Rust `build.rs`); it fits under the distlib 1 MB per-module cap (toolchain 6.4.10).
- **Sakshi errors** — no panics; fallible functions return a sakshi packed-i64 code (`0 == ok`, test with `shabda_is_err`) or a payload pointer (`0` == none).

## Quick Start

```sh
cyrius deps                                  # resolve dependencies (svara, varna, stdlib)
cyrius build src/main.cyr build/shabdakosh   # compile the smoke binary
cyrius tests tests                           # run all .tcyr suites
```

## Usage

Names are flat and `shabda_`-prefixed. A pronunciation is a `vec` of svara
`SVARA_PH_*` integer ordinals; `shabda_dict_lookup` returns that vec, or `0`
when the word is absent.

```cyrius
# Load the base English dictionary (10k+ entries).
var d = shabda_dict_english();

# Look up a word -> vec of SVARA_PH_* phonemes, or 0 if not found.
var ph = shabda_dict_lookup(d, "hello");
if (ph == 0) {
    println("not found");
} else {
    println_int(vec_len(ph));   # 4 phonemes: HH AH0 L OW1
}

# Extend with an application-specific pronunciation (user overlay wins).
var agnos = vec_new();
vec_push(agnos, SVARA_PH_VOWEL_ASH);
vec_push(agnos, SVARA_PH_PLOSIVE_G);
vec_push(agnos, SVARA_PH_NASAL_N);
vec_push(agnos, SVARA_PH_VOWEL_O);
vec_push(agnos, SVARA_PH_FRIC_S);
shabda_dict_insert_user(d, "agnos", agnos);

# Import CMUdict text; export JSON / binary.
var parsed = shabda_parse_cmudict("cat  K AE1 T\ndog  D AO1 G\n");
var json = shabda_to_json(parsed);
```

See [`docs/guides/getting-started.md`](docs/guides/getting-started.md) and
[`docs/guides/usage.md`](docs/guides/usage.md) for worked examples across all
formats and operations.

## Consuming the distlib

Downstream AGNOS components (shabda, dhvani, vansh) pull the concatenated bundle
`dist/shabdakosh.cyr` and its `dist/shabdakosh.deps` sidecar rather than
rebuilding from `src/`. Point `cyrius deps` at it as a path/git dependency; the
sidecar leaves the required stdlib folds in scope. The bundle's module order is
the `[lib].modules` list in [`cyrius.cyml`](cyrius.cyml).

See [`docs/guides/consuming-the-distlib.md`](docs/guides/consuming-the-distlib.md)
for a complete, runnable consumer example (declare the dep → `cyrius deps` →
include the svara/varna chain + bundle → call `shabda_dict_english()`).

## Module Overview

Include order from `src/main.cyr` (modules never include each other — the entry
orders them; stdlib + svara/varna auto-resolve from `cyrius.cyml`):

```text
src/
├── error.cyr                    sakshi-backed error surface (shabda_err_*, shabda_is_err)
├── arpabet.cyr                  ARPABET <-> svara SVARA_PH_* phonemes (with stress)
├── ipa.cyr                      IPA <-> phoneme; parse_ipa_word / phonemes_to_ipa
├── notation.cyr                 unified ARPABET / IPA / X-SAMPA render + parse
└── dictionary/
    ├── entry.cyr                DictEntry, Pronunciation, Region
    ├── morphology.cyr           Morpheme / Decomposition tags
    ├── syllable.cyr             syllabify() via Maximal Onset Principle
    ├── _cmudict_data.cyr        generated base-dictionary data (single module)
    ├── cmudict.cyr              loads the data into a hashmap
    ├── mod.cyr                  PronunciationDict core: new/english/lookup/insert_user/merge/diff/...
    ├── coverage.cyr             text-corpus coverage analysis
    ├── stream.cyr              zero-alloc streaming word->phoneme lookup
    ├── trie.cyr                 O(k) prefix search / autocomplete
    ├── heteronym.cyr            context-aware heteronym resolution
    ├── g2p.cyr                  FallbackDict + FstModel G2P chain (fnptr dispatch)
    ├── static_dict.cyr          lazily-cached static dictionary singleton
    ├── lazy.cyr                 LazyDict — mmap-backed binary loading
    ├── detect.cyr              script / language detection
    ├── validate.cyr            varna inventory + phonotactics validation
    └── format/
        ├── mod.cyr             CMUdict / IPA text codecs
        ├── json.cyr            JSON via the bayan DOM
        ├── pls.cyr             W3C PLS XML
        ├── ssml.cyr            SSML <phoneme> tags
        └── binary.cyr          compact hand-rolled binary format
src/wasm.cyr                     WasmDict binding surface
```

See [`docs/architecture/`](docs/architecture/) for non-obvious constraints and
[`docs/adr/`](docs/adr/) for the port decisions.

## Consumers

- [shabda](https://github.com/MacCracken/shabda) — G2P engine (dictionary lookup + rules fallback)
- [dhvani](https://github.com/MacCracken/dhvani) — audio engine
- [vansh](https://github.com/MacCracken/vansh) — voice AI shell

## License

GPL-3.0-only
