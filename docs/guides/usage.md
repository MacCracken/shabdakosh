# Usage Guide

shabdakosh is a **Cyrius** (`.cyr`) library — flat, C-style functions, every symbol
prefixed `shabda_`/`SHABDA_`. It is **not** a cargo crate: there is no `Cargo.toml`,
no `use`, no traits or generics. You consume it by pulling the distlib bundle and
`include`-ing it, then calling `shabda_*` functions directly.

## Calling Conventions (read this first)

The whole surface follows a handful of rules. Learn them once:

- **Handles are `i64`.** A dictionary, entry, report, stream, vec, or map is an opaque
  pointer stored in an `i64`. You pass it back to the next call.
- **`Option<T>` → sentinel.** A "no value" result is `0` (a null pointer). Every
  `#must_use` lookup returns `0` for "not found" — test it with a plain `if (x == 0)`.
  For phoneme scalars the sentinel is `SHABDA_PH_NONE` (not `0`, since `0` is a real
  phoneme ordinal).
- **`Result<T, E>` → pointer-or-0.** Fallible constructors/parsers return a payload
  pointer, or `0` on failure. `shabda_parse_cmudict`, `shabda_from_json`,
  `shabda_from_binary`, `shabda_parse_pls` all return `0` on malformed input.
- **Packed-i64 errors (sakshi).** Where a function returns a status code it is a packed
  `sakshi` error: `0 == ok`, non-zero == error. Test with `shabda_is_err(err)` /
  `shabda_is_ok(err)`; get diagnostic text with `shabda_err_name(err)` (returns a
  `cstring`).
- **Out-params.** A few functions write results into caller-allocated slots
  (`out_code`, `out_name`) and return `1`/`0` for success/failure — e.g.
  `shabda_detect_script_name`.
- **Phonemes are `SVARA_PH_*` integer ordinals** from svara. A pronunciation is a `vec`
  of them (`vec_new` / `vec_push` / `vec_get` / `vec_len`). They interoperate directly
  with svara's `PhonemeEvent`.

---

## Building and Consuming

```sh
cyrius deps                                   # resolve dependencies
cyrius build src/main.cyr build/shabdakosh    # compile
cyrius tests tests                            # run every .tcyr suite
```

Consumers pull `dist/shabdakosh.cyr` and its `dist/shabdakosh.deps` sidecar, and
declare the svara + varna git deps plus the stdlib folds (see `cyrius.cyml [deps]`).
The svara chain must be included before the shabda modules — the include order in
`src/main.cyr` is the reference.

---

## Getting a Dictionary

Three constructors cover the common cases:

```
var d1 = shabda_dict_new();                # empty, no language tag
var d2 = shabda_dict_english();            # the full built-in dict (10,617 entries), tagged "en"
var d3 = shabda_dict_english_minimal();    # ~29 common function words, tagged "en"
```

`shabda_dict_english()` loads the generated CMUdict data
(`src/dictionary/_cmudict_data.cyr`) into a `lib/hashmap` on first construction.
`shabda_dict_english_minimal()` is the memory-light dict for tests and constrained use.

Size and emptiness:

```
var n     = shabda_dict_len(d2);        # base-entry count
var un    = shabda_dict_user_len(d2);   # user-overlay entry count
var empty = shabda_dict_is_empty(d1);   # 1 if base is empty, else 0
```

---

## Lookup

Two lookup entry points. Both take a `cstr` word, are case-insensitive (ASCII
lowercase fold, with a fast-path when the word is already lowercase), and check the
**user overlay first, then the base**.

```
# Primary pronunciation as a vec of SVARA_PH_* ordinals, or 0 if not found.
var ph = shabda_dict_lookup(d2, "hello");
if (ph == 0) {
    println("not in dictionary");
} else {
    println_int(vec_len(ph));            # 4 for "hello"
    var first = vec_get(ph, 0);          # SVARA_PH_FRIC_H
}
```

```
# Full entry (all pronunciations + freq/region), or 0.
var e = shabda_dict_lookup_entry(d2, "read");
if (e != 0) {
    var count = shabda_dict_entry_len(e);            # number of pronunciations
    var prim  = shabda_dict_entry_primary_phonemes(e);   # primary pronunciation vec
    var all   = shabda_dict_entry_all(e);            # vec of ShPronunciation* handles
}
```

`shabda_dict_lookup_all(d, word)` is a shortcut that returns the vec of pronunciation
handles directly (or `0`).

Working with a pronunciation handle:

```
var pron = vec_get(shabda_dict_entry_all(e), 0);
var pph  = shabda_pronunciation_phonemes(pron);       # the phoneme vec
if (shabda_pronunciation_has_frequency(pron) == 1) {
    var f = shabda_pronunciation_frequency(pron);     # f64 bits
}
if (shabda_pronunciation_has_region(pron) == 1) {
    var reg = shabda_pronunciation_region(pron);      # SHABDA_REGION_*
}
```

Region-specific pronunciation (`SHABDA_REGION_GA` / `SHABDA_REGION_RP`, or
`shabda_region_from_code("GA")`):

```
var ga = shabda_dict_entry_for_region(e, SHABDA_REGION_GA);   # a pronunciation handle, or 0
```

---

## The User Overlay

The overlay is a second map that shadows the base dict during lookup. Use it to add or
override words without touching the base.

```
var d = shabda_dict_english();

# Add or override "agnos" in the overlay.
var seq = vec_new();
vec_push(seq, SVARA_PH_VOWEL_ASH);
vec_push(seq, SVARA_PH_PLOSIVE_G);
vec_push(seq, SVARA_PH_NASAL_N);
vec_push(seq, SVARA_PH_VOWEL_O);
vec_push(seq, SVARA_PH_FRIC_S);
shabda_dict_insert_user(d, "agnos", seq);
# shabda_dict_lookup(d, "agnos") now returns the overlay pronunciation.

# Override an existing base word — overlay wins.
var the = vec_new();
vec_push(the, SVARA_PH_FRIC_DH);
vec_push(the, SVARA_PH_VOWEL_E);
shabda_dict_insert_user(d, "the", the);

# Remove the override; returns 1 if it was present, 0 if not. Base pronunciation returns.
shabda_dict_remove_user(d, "the");
```

Base-dict inserts (not the overlay) go through `shabda_dict_insert(d, word, phonemes)`.
Entry-level variants: `shabda_dict_insert_entry` / `shabda_dict_insert_user_entry` take a
prebuilt DictEntry handle instead of a phoneme vec.

Override semantics recap: **overlay beats base** on lookup; `remove_user` only touches the
overlay, so removing an override restores whatever the base had.

---

## Language Tag

A dict carries an optional ISO-639 code (a `cstr`, or `0` for untagged):

```
var code = shabda_dict_language(d2);          # "en" for english(); 0 for dict_new()
shabda_dict_set_language(d1, "fr");           # mutate in place
var d = shabda_dict_with_language(shabda_dict_new(), "de");   # builder form, returns d
```

---

## Notation Bridges: ARPABET / IPA / X-SAMPA

### ARPABET (CMU)

```
var p  = shabda_arpabet_to_phoneme("SH");                 # SVARA_PH_FRIC_SH, or SHABDA_PH_NONE
var s  = shabda_phoneme_to_arpabet(SVARA_PH_FRIC_SH);     # "SH" (cstr), or 0 if unmapped

# Stress-aware: AH0 -> schwa, AH1/AH2 -> cup-v.
var sch = shabda_arpabet_to_phoneme_with_stress("AH0");   # SVARA_PH_VOWEL_SCHWA
```

`shabda_arpabet_to_phoneme` returns `SHABDA_PH_NONE` for an unknown symbol (not `0`).

### IPA

```
var one = shabda_ipa_to_phoneme("ʃ");            # single symbol -> SVARA_PH_FRIC_SH, or SHABDA_PH_NONE
var sym = shabda_phoneme_to_ipa(SVARA_PH_FRIC_SH);   # "ʃ" (cstr)

# Full word: greedy longest-byte-match parser (handles tie-bar affricates, strips stress marks).
var seq = shabda_parse_ipa_word("hɛˈloʊ");       # vec of 4 phonemes
var str = shabda_phonemes_to_ipa(seq);           # "hɛloʊ" (cstr)
```

### Unified notation dispatch (X-SAMPA and the others)

`SHABDA_NOTATION_ARPABET` / `SHABDA_NOTATION_IPA` / `SHABDA_NOTATION_XSAMPA` select a
table; one pair of functions parses/renders a whole transcription:

```
var cat = vec_new();
vec_push(cat, SVARA_PH_PLOSIVE_K);
vec_push(cat, SVARA_PH_VOWEL_ASH);
vec_push(cat, SVARA_PH_PLOSIVE_T);
var xs = shabda_notation_render(SHABDA_NOTATION_XSAMPA, cat);   # "k { t" (cstr)
var back = shabda_notation_parse(SHABDA_NOTATION_XSAMPA, xs);   # vec of phonemes

# Single-symbol helpers on a chosen notation:
var ph  = shabda_notation_str_to_phoneme(SHABDA_NOTATION_XSAMPA, "{");   # SVARA_PH_VOWEL_ASH
var str = shabda_notation_phoneme_to_str(SHABDA_NOTATION_IPA, SVARA_PH_VOWEL_ASH);   # "æ"
var nm  = shabda_notation_name(SHABDA_NOTATION_XSAMPA);   # "X-SAMPA"
```

---

## I/O Formats

Every parser returns a **dict handle or `0`** on malformed input. Every emitter returns
a `cstr`.

### CMUdict text

```
var d = shabda_parse_cmudict("cat  K AE1 T\ndog  D AO1 G\n");   # dict, or 0
if (d == 0) { println("parse failed"); }

var out      = shabda_to_cmudict(d);              # base entries, sorted (cstr)
var out_user = shabda_to_cmudict_with_user(d);    # includes the user overlay
```

The extended format carries `;;; @freq=0.7` and `;;; @region=GA` annotation lines
before an entry; variant pronunciations use `read(2)  ...`. A malformed `@freq` token or
an unknown `@region` makes `shabda_parse_cmudict` return `0` (matching the Rust `Err`).

File helpers: `shabda_load_cmudict_file(path)` (dict-or-0) and
`shabda_save_cmudict_file(dict, path)` (packed status; test with `shabda_is_err`).

### IPA lexicon text

```
var d   = shabda_parse_ipa("hello /hɛloʊ/\nworld /wɜld/\n");   # dict, or 0
var out = shabda_to_ipa(d);                                    # cstr
```

### W3C PLS (Pronunciation Lexicon Specification)

```
var d = shabda_parse_pls("<lexicon alphabet=\"ipa\"><lexeme><grapheme>hi</grapheme><phoneme>haɪ</phoneme></lexeme></lexicon>");
var out = shabda_to_pls(d, "en-US");                # cstr
var out_user = shabda_to_pls_with_user(d, "en-US");
```

### SSML `<phoneme>` tags

```
var cat = vec_new();
vec_push(cat, SVARA_PH_PLOSIVE_K);
vec_push(cat, SVARA_PH_VOWEL_ASH);
vec_push(cat, SVARA_PH_PLOSIVE_T);
var tag = shabda_to_ssml_phoneme("cat", cat);   # <phoneme alphabet="ipa" ph="kæt">cat</phoneme>

var r = shabda_parse_ssml_phoneme(tag);          # result handle, or 0
if (r != 0) {
    var word = shabda_ssml_result_word(r);       # "cat" (cstr)
    var ph   = shabda_ssml_result_phonemes(r);   # phoneme vec
}
```

### JSON

Hand-written codec over the whole dict (round-trips words, phonemes, frequency, region,
language, and the user overlay):

```
var json = shabda_to_json(d);                     # cstr
var back = shabda_from_json(json);                # dict, or 0 on invalid JSON
```

### Binary

Compact `SHBD`-magic format; the deserializer is bounds-checked against hostile input:

```
var bytes = shabda_to_binary(d);                  # buffer handle
var back  = shabda_from_binary(bytes);            # dict, or 0 if magic/length invalid

shabda_save_binary_file(d, "/path/dict.bin");     # packed status
var d2 = shabda_load_binary_file("/path/dict.bin");   # dict, or 0
```

---

## Coverage

Report how much of a text the dict can pronounce:

```
var r = shabda_dict_coverage(d2, "the hello world xyz");
var total    = shabda_coverage_total_tokens(r);       # 4
var covered  = shabda_coverage_covered_tokens(r);     # 3
var missing  = shabda_coverage_uncovered_words(r);    # vec of cstr (deduped, sorted)
var nmiss    = shabda_coverage_uncovered_count(r);
var full     = shabda_coverage_is_fully_covered(r);   # 1 if nothing uncovered
var pct      = shabda_coverage_pct(r);                # 0.0-100.0 as f64 bits
```

---

## Prefix Search (Trie)

```
var hits = shabda_dict_prefix_search(d3, "he");   # vec of matching words (cstr): he, hello
var n    = vec_len(hits);
```

Build a standalone trie if you need repeated queries:

```
var t = shabda_trie_from_dict(d3);
var has = shabda_trie_contains(t, "hello");        # 1/0
var matches = shabda_trie_search_prefix(t, "he");  # vec of cstr
var size = shabda_trie_len(t);
```

---

## Streaming Lookup

An allocation-free cursor over a vec of words:

```
var words = vec_new();
vec_push(words, "hello");
vec_push(words, "xyz");
var s = shabda_dict_lookup_stream(d3, words);
while (shabda_lookup_stream_next(s) == 1) {
    var w  = shabda_stream_word(s);       # current word (cstr)
    var ph = shabda_stream_phonemes(s);   # phoneme vec, or 0 if the word missed
}
var hint = shabda_lookup_stream_size_hint(s);   # remaining count
```

---

## Heteronyms (context-sensitive lookup)

Provide surrounding words and a resolver fn-pointer to disambiguate words like "read"
and "bass":

```
var ctx_words = vec_new();
vec_push(ctx_words, "I");
vec_push(ctx_words, "read");
vec_push(ctx_words, "books");
var ctx = shabda_heteronym_context_new(ctx_words, 1);   # target at index 1
var preceding = shabda_heteronym_preceding_words(ctx);
var following = shabda_heteronym_following_words(ctx);

# resolver: fn(context, entry) -> chosen-pronunciation-index.
var ph = shabda_dict_lookup_with_context(d, "read", &my_resolver, ctx);
```

---

## G2P Fallback

Wrap a dict with a grapheme→phoneme model so out-of-dictionary words still resolve.
The model is a **predict fn-pointer + a state handle**: the fn-pointer has signature
`fn(word, state) -> G2PResult-or-0`, and `state` is threaded through to it.

```
# FstModel is the built-in state stub; wire a real WFST engine into shabda_fst_model_predict.
var model = shabda_fst_model_new("/path/model", SHABDA_FST_NOTATION_ARPABET);
var fb = shabda_dict_with_fallback(shabda_dict_english_minimal(), &shabda_fst_model_predict, model);

# Lookup order: user overlay -> base dict -> G2P model. Returns a phoneme vec, or 0.
var ph = shabda_fallback_dict_lookup(fb, "hello");

# Want to know where the answer came from?
var lr = shabda_fallback_dict_lookup_with_source(fb, "xyzzy");
var src = shabda_lookup_result_source(lr);   # SHABDA_SOURCE_USER_OVERLAY / _BASE_DICTIONARY / _G2P_MODEL
var phs = shabda_lookup_result_phonemes(lr);
var conf = shabda_lookup_result_confidence(lr);

# Promote a confident G2P prediction into the user overlay so it caches.
shabda_fallback_dict_promote_if_confident(fb, "xyzzy", 0.75);
```

A predict fn builds its result with `shabda_g2p_result_new(phonemes_vec, confidence)` and
returns `0` for "no prediction". `shabda_g2p_result_phonemes` / `_confidence` read it back.

---

## varna: Validation and Detection

These use the `varna` dependency (bare `phoneme_*` / `script_*` symbols that link
alongside svara). shabda bridges svara `SVARA_PH_*` ordinals to varna IPA internally.

### Inventory validation

Check that a dict's phonemes all belong to a language's inventory:

```
var report = shabda_validate_inventory(d, phoneme_spanish());   # or phoneme_english/hindi/german/sanskrit
var ok    = shabda_validation_report_is_valid(report);          # 1 if every entry is in-inventory
var nbad  = shabda_validation_report_invalid_count(report);
var bad   = shabda_validation_report_invalid_entries(report);   # vec of InvalidEntry handles
var first = vec_get(bad, 0);
var word  = shabda_invalid_entry_word(first);                   # offending word (cstr)
var phs   = shabda_invalid_entry_invalid_phonemes(first);       # the offending phonemes
```

Convenience wrappers validate against the dict's own language tag:
`shabda_dict_validate(d)` and `shabda_dict_validate_phonotactics(d)`.
Phonotactics has its own path: `shabda_validate_phonotactics(dict, phonotactics)` →
a report read with `shabda_phonotactic_report_is_valid` / `_violations` /
`_violation_count`.

### Script and language detection

```
var script = shabda_detect_script("hello");   # "Latn" (cstr), or 0 if undetermined
var hints  = shabda_detect_language_hint("नमस्ते");   # vec of language codes (cstr), sorted

# script name via out-params; returns 1 on success, 0 otherwise.
var out_code = alloc(8);
var out_name = alloc(8);
if (shabda_detect_script_name("γεια", out_code, out_name) == 1) {
    # load64(out_code) -> "Grek", load64(out_name) -> "Greek" (both static cstr)
}
```

---

## Dictionary Operations

### Merge

```
shabda_dict_merge(base, other);                 # other wins on conflict
shabda_dict_merge_conservative(base, other);    # base kept on conflict
```

Both merge base entries and overlay entries.

### Diff

```
var dd = shabda_dict_diff(v1, v2);              # by effective lookup, sorted
var added   = shabda_dict_diff_added(dd);       # vec of cstr: in v2 only
var removed = shabda_dict_diff_removed(dd);     # in v1 only
var changed = shabda_dict_diff_changed(dd);     # present in both, different pronunciation
var total   = shabda_dict_diff_len(dd);
var empty   = shabda_dict_diff_is_empty(dd);    # 1 if identical
```

---

## Static Cached Dictionary

A lazily-initialized singleton over `shabda_dict_english()` (the Cyrius stand-in for the
Rust `phf` static). First access pays a one-time load; subsequent calls are free:

```
var e   = shabda_static_lookup_entry("hello");   # entry handle, or 0
var ph  = shabda_static_lookup("hello");         # phoneme vec, or 0
var n   = shabda_static_len();                    # >10k
```

---

## Lazy (mmap) Dictionary

Open a binary dict via mmap (real `mmap` on Linux, `file_read_all` fallback on AGNOS).
The handle **is** a dict handle — decode is eager, so any `shabda_dict_*` /
`shabda_lazy_*` accessor works:

```
shabda_save_binary_file(shabda_dict_english_minimal(), "/tmp/dict.bin");
var lz = shabda_lazy_open("/tmp/dict.bin");       # dict handle, or 0
var n  = shabda_lazy_len(lz);
var ph = shabda_lazy_lookup(lz, "hello");
```

---

## WasmDict (JSON boundary surface)

A thin handle whose methods cross the boundary as JSON strings (for browser delivery via
the toolchain). Pronunciations come back as IPA arrays.

```
var w = shabda_wasm_dict_english_minimal();       # or _new / _english
shabda_wasm_dict_insert_user_ipa(w, "foo", "kæt");
var json = shabda_wasm_dict_lookup(w, "foo");     # JSON IPA array (cstr)
var pref = shabda_wasm_dict_prefix_search(w, "he");   # JSON array
var cov  = shabda_wasm_dict_coverage(w, "the cat");   # JSON {total_tokens, covered_tokens, uncovered_words}
var n    = shabda_wasm_dict_len(w);
```

---

## Errors and Diagnostics

Where a function returns a `sakshi` packed error rather than a payload:

```
var err = shabda_save_binary_file(d, "/bad/path");
if (shabda_is_err(err) == 1) {
    println(shabda_err_name(err));       # e.g. "dictionary parse error"
}
```

Named error constructors exist for the four categories — `shabda_err_dict_parse()`,
`shabda_err_unknown_symbol()`, `shabda_err_phoneme_not_in_inventory()`,
`shabda_err_unknown_language()` — mapping to `SHABDA_ERR_*` codes.
