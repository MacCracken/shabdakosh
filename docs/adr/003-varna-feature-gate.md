# ADR-003: Feature-Gated varna Integration

## Status

Accepted

## Context

v1.1.0 adds multi-language support via varna (phoneme inventories, lexicons, script metadata for 51 languages). varna requires `std` and adds a non-trivial dependency tree (serde, tracing). Consumers that only need English dictionary lookup should not pay for this.

Options considered:
1. **Always included** — simpler API, but forces varna on all consumers including `no_std`
2. **Separate crate** (`shabdakosh-varna`) — maximum isolation, but fragments the API
3. **Feature-gated modules** — opt-in via `varna` feature flag, single crate

## Decision

Feature-gate varna integration behind a `varna` feature that implies `std`:

```toml
[features]
varna = ["std", "dep:varna"]
full = ["std", "json", "varna"]
```

New modules `dictionary::validate` and `dictionary::detect` are compiled only when `varna` is enabled. Methods on `PronunciationDict` that use varna (`validate()`, `from_lexicon()`, `spanish()`, `hindi()`, `german()`, `sanskrit()`) are `#[cfg(feature = "varna")]`.

## Consequences

**Benefits:**
- `cargo check --no-default-features` still passes — core is `no_std` compatible
- Default feature set (`std` only) has zero new dependencies
- `full` feature provides one-line opt-in for AGNOS stack consumers
- Validation, detection, and seed dictionaries are cleanly separated

**Trade-offs:**
- `#[cfg(feature = "varna")]` gating on methods and modules adds conditional complexity
- Doc examples for varna features use `# #[cfg(feature = "varna")]` guards
- IPA normalization (length mark handling) is needed because svara and varna use different IPA conventions — this is an ongoing maintenance surface

**Validation:**
- `cargo test --all-features` — 122 tests pass (62 unit + 53 integration + 7 doc)
- `cargo check --no-default-features` — no_std builds without varna
- `cargo test` (default features) — 77 tests pass without varna
