# ADR-002: Two-Layer Dictionary Lookup

## Status

Accepted (**pre-port** Rust-crate decision, carried unchanged into the CYRIUS port —
the base + user-overlay two-layer lookup is `src/dictionary/mod.cyr`).

## Context

Applications using shabdakosh need to override or extend the built-in dictionary with domain-specific or user-specific pronunciations (e.g., brand names, technical jargon, accent preferences). The override mechanism must be simple to use, serializable, and must not mutate the compiled-in base dictionary.

Options considered:
1. **Single map with insert** — simple, but loses the distinction between "shipped" and "customized" entries
2. **Decorator/chain pattern** — stack multiple dictionaries with precedence, flexible but complex API
3. **Two-layer map** — base (read-heavy, compiled-in) + user overlay (small, mutable), lookup checks user first

## Decision

`PronunciationDict` contains two maps:
- `entries: HashMap<String, DictEntry>` — base dictionary (hashbrown, O(1))
- `user_entries: BTreeMap<String, DictEntry>` — user overlay (sorted, small)

Lookup checks `user_entries` first, then falls back to `entries`. User entries are serialized separately and can be exported/imported independently.

## Consequences

**Benefits:**
- Clear separation: base dictionary is immutable after construction, user entries are the application's responsibility
- `remove_user()` restores the base pronunciation without needing to know what it was
- Serde: user overlay is `skip_serializing_if = "BTreeMap::is_empty"` — v0.1 backward compat
- Merge operations preserve the distinction: `merge()` merges base-to-base and user-to-user
- BTreeMap for user layer gives sorted export (alphabetical CMUdict output)

**Trade-offs:**
- Every lookup checks two maps (BTreeMap first, then HashMap) — negligible for small user overlays
- `diff()` must consider both layers to compute effective differences
- Slightly more complex serialization (two deserializers for backward compat)
