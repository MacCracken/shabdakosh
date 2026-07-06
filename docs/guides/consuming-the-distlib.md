# Consuming the shabdakosh distlib

This guide is for **downstream Cyrius crates** — shabda (G2P engine), dhvani
(audio engine), vansh (voice AI shell), or anything in the AGNOS stack that needs
pronunciation lookup. It shows how to depend on shabdakosh, vendor it, and call
its pronunciation API from your own `.cyr` source.

shabdakosh ships as a **Cyrius distlib bundle** (`dist/shabdakosh.cyr`) — a single
concatenation of its modules in dependency order. You pull that bundle; you do
**not** build from `src/`. There is no Cargo crate, no `use`, no crates.io — the
distlib links flat, C-style, with every public symbol `shabda_`-prefixed.

## 1. Declare the dependency

In your consumer crate's `cyrius.cyml`, add shabdakosh as a path (or git) dep and
pull its bundle module:

```toml
[deps.shabdakosh]
path = "../shabdakosh"          # or: git = "https://github.com/MacCracken/shabdakosh", tag = "v3.0.0"
modules = ["dist/shabdakosh.cyr"]
```

shabdakosh does **not** vendor its own transitive AGNOS deps into the bundle — you
declare them too, because their symbols must be in scope when the shabdakosh
bundle links. shabdakosh is built on two sibling AGNOS crates:

- **svara** — provides the `SVARA_PH_*` phoneme identities. A pronunciation in
  shabdakosh is a sequence of svara phoneme ordinals, so svara must be present.
  svara itself has a small backend prefix-chain (`hisab` → `goonj` → `naad`),
  surfaced by the `dist/shabdakosh.deps` sidecar (see §4).
- **varna** — phoneme inventories, phonotactics, and script/registry metadata.
  Used by `shabda_validate_inventory` and `shabda_detect_script`.

```toml
[deps.svara]
path = "../svara"               # sibling AGNOS crate
modules = ["dist/svara.cyr"]

[deps.varna]
path = "../varna"               # sibling AGNOS crate
modules = ["dist/varna.cyr"]
```

You also need the **stdlib folds** shabdakosh's bundle expects in scope. At
minimum this is the set it leans on internally — the O(1) base-dict hashmap, the
JSON codec fold, the mmap fold behind the lazy on-disk dictionary, plus the usual
alloc/string/vec/fmt/io basics:

```toml
[deps]
stdlib = [
    "syscalls", "string", "alloc", "str", "fmt", "vec", "io", "args",
    "assert", "fnptr", "atomic", "sakshi", "math", "ganita", "tagged",
    "hashmap",   # base-dictionary store (the hashbrown replacement)
    "bayan",     # JSON codecs (shabda_to_json / shabda_from_json)
    "mmap",      # memory-mapped lazy dictionary (shabda_lazy_open)
]
```

Add `bench` too if you benchmark against shabdakosh. If a fold is already listed
for another dep, list it once — the set is a union.

## 2. Vendor the dependencies

```sh
cyrius deps
```

This resolves and vendors shabdakosh, svara, varna, and the stdlib folds into your
crate's `lib/` so they can be `include`d. `cyrius deps` reads the
`dist/shabdakosh.deps` sidecar (see §4) to pull svara's backend fold requires in
transitively — you do not list `hisab`/`goonj`/`naad` yourself.

## 3. Include and call it

Cyrius modules **never `include` each other** — your entry file orders the
includes explicitly, deps before dependents. The order that links:

1. the svara prefix-chain, in dependency order: `hisab` → `goonj` → `naad` → `svara`
2. `varna`
3. the shabdakosh bundle

Then call `alloc_init()` once at the top of `main()` before any allocating call
(every dictionary constructor allocates), and use the `shabda_*` API.

### Complete minimal example

This mirrors the exact include order and smoke calls shabdakosh itself builds
with — it links and runs.

```cyrius
# svara phoneme chain (SVARA_PH_* + its transitive backends), in dep order.
include "lib/hisab.cyr"
include "lib/goonj.cyr"
include "lib/naad.cyr"
include "lib/svara.cyr"
# varna: script detection + phoneme inventories.
include "lib/varna.cyr"
# the vendored shabdakosh distlib bundle.
include "lib/shabdakosh.cyr"

fn main(): i64 {
    alloc_init();

    # Built-in English dictionary — 10,617 generated entries.
    var dict = shabda_dict_english();
    println_int(shabda_dict_len(dict));          # -> 10617

    # Script detection (varna-backed): "hello" is Latin script.
    println(shabda_detect_script("hello"));       # -> Latn

    return 0;
}

var r = main();
syscall(SYS_EXIT, r);
```

Build and run it the same way any Cyrius binary is built:

```sh
cyrius deps
cyrius build src/main.cyr build/myapp
./build/myapp
# 10617
# Latn
```

### Looking up a pronunciation

`shabda_dict_lookup(dict, word)` returns the **primary pronunciation** as a `vec`
of svara `SVARA_PH_*` ordinals — or `0` when the word is unknown (see the calling
convention below). Render an ordinal with `shabda_phoneme_to_ipa` /
`shabda_phoneme_to_arpabet`, or the whole vec with `shabda_phonemes_to_ipa`:

```cyrius
var phonemes = shabda_dict_lookup(dict, "hello");
if (phonemes == 0) {
    println("unknown word");
} else {
    println_int(vec_len(phonemes));               # phoneme count
    println(shabda_phonemes_to_ipa(phonemes));    # e.g. "hɛloʊ"
}
```

Other core dictionary entry points (all in `src/dictionary/mod.cyr`):

| Function | Returns |
|---|---|
| `shabda_dict_new()` | empty dictionary handle |
| `shabda_dict_english()` | full English dict (10,617 entries) |
| `shabda_dict_english_minimal()` | small built-in dict (fast, for tests) |
| `shabda_dict_len(d)` | base entry count |
| `shabda_dict_user_len(d)` | user-overlay entry count |
| `shabda_dict_lookup(d, word)` | primary phonemes vec, or `0` |
| `shabda_dict_lookup_entry(d, word)` | full `DictEntry` handle, or `0` |
| `shabda_dict_lookup_all(d, word)` | vec of all pronunciations, or `0` |
| `shabda_dict_insert(d, word, phonemes)` | insert into base map |
| `shabda_dict_insert_user(d, word, phonemes)` | insert into the user overlay |
| `shabda_dict_language(d)` | language code (0 if unset) |
| `shabda_detect_script(word)` | ISO-15924 script code cstring (e.g. `"Latn"`) |

## Calling convention: pointer-or-0 and `shabda_is_err`

shabdakosh is a Cyrius port of a Rust library, so Rust's `Option`/`Result` map to
two flat conventions. Know which one a function uses before you branch on it:

- **`Option<T>` → sentinel.** A function that "returns a payload or nothing"
  returns a **payload pointer, with `0` meaning none**. `shabda_dict_lookup`
  returns the phonemes vec, or `0` for an unknown word — always guard with
  `if (result == 0)` before dereferencing. (The phoneme-sentinel for a *single*
  missing phoneme is `SHABDA_PH_NONE`, i.e. `-1`, not `0`.)

- **`Result<(), E>` → packed sakshi error.** A fallible operation returns a
  **packed i64 error** where `0 == ok` and non-zero is an error, and writes any
  payload to an out-param. Test the return with `shabda_is_err(err)` (1 if error)
  or `shabda_is_ok(err)` (1 if ok). Get diagnostic text with
  `shabda_err_name(err)`. There is no `Display`/`thiserror` — sakshi (the AGNOS
  error substrate) packs `[ctx][category][code]` into the i64.

```cyrius
var err = /* some fallible shabda_* call */;
if (shabda_is_err(err) == 1) {
    println(shabda_err_name(err));                # human-readable diagnostic
}
```

The distinction matters: a `0` from a lookup means "not found" (an `Option::None`),
while a `0` from a fallible operation means "ok" (a `Result::Ok`). The function's
purpose tells you which — payload-returning functions use `0 == none`; action
functions use `0 == ok`.

## 4. The `dist/shabdakosh.deps` sidecar

Alongside the bundle, shabdakosh ships `dist/shabdakosh.deps` — a small
auto-generated sidecar (written by `cyrius distlib`, consumed by `cyrius deps`)
that lists the **stdlib fold requires** the bundle needs left in scope. For
shabdakosh this is svara's backend chain:

```
hisab
goonj
naad
```

You do not edit or hand-manage this file — `cyrius deps` reads it and pulls those
folds in transitively when it vendors shabdakosh. It exists because the distlib is
a flat concatenation with no internal `include`s, so its required folds have to be
declared out-of-band. If a `cyrius build` of your consumer fails with an
undefined `hisab_*`/`goonj_*`/`naad_*` symbol, it means the sidecar was not
honored — re-run `cyrius deps` and confirm the three folds landed in your `lib/`.

## See also

- [`getting-started.md`](getting-started.md) — building shabdakosh itself
- [`usage.md`](usage.md) — the API surface in depth
- `src/main.cyr` — the canonical include order + smoke calls this guide mirrors
- `../development/state.md` — per-module port ledger and locked decisions
