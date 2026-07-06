# ADR-001: hashbrown::HashMap for Base Dictionary

## Status

Accepted (**pre-port** Rust-crate decision). The CYRIUS port uses `lib/hashmap` (the
hashbrown replacement) and drops the `BTreeMap` user overlay for hashmap + sort-on-export; the
`cargo`/criterion details below are Rust-era history. See [ADR-004](004-cyrius-port-decisions.md).

## Context

The base dictionary holds 10,600+ entries that are looked up by word (string key) on every TTS request. The choice of map implementation affects lookup latency, memory layout, and `no_std` compatibility.

Options considered:
1. **`BTreeMap`** (stdlib/alloc) — O(log n) lookup, sorted iteration, no_std compatible
2. **`HashMap`** (stdlib) — O(1) amortized lookup, requires `std`
3. **`hashbrown::HashMap`** — O(1) amortized lookup, no_std compatible (alloc only), SwissTable internals

## Decision

Use `hashbrown::HashMap` for the base dictionary. Keep `BTreeMap` for the user overlay (small, benefits from sorted export).

## Consequences

**Benefits:**
- O(1) lookup for base dictionary (~30 ns miss, ~210 ns hit at 10K entries)
- `no_std` compatible via hashbrown's `default-hasher` and `serde` features
- SwissTable provides better cache behavior than stdlib HashMap for this entry count
- User overlay remains `BTreeMap` — small size makes O(log n) negligible, sorted iteration is useful for CMUdict export

**Trade-offs:**
- Additional dependency (hashbrown) beyond what alloc provides
- `entries()` returns `&hashbrown::HashMap` rather than a stdlib type — minor API friction
- Two different map types in one struct adds conceptual overhead

**Validation:**
- Benchmark: `dict_lookup_hit` ~210 ns, `dict_lookup_miss` ~30 ns (criterion)
- `cargo check --no-default-features` passes — hashbrown works without std
