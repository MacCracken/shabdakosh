# Runnable examples

Each file here is a complete CYRIUS program. Build and run it from the
**repository root** (the `include` paths are root-relative, and `cyrius build`
resolves the stdlib and the svara/varna deps from `cyrius.cyml`):

```sh
cyrius deps                                                   # once, to vendor lib/
cyrius build docs/examples/quickstart.cyr build/quickstart && ./build/quickstart
cyrius build docs/examples/roundtrip.cyr  build/roundtrip  && ./build/roundtrip
```

| example | what it shows |
|---|---|
| [`quickstart.cyr`](quickstart.cyr) | The smallest useful program: open the built-in English dictionary, look a word up, render it as IPA / ARPABET / X-SAMPA. |
| [`roundtrip.cyr`](roundtrip.cyr) | The full surface end to end: build a dictionary, look up, apply and remove a user override, then round-trip through CMUdict text, IPA text, JSON, PLS, SSML and the binary format (in memory and on disk), finishing with a prefix search and a coverage report over the built-in dictionary. |

## Things these examples exist to show you

- **`alloc_init()` runs first.** The bump allocator returns 0 until it is
  initialised, so every entry point calls it before touching the heap.
- **The entry file owns the include order.** Modules never include each other;
  they are pasted in tier order (svara chain → `error` → leaves → keystone →
  extensions → formats). Copy the block from an example and delete what you
  do not need.
- **A pronunciation is a vec of `SVARA_PH_*` ordinals.** That vec — not a
  string — is the interchange format across the AGNOS stack; it is what makes
  the output compatible with svara's `PhonemeEvent`.
- **Absence is `0`, not an error.** `shbdk_dict_lookup` on a missing word
  returns `0`, parsers return `0` on malformed input, and nothing panics. Errors
  that need a *reason* are packed sakshi codes (`src/error.cyr`).
- **`println` dispatches on the static return type.** A function returning a
  cstring must be annotated `: cstring`, or `println` prints the pointer as a
  number. Both examples define a one-line `say(s: cstring)` for this.

For the API reference and worked snippets grouped by task, see
[`../guides/usage.md`](../guides/usage.md); for consuming the built bundle from
another crate, see
[`../guides/consuming-the-distlib.md`](../guides/consuming-the-distlib.md).
