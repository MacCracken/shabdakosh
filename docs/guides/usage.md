# Usage Guide

shabdakosh is a **Cyrius** (`.cyr`) library — flat, C-style functions, every symbol
prefixed `shbdk_`/`SHBDK_`. It is **not** a cargo crate: there is no `Cargo.toml`,
no `use`, no traits or generics. You consume it by pulling the distlib bundle and
`include`-ing it, then calling `shbdk_*` functions directly.

## Calling Conventions (read this first)

The whole surface follows a handful of rules. Learn them once:

- **Handles are `i64`.** A dictionary, entry, report, stream, vec, or map is an opaque
  pointer stored in an `i64`. You pass it back to the next call.
- **`Option<T>` → sentinel.** A "no value" result is `0` (a null pointer). Every
  `#must_use` lookup returns `0` for "not found" — test it with a plain `if (x == 0)`.
  For phoneme scalars the sentinel is `SHBDK_PH_NONE` (not `0`, since `0` is a real
  phoneme ordinal).
- **`Result<T, E>` → pointer-or-0.** Fallible constructors/parsers return a payload
  pointer, or `0` on failure. `shbdk_parse_cmudict`, `shbdk_from_json`,
  `shbdk_from_binary`, `shbdk_parse_pls` all return `0` on malformed input.
- **Packed-i64 errors (sakshi).** Where a function returns a status code it is a packed
  `sakshi` error: `0 == ok`, non-zero == error. Test with `shbdk_is_err(err)` /
  `shbdk_is_ok(err)`; get diagnostic text with `shbdk_err_name(err)` (returns a
  `cstring`).
- **Out-params.** A few functions write results into caller-allocated slots
  (`out_code`, `out_name`) and return `1`/`0` for success/failure — e.g.
  `shbdk_detect_script_name`.
- **Phonemes are `SVARA_PH_*` integer ordinals** from svara. A pronunciation is a `vec`
  of them (`vec_new` / `vec_push` / `vec_get` / `vec_len`). They interoperate directly
  with svara's `PhonemeEvent`.

---

## Building and Consuming

```sh
cyrius deps                                   # resolve dependencies
cyrius build src/main.cyr build/shabdakosh    # compile
cyrius test tests                             # run every .tcyr suite
```

Consumers pull `dist/shabdakosh.cyr` and its `dist/shabdakosh.deps` sidecar, and
declare the svara + varna git deps plus the stdlib folds (see `cyrius.cyml [deps]`).
The svara chain must be included before the shabda modules — the include order in
`src/main.cyr` is the reference.

---

## Getting a Dictionary

Three constructors cover the common cases:

```
var d1 = shbdk_dict_new();                # empty, no language tag
var d2 = shbdk_dict_english();            # the full built-in dict (10,617 entries), tagged "en"
var d3 = shbdk_dict_english_minimal();    # ~29 common function words, tagged "en"
```

`shbdk_dict_english()` loads the generated CMUdict data
(`src/dictionary/_cmudict_data.cyr`) into a `lib/hashmap` on first construction.
`shbdk_dict_english_minimal()` is the memory-light dict for tests and constrained use.

Size and emptiness:

```
var n     = shbdk_dict_len(d2);        # base-entry count
var un    = shbdk_dict_user_len(d2);   # user-overlay entry count
var empty = shbdk_dict_is_empty(d1);   # 1 if base is empty, else 0
```

---

## Lookup

Two lookup entry points. Both take a `cstr` word, are case-insensitive (Unicode
lowercase with Rust's `to_lowercase` rules — `"ÜBER"` finds `"über"`, a word-final `Σ` is
`ς` — and a fast path when the word is already lowercase), and check the **user overlay
first, then the base**.

```
# Primary pronunciation as a vec of SVARA_PH_* ordinals, or 0 if not found.
var ph = shbdk_dict_lookup(d2, "hello");
if (ph == 0) {
    println("not in dictionary");
} else {
    println_int(vec_len(ph));            # 4 for "hello"
    var first = vec_get(ph, 0);          # SVARA_PH_FRIC_H
}
```

```
# Full entry (all pronunciations + freq/region), or 0.
var e = shbdk_dict_lookup_entry(d2, "read");
if (e != 0) {
    var count = shbdk_dict_entry_len(e);            # number of pronunciations
    var prim  = shbdk_dict_entry_primary_phonemes(e);   # primary pronunciation vec
    var all   = shbdk_dict_entry_all(e);            # vec of ShbdkPronunciation* handles
}
```

`shbdk_dict_lookup_all(d, word)` is a shortcut that returns the vec of pronunciation
handles directly (or `0`).

Working with a pronunciation handle:

```
var pron = vec_get(shbdk_dict_entry_all(e), 0);
var pph  = shbdk_pronunciation_phonemes(pron);       # the phoneme vec
if (shbdk_pronunciation_has_frequency(pron) == 1) {
    var f = shbdk_pronunciation_frequency(pron);     # f64 bits
}
if (shbdk_pronunciation_has_region(pron) == 1) {
    var reg = shbdk_pronunciation_region(pron);      # SHBDK_REGION_*
}
```

Region-specific pronunciation (`SHBDK_REGION_GA` / `SHBDK_REGION_RP`, or
`shbdk_region_from_code("GA")`):

```
var ga = shbdk_dict_entry_for_region(e, SHBDK_REGION_GA);   # a pronunciation handle, or 0
```

`SHBDK_REGION_NONE` is not a region, so `for_region` finds nothing for it.

### Frequencies

A frequency is a relative weight, **0.0 to 1.0** (f64 bits). `shbdk_frequency_is_valid(f)`
says whether a value is one; NaN, the infinities and anything outside the range are not.
`shbdk_pronunciation_with_frequency` stores only a valid value (otherwise the pronunciation
keeps the frequency it had), every reader (CMUdict `@freq`, JSON, binary) refuses an
invalid one, and the writers write the shortest text that reads back exactly
(`shbdk_frequency_text`: `0.5`, `0.85`, `0.3333333333333333`).

---

## The User Overlay

The overlay is a second map that shadows the base dict during lookup. Use it to add or
override words without touching the base.

```
var d = shbdk_dict_english();

# Add or override "agnos" in the overlay.
var seq = vec_new();
vec_push(seq, SVARA_PH_VOWEL_ASH);
vec_push(seq, SVARA_PH_PLOSIVE_G);
vec_push(seq, SVARA_PH_NASAL_N);
vec_push(seq, SVARA_PH_VOWEL_O);
vec_push(seq, SVARA_PH_FRIC_S);
shbdk_dict_insert_user(d, "agnos", seq);
# shbdk_dict_lookup(d, "agnos") now returns the overlay pronunciation.

# Override an existing base word — overlay wins ("thee").
var the = vec_new();
vec_push(the, SVARA_PH_FRIC_DH);
vec_push(the, SVARA_PH_VOWEL_LONG_I);
shbdk_dict_insert_user(d, "the", the);

# Remove the override; returns 1 if it was present, 0 if not. Base pronunciation returns.
shbdk_dict_remove_user(d, "the");
```

Base-dict inserts (not the overlay) go through `shbdk_dict_insert(d, word, phonemes)`.
`insert` and `insert_user` **copy** the phoneme vec, so a caller may reuse its buffer;
`shbdk_pronunciation_new(vec)` takes the vec over, as the Rust's `Pronunciation::new(Vec)`
did. `shbdk_dict_set_language` copies the code. `shbdk_dict_clone(d)` is a deep copy.
Entry-level variants: `shbdk_dict_insert_entry` / `shbdk_dict_insert_user_entry` take a
prebuilt DictEntry handle instead of a phoneme vec.

Override semantics recap: **overlay beats base** on lookup; `remove_user` only touches the
overlay, so removing an override restores whatever the base had.

---

## Language Tag

A dict carries an optional ISO-639 code (a `cstr`, or `0` for untagged):

```
var code = shbdk_dict_language(d2);          # "en" for english(); 0 for dict_new()
shbdk_dict_set_language(d1, "fr");           # mutate in place
var d = shbdk_dict_with_language(shbdk_dict_new(), "de");   # builder form, returns d
```

---

## Notation Bridges: ARPABET / IPA / X-SAMPA

### ARPABET (CMU)

```
var p  = shbdk_arpabet_to_phoneme("SH");                 # SVARA_PH_FRIC_SH, or SHBDK_PH_NONE
var s  = shbdk_phoneme_to_arpabet(SVARA_PH_FRIC_SH);     # "SH" (cstr), or 0 if unmapped

# Stress-aware: AH0 -> schwa, AH1/AH2 -> cup-v.
var sch = shbdk_arpabet_to_phoneme_with_stress("AH0");   # SVARA_PH_VOWEL_SCHWA
```

`shbdk_arpabet_to_phoneme` returns `SHBDK_PH_NONE` for an unknown symbol (not `0`).

### IPA

```
var one = shbdk_ipa_to_phoneme("ʃ");            # single symbol -> SVARA_PH_FRIC_SH, or SHBDK_PH_NONE
var sym = shbdk_phoneme_to_ipa(SVARA_PH_FRIC_SH);   # "ʃ" (cstr)

# Full word: greedy longest-byte-match parser (handles tie-bar affricates, strips stress marks).
var seq = shbdk_parse_ipa_word("hɛˈloʊ");       # vec of 4 phonemes
var str = shbdk_phonemes_to_ipa(seq);           # "hɛloʊ" (cstr)
```

### Unified notation dispatch (X-SAMPA and the others)

`SHBDK_NOTATION_ARPABET` / `SHBDK_NOTATION_IPA` / `SHBDK_NOTATION_XSAMPA` select a
table; one pair of functions parses/renders a whole transcription:

```
var cat = vec_new();
vec_push(cat, SVARA_PH_PLOSIVE_K);
vec_push(cat, SVARA_PH_VOWEL_ASH);
vec_push(cat, SVARA_PH_PLOSIVE_T);
var xs = shbdk_notation_render(SHBDK_NOTATION_XSAMPA, cat);   # "k { t" (cstr)
var back = shbdk_notation_parse(SHBDK_NOTATION_XSAMPA, xs);   # vec of phonemes

# Single-symbol helpers on a chosen notation:
var ph  = shbdk_notation_str_to_phoneme(SHBDK_NOTATION_XSAMPA, "{");   # SVARA_PH_VOWEL_ASH
var str = shbdk_notation_phoneme_to_str(SHBDK_NOTATION_IPA, SVARA_PH_VOWEL_ASH);   # "æ"
var nm  = shbdk_notation_name(SHBDK_NOTATION_XSAMPA);   # "X-SAMPA"
```

---

## I/O Formats

Every parser returns a **dict handle or `0`** on malformed input. Every emitter returns
a `cstr`.

### CMUdict text

```
var d = shbdk_parse_cmudict("cat  K AE1 T\ndog  D AO1 G\n");   # dict, or 0
if (d == 0) { println("parse failed"); }

var out      = shbdk_to_cmudict(d);              # base entries, sorted (cstr)
var out_user = shbdk_to_cmudict_with_user(d);    # includes the user overlay
```

The extended format carries `;;; @freq=0.7` and `;;; @region=GA` annotation lines
before an entry; variant pronunciations use `read(2)  ...` (only a trailing `(digits)` is a
variant marker, so CMUdict's own `(PAREN` and `;SEMI-COLON` are words). Any of these makes
`shbdk_parse_cmudict` return `0`, with the reason in `shbdk_last_error()` (see *Errors and
Diagnostics*): a `@freq` that is not a number or not 0.0–1.0, an unknown `@region`, a line
with no two-space separator, an unknown ARPABET symbol, a word with no phonemes, or input
that is not UTF-8. Whitespace is Unicode White_Space.

The writers return `0` instead of a file that would not load back: a word with a newline,
a two-space run or a trailing `(n)`, or a pronunciation with no ARPABET symbol at all
(ARPABET is an English phoneme set; a phoneme without a symbol is left out).

File helpers: `shbdk_load_cmudict_file(path)` (dict-or-0) and
`shbdk_save_cmudict_file(dict, path)` (0 on success, negative on error). The loaders read
the whole file, whatever its size.

### IPA lexicon text

```
var d   = shbdk_parse_ipa("hello /hɛloʊ/\nworld /wɜld/\n");   # dict, or 0
var out = shbdk_to_ipa(d);                                    # cstr, or 0
```

The IPA bridge covers every svara phoneme with a symbol (all 100, silence aside).
Affricates are written with their tie bar (`t͡ʃ`, `d͡ʒ`, `t͡s`); a bare `tʃ` / `dʒ` still reads
as the affricate, a bare `ts` is a cluster. Where two symbols would read back as one
(`[a, ɪ]` as `aɪ`), the writer puts a syllable break between them (`a.ɪ`). `r` is the
alveolar trill and `ɹ` the English approximant, as IPA defines them.

### W3C PLS (Pronunciation Lexicon Specification)

```
var d = shbdk_parse_pls("<lexicon alphabet=\"ipa\"><lexeme><grapheme>hi</grapheme><phoneme>haɪ</phoneme></lexeme></lexicon>");
var out = shbdk_to_pls(d, "en-US");                # cstr
var out_user = shbdk_to_pls_with_user(d, "en-US");
```

The reader is a real XML reader: attributes in either quote style, comments, processing
instructions, CDATA and character references, and an end tag must close its start tag.
Every `<grapheme>` of a `<lexeme>` gets its pronunciations, `prefer="true"` ones first.
Only the `ipa` alphabet is read; another one (on the lexicon or a phoneme) is an error.

### SSML `<phoneme>` tags

```
var cat = vec_new();
vec_push(cat, SVARA_PH_PLOSIVE_K);
vec_push(cat, SVARA_PH_VOWEL_ASH);
vec_push(cat, SVARA_PH_PLOSIVE_T);
var tag = shbdk_to_ssml_phoneme("cat", cat);   # <phoneme alphabet="ipa" ph="kæt">cat</phoneme>

var r = shbdk_parse_ssml_phoneme(tag);          # result handle, or 0
if (r != 0) {
    var word = shbdk_ssml_result_word(r);       # "cat" (cstr)
    var ph   = shbdk_ssml_result_phonemes(r);   # phoneme vec
}
```

### JSON

Hand-written codec over the whole dict (round-trips words, phonemes, frequency, region,
language, and the user overlay):

```
var json = shbdk_to_json(d);                     # cstr
var back = shbdk_from_json(json);                # dict, or 0 (shbdk_last_error() says why)
```

The reader checks the schema: `entries` is required, a phoneme is an integer 0–100, a
frequency a number 0.0–1.0, a region `"GA"` / `"RP"`, an entry has at least one
pronunciation. It also reads the v0.1 shape where an entry is its phoneme array
(`{"entries":{"cat":[24,7,22]}}`).

Every other record type has a codec and an equality function too, in
`src/dictionary/format/records.cyr`: `shbdk_<type>_to_json` / `_from_json` / `_eq` for
`pronunciation`, `dict_entry`, `dict_diff`, `morpheme`, `decomposition`, `syllable`,
`coverage_report`, `trie` (its sorted words), `heteronym_context`, `g2p_result`,
`fst_model`; plus `shbdk_region_to_json` / `_from_json`, `shbdk_lookup_source_to_json` /
`_from_json` and `shbdk_err_to_json` / `_from_json`.

### Binary

Compact `SHBD`-magic format; the deserializer is bounds-checked against hostile input:

```
var bytes = shbdk_to_binary(d);                  # a Str; empty if the dict does not fit the format
var back  = shbdk_from_binary(bytes);            # dict, or 0 (shbdk_last_error() says why)

shbdk_save_binary_file(d, "/path/dict.bin");     # 0, or negative on error
var d2 = shbdk_load_binary_file("/path/dict.bin");   # dict, or 0
```

Every field is checked, not only every length: an option flag is 0 or 1, a phoneme 0–100,
a region GA or RP, a frequency 0.0–1.0, a word well-formed UTF-8, an entry has a
pronunciation, and nothing follows the overlay. The writer refuses what the format cannot
hold (a field over 65,535, a phoneme outside 0–100) rather than write an unreadable file.

---

## Coverage

Report how much of a text the dict can pronounce:

```
var r = shbdk_dict_coverage(d2, "the hello world xyz");
var total    = shbdk_coverage_total_tokens(r);       # 4
var covered  = shbdk_coverage_covered_tokens(r);     # 3
var missing  = shbdk_coverage_uncovered_words(r);    # vec of Str (deduped, sorted)
var nmiss    = shbdk_coverage_uncovered_count(r);
var full     = shbdk_coverage_is_fully_covered(r);   # 1 if nothing uncovered
var pct      = shbdk_coverage_pct(r);                # 0.0-100.0 as f64 bits
```

Tokens split on Unicode whitespace (NBSP, U+3000, …). Each token loses leading and trailing
punctuation and symbols (ASCII and Unicode: curly quotes, the ellipsis, guillemets, emoji),
reads a typographic apostrophe (`’`, `ʼ`) as `'`, and lowercases with Unicode rules.

---

## Prefix Search (Trie)

```
var hits = shbdk_dict_prefix_search(d3, "he");   # vec of matching words (Str): he, hello
var n    = vec_len(hits);
```

Build a standalone trie if you need repeated queries:

```
var t = shbdk_trie_from_dict(d3);
var has = shbdk_trie_contains(t, "hello");        # 1/0
var matches = shbdk_trie_search_prefix(t, "he");  # vec of Str (str_cstr() to print one)
var size = shbdk_trie_len(t);
```

---

## Streaming Lookup

An allocation-free cursor over a vec of words:

```
var words = vec_new();
vec_push(words, "hello");
vec_push(words, "xyz");
var s = shbdk_dict_lookup_stream(d3, words);
while (shbdk_lookup_stream_next(s) == 1) {
    var w  = shbdk_stream_word(s);       # current word (cstr)
    var ph = shbdk_stream_phonemes(s);   # phoneme vec, or 0 if the word missed
}
var hint = shbdk_lookup_stream_size_hint(s);   # remaining count
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
var ctx = shbdk_heteronym_context_new(ctx_words, 1);   # target at index 1
var preceding = shbdk_heteronym_preceding_words(ctx);
var following = shbdk_heteronym_following_words(ctx);

# resolver: fn(word, context) -> the index of the pronunciation to use, or -1 for none.
var ph = shbdk_dict_lookup_with_context(d, "read", &my_resolver, ctx);
```

The context keeps the words vec it was given; `shbdk_heteronym_context_set_position(ctx, i)`
moves the target.

---

## Syllables and Morphemes

```
var syl = shbdk_syllabify(ph);                  # vec of syllable handles
var first = vec_get(syl, 0);
var sp = shbdk_syllable_phonemes(first);        # its phoneme vec
var st = shbdk_syllable_stress(first);          # SHBDK_STRESS_* (Unstressed unless set)
```

`shbdk_syllabify` follows the Maximal Onset Principle: between two nuclei, the next
syllable takes the longest run of consonants that can begin a syllable — English's onsets
when every consonant is English (`astray` → `ə.stɹeɪ`, `atlas` → `æt.ləs`), the Sonority
Sequencing Principle otherwise — and `ŋ` never begins one. Every svara vowel and diphthong
is a nucleus, the extended ones included (`über` has two syllables).

A `Decomposition` is a vec of morphemes (`shbdk_morpheme_new(text, SHBDK_MORPH_*, phonemes)`,
which copies the text); `shbdk_decomposition_composite_phonemes` / `_text` / `_root` /
`_prefixes` / `_suffixes` read it.

---

## G2P Fallback

Wrap a dict with a grapheme→phoneme model so out-of-dictionary words still resolve.
The model is a **predict fn-pointer + a state handle**: the fn-pointer has signature
`fn(word, state) -> G2PResult-or-0`, and `state` is threaded through to it.

```
# FstModel is the built-in state stub; wire a real WFST engine into shbdk_fst_model_predict.
var model = shbdk_fst_model_new("/path/model", SHBDK_FST_NOTATION_ARPABET);
var fb = shbdk_dict_with_fallback(shbdk_dict_english_minimal(), &shbdk_fst_model_predict, model);

# Lookup order: user overlay -> base dict -> G2P model. Returns a phoneme vec, or 0.
var ph = shbdk_fallback_dict_lookup(fb, "hello");

# Want to know where the answer came from?
var lr = shbdk_fallback_dict_lookup_with_source(fb, "xyzzy");
var src = shbdk_lookup_result_source(lr);   # SHBDK_SOURCE_USER_OVERLAY / _BASE_DICTIONARY / _G2P_MODEL
var phs = shbdk_lookup_result_phonemes(lr);
var conf = shbdk_lookup_result_confidence(lr);

# Promote a confident G2P prediction into the user overlay so it caches.
shbdk_fallback_dict_promote_if_confident(fb, "xyzzy", 0.75);
```

A predict fn receives the lowercased word as a `cstr`, builds its result with
`shbdk_g2p_result_new(phonemes_vec, confidence)` and returns `0` for "no prediction".
`shbdk_g2p_result_phonemes` / `_confidence` read it back. Lookups and promotions copy the
predicted phonemes, so a model may reuse one buffer.

---

## varna: Validation and Detection

These use the `varna` dependency (bare `phoneme_*` / `script_*` symbols that link
alongside svara). shabda bridges svara `SVARA_PH_*` ordinals to varna IPA internally.

### Inventory validation

Check that a dict's phonemes all belong to a language's inventory:

```
var report = shbdk_validate_inventory(d, phoneme_spanish());   # or phoneme_english/hindi/german/sanskrit
var ok    = shbdk_validation_report_is_valid(report);          # 1 if every entry is in-inventory
var nbad  = shbdk_validation_report_invalid_count(report);
var bad   = shbdk_validation_report_invalid_entries(report);   # vec of InvalidEntry handles
var first = vec_get(bad, 0);
var word  = shbdk_invalid_entry_word(first);                   # offending word (cstr)
var phs   = shbdk_invalid_entry_invalid_phonemes(first);       # the offending phonemes
```

Membership treats `ɔ`/`ɔː` (length) and `t͡ʃ`/`tʃ` (the affricate tie) as one phoneme.

Convenience wrappers validate against the dict's own language tag:
`shbdk_dict_validate(d)` and `shbdk_dict_validate_phonotactics(d)` — for every language
varna has an inventory / a phonotactic profile for, and a regional tag (`pt-BR`) falls back
to its language. Phonotactics has its own path: `shbdk_validate_phonotactics(dict,
phonotactics)` → a report read with `shbdk_phonotactic_report_is_valid` / `_violations` /
`_violation_count`. Each pronunciation is syllabified (Maximal Onset) and its onsets,
nuclei, codas and syllable boundaries are checked against the profile's constraints for
that position, with onsets held to the profile's maximum onset and codas to its maximum
coda. The built-in English dictionary has no English phonotactic violation.

### Script and language detection

```
var script = shbdk_detect_script("hello");   # "Latn" (cstr), or 0 if undetermined
var hints  = shbdk_detect_language_hint("नमस्ते");   # vec of language codes (cstr), sorted

# script name via out-params; returns 1 on success, 0 otherwise.
var out_code = alloc(8);
var out_name = alloc(8);
if (shbdk_detect_script_name("γεια", out_code, out_name) == 1) {
    # load64(out_code) -> "Grek", load64(out_name) -> "Greek" (both static cstr)
}
```

---

## Dictionary Operations

### Merge

```
shbdk_dict_merge(base, other);                 # other wins on conflict
shbdk_dict_merge_conservative(base, other);    # base kept on conflict
```

Both merge base entries and overlay entries.

### Diff

```
var dd = shbdk_dict_diff(v1, v2);              # by effective lookup, sorted
var added   = shbdk_dict_diff_added(dd);       # vec of Str: in v2 only
var removed = shbdk_dict_diff_removed(dd);     # in v1 only
var changed = shbdk_dict_diff_changed(dd);     # present in both, different pronunciation
                                               # (frequencies compared by value: 0.0 == -0.0)
var total   = shbdk_dict_diff_len(dd);
var empty   = shbdk_dict_diff_is_empty(dd);    # 1 if identical
```

---

## Static Dictionary

The built-in English dictionary behind a compile-time perfect hash, the port of the Rust
`phf` static. Nothing is loaded: the first lookup in a process costs ~35 µs rather than
the ~8 ms `shbdk_dict_english()` takes to build a dictionary, and a lookup is ~124 ns.
A miss, a repeated hit and a capitalised word (`"Hello"`) allocate nothing:

```
var e   = shbdk_static_lookup_entry("hello");   # entry handle, or 0
var ph  = shbdk_static_lookup("Hello");         # phoneme vec, or 0 (case-insensitive)
var n   = shbdk_static_len();                    # 10617, a generated constant
```

The entry a hit returns is the static dictionary's own and the same pointer every time,
so treat it as read-only. For a full `PronunciationDict` handle over the same data (prefix
search, export, merge), `shbdk_static_dict()` builds one on first use and caches it; its
entries are separate objects with the same content. Both caches live in the allocator like
any other allocation: after `alloc_reset()`, call `shbdk_static_reset()` before the next
static lookup.

---

## Lazy (mmap) Dictionary

Open a binary dict via mmap (real `mmap` on Linux, `file_read_all` fallback on AGNOS).
The handle **is** a dict handle — decode is eager, so any `shbdk_dict_*` /
`shbdk_lazy_*` accessor works:

```
shbdk_save_binary_file(shbdk_dict_english_minimal(), "/tmp/dict.bin");
var lz = shbdk_lazy_open("/tmp/dict.bin");       # dict handle, or 0
var n  = shbdk_lazy_len(lz);
var ph = shbdk_lazy_lookup(lz, "hello");
```

---

## WasmDict (JSON boundary surface)

A thin handle whose methods cross the boundary as JSON strings (for browser delivery via
the toolchain). Pronunciations come back as IPA arrays.

```
var w = shbdk_wasm_dict_english_minimal();       # or _new / _english
shbdk_wasm_dict_insert_user_ipa(w, "foo", "kæt");
var json = shbdk_wasm_dict_lookup(w, "foo");     # JSON IPA array (cstr)
var pref = shbdk_wasm_dict_prefix_search(w, "he");   # JSON array
var cov  = shbdk_wasm_dict_coverage(w, "the cat");   # JSON {total_tokens, covered_tokens, uncovered_words}
var n    = shbdk_wasm_dict_len(w);
```

---

## Errors and Diagnostics

Where a function returns a `sakshi` packed error rather than a payload:

```
var err = shbdk_save_binary_file(d, "/bad/path");
if (shbdk_is_err(err) == 1) {
    println(shbdk_err_name(err));       # e.g. "dictionary parse error"
}
```

Named error constructors exist for the four categories — `shbdk_err_dict_parse()`,
`shbdk_err_unknown_symbol()`, `shbdk_err_phoneme_not_in_inventory()`,
`shbdk_err_unknown_language()` — mapping to `SHBDK_ERR_*` codes.

A parser or loader that fails returns `0` and keeps the reason, the detail the Rust carried
in `DictParseError(String)`:

```
var d = shbdk_parse_cmudict("word  XX1\n");
if (d == 0) {
    println(shbdk_last_error());        # line 1: unknown ARPABET symbol 'XX1'
}
```

Each parse / load entry point clears it on entry, so it describes the last failure (and is
`""` after a success). `shbdk_err_to_json(err, detail)` writes `{"DictParseError":"..."}`.
