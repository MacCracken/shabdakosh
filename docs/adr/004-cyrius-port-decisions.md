# 004 — CYRIUS port language-mapping decisions

**Status**: Accepted
**Date**: 2026-07-05

## Context

shabdakosh v3.0.0 is a full-parity port of the Rust crate (preserved at `rust-old/`) to the
CYRIUS language. CYRIUS is deliberately minimal — everything is `i64`, raw memory (`load8`/
`store8`, no bounds checks), a flat global namespace (no modules; textual `include`), no
generics/traits, no `serde`/proc-macros/const-eval, no `build.rs`, and a bump allocator that
never frees. Several Rust idioms therefore have no direct equivalent and needed a deliberate
mapping. This ADR records the seven non-obvious decisions in one place so a future reader (or a
sibling AGNOS port) doesn't have to reconstruct the "why". Each is a real choice, not a default.

## Decision

Port every Rust module function-for-function, mapping the seven idioms below. Cross-check every
port against `rust-old/`; the correctness bar is "matches what Rust did."

1. **Errors: `thiserror` enums → sakshi packed-i64.** Errors are `sakshi` packed `i64`
   (`[ctx][category][code]`, `0 == ok`), tested with `shabda_is_err`. Fallible functions return a
   payload pointer with `0 == none/error`, or write the payload to an out-param.
2. **Traits → function-pointer + enum-tag dispatch.** `G2PModel`/`FallbackDict` and the heteronym
   resolver become function pointers (`fncall`); notation and lookup-source become enum-tag
   dispatch. `Option<T>` → sentinels (`SHABDA_PH_NONE` etc.); `Result<T>` → pointer-or-0.
3. **Base dictionary: `build.rs` → generated `.cyr` shards.** `programs/gen_cmudict.cyr` reads
   `data/cmudict-5k.txt` and emits checked-in `src/dictionary/_cmudict_data_N.cyr` packed-string
   globals, loaded into `lib/hashmap` at startup. **Sharded** across files so each stays under
   `cyrius distlib`'s 256 KB per-module read cap (the single file was 283 KB).
4. **Binary format: `postcard` → hand-rolled.** A hand-written little-endian format (`"SHBD"`
   magic + version, u16/u32 length-prefixed) with bounds-checked decode. Not wire-compatible with
   the Rust postcard payload (see Consequences).
5. **Static dict: `phf` compile-time perfect hash → lazy cached singleton.** CYRIUS has no
   const-eval to bake a perfect hash, so `static_dict` is a lazily-built cached singleton over
   `shabda_dict_english()` (one-time ~9.6 ms load; surface + lookup preserved).
6. **WASM: `wasm-bindgen` → plain `.cyr` surface.** `WasmDict` is ported as a normal `.cyr` module
   (`shabda_wasm_dict_*`) with JSON at the boundary. Browser delivery is a toolchain concern, not
   this crate's — the binding is just another `.cyr` surface.
7. **C FFI dropped.** `ffi.rs` is **not ported** — FFI is dead in the CYRIUS/AGNOS stack (no
   C-ABI consumers). It is the one module consciously excluded from the parity port.

## Consequences

- **Positive**: the port keeps the full Rust surface behavior (verified by a 677-assertion suite)
  while fitting CYRIUS's constraints; `dist/shabdakosh.cyr` links flat into AGNOS consumers.
- **Negative / owned**: hand-written codecs and error plumbing we now maintain (no serde/thiserror
  to lean on); the binary + JSON formats are CYRIUS-native and **not wire-compatible** with the
  Rust crate's output; `static_dict` pays a one-time load that `phf` avoided.
- **Neutral**: the phf gap is tracked upstream in the cyrius proposal
  `2026-07-05-const-eval-comptime.md`; the 256 KB distlib cap is tracked in
  `2026-07-05-distlib-per-module-read-cap.md` (raising it would retire the data sharding).

## Alternatives considered

- **Runtime data-file parsing** (instead of generated `.cyr`) — rejected: the toml/JSON loaders
  cap at 256 KB / 16 KB and there is no asset-path resolver. Generated `.cyr` matches the
  varna / cyrius-unicode precedent.
- **A single unsharded data module** — rejected: exceeds the distlib 256 KB per-module read cap.
- **Keeping a C FFI surface** — rejected: no consumer, and it would carry an unused C-ABI layer.
- **Separate ADRs per decision** — deferred: the load-bearing ones (errors, binary, phf, sharding)
  may be split out later; a single consolidated record closes the gap while the rationale is fresh.
