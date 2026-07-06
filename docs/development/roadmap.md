# shabdakosh — Roadmap

> Milestone plan through v1.0. State lives in [`state.md`](state.md);
> this file is the sequencing — what ships, in what order, against
> what dependency gates.

The port shipped straight to **v3.0.0** (full parity with the Rust 2.x surface), so the
original v0.x/v1.0 sequencing collapsed into a single parity milestone.

## Release criteria (v3.0.0)

- [x] Rust → Cyrius surface parity verified (function-level against `rust-old/`; every module ✅ or consciously dropped)
- [x] Test coverage adequate for the surface area (689 assertions / 26 suites, all green)
- [x] Benchmarks captured in [`docs/benchmarks.md`](../benchmarks.md)
- [x] At least one downstream consumer green (`dist/shabdakosh.cyr` linked + exercised by a consumer smoke)
- [x] CHANGELOG complete (3.0.0 entry)
- [x] Security audit pass ([`docs/audit/2026-07-05-audit.md`](../audit/2026-07-05-audit.md) — 9 findings, all fixed + regression-tested)

## Milestones

### M0 — Port scaffold (v0.1.0) — ✅ shipped 2026-07-05

- `cyrius port` scaffold landed
- Rust source moved to `rust-old/`
- Doc-tree per [first-party-documentation.md](https://github.com/MacCracken/agnosticos/blob/main/docs/development/applications/first-party-documentation.md)

### M1 — Full parity port (v3.0.0) — ✅ shipped 2026-07-05

Every Rust module ported function-for-function to CYRIUS and cross-checked against `rust-old/`:
error/arpabet/ipa/entry/morphology/syllable/notation, the dictionary keystone + coverage/stream/
trie/heteronym/g2p/static_dict, all format codecs (CMUdict/IPA/PLS/SSML/JSON/binary), lazy (mmap),
detect + validate (varna), and the WASM surface. Base CMUdict data generated as a single `.cyr` module (`_cmudict_data.cyr`).
distlib bundle built + consumer-verified. See [`state.md`](state.md) for the per-module ledger.

## Out of scope (v3.0.0)

- **C FFI** (`ffi.rs`) — dropped; dead in the CYRIUS/AGNOS stack (no C-ABI consumers).
- **Compile-time `phf` perfect hash** — CYRIUS has no const-eval; `static_dict` ships as a lazy
  cached singleton instead (surface preserved). Tracked upstream in the cyrius proposal
  `2026-07-05-const-eval-comptime.md`.
- **(all release gates met)** — the security audit is complete; every criterion above is ✅.
