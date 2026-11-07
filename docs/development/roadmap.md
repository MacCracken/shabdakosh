# shabdakosh — Roadmap

> Milestone plan through v1.0. State lives in [`state.md`](state.md);
> this file is the sequencing — what ships, in what order, against
> what dependency gates.

The port shipped straight to **v3.0.0** (full parity with the Rust 2.x surface), so the
original v0.x/v1.0 sequencing collapsed into a single parity milestone.

## Release criteria (v3.0.0)

- [x] Rust → Cyrius surface parity verified (function-level against `rust-old/`; every module ✅ or consciously dropped)
- [x] Test coverage adequate for the surface area (873 assertions / 26 suites, all green — 689 at
      the 3.0.0 release gate, +132 regressions from the 3.0.4 P1 sweep, +52 from the 3.0.5 backlog sweep)
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

- **C FFI** (`ffi.rs`) — dropped as redundant: CYRIUS interops with C directly, so a dedicated FFI
  shim buys nothing, and there are no C-ABI consumers today. A backdoor exists to add one if needed.
- **Compile-time `phf` perfect hash** — CYRIUS has no const-eval; `static_dict` ships as a lazy
  cached singleton instead (surface preserved). Tracked upstream in the cyrius proposal
  `2026-07-05-const-eval-comptime.md`.
- **(all release gates met)** — the security audit is complete; every criterion above is ✅.

---

## Moving the cyrius pin to 6.6.6

**Current pin:** `cyrius = "6.6.3"` (`cyrius.cyml`).

No source change needed. One latent item is worth writing down.

**shabdakosh does write files, via the stdlib helpers rather than raw flags:** 5
`file_write_all` call sites and 7 `file_exists` / `file_read_all` sites (dictionary
load and cache paths, plus `programs/gen_cmudict.cyr`). There is no raw
`sys_open`, no `O_APPEND` and no `O_TRUNC` in shabdakosh's own source. But
`file_write_all` opens `O_WRONLY|O_CREAT|O_TRUNC` under the hood
(`lib/io.cyr:546`), so **if shabdakosh is ever built for Windows** those five
sites carried the pre-6.6.6 PE bug where `O_TRUNC` did not truncate — a rewritten
dictionary cache that is shorter than the previous one kept the old tail, which
for a parsed dictionary format is a silent corruption rather than a parse error.

That is latent, not live: shabdakosh has **zero `CYRIUS_TARGET_WIN` guards**, CI
is `ubuntu-latest` only, and nothing in the repo claims a PE target. So this is a
"good to have fixed before anyone tries" item, not a blocker. The companion fix
helps here too — `file_exists` and `file_read_all` open read-only, and 6.6.6
stops PE requesting write access for a read, so on Windows they would begin
succeeding against a read-only dictionary file or volume.

**Everything 6.6.6 turns into a compile error was checked and is absent:** 0
structs (no different-struct-copy site, no by-value deep-copy site), 0 `async fn`,
0 `operator` fns, 0 `ret2` / `rethi`, no SIMD intrinsics. The 46 `: cstring`
annotations are all **return** types (`shbdk_phoneme_to_ipa`,
`shbdk_notation_render`, the `src/wasm.cyr` surface) — 6.6.6's rule is about a
literal int passed into a `: cstring` **param**, and there are none. No `var`
inside a top-level block (the `tests/json.tcyr` and `tests/notation.tcyr`
candidates are all bodies of `fn`s whose signatures wrap across two lines). No own
`vec_*` definitions, so `assert.cyr`'s new transitive `vec.cyr` include cannot
collide with the 26 assert call sites. No raw `SYS_STATFS`. `lib/regression.cyr`
is not vendored, so its new exec deadline and `PR_SET_PDEATHSIG` behaviour does
not reach here. `lib/` holds no symlinks and `cyrius.lock` is writable, so 6.6.6's
fail-hard `cyrius deps` / `publish` is a no-op.

The `r` global appears at both `src/main.cyr:130` and
`programs/gen_cmudict.cyr:367`, but those are two separate entry points that are
never co-linked, so 6.6.6's "a later redeclaration now wins everywhere" flip and
the new different-type-co-linked-global error do not apply.

After bumping: `cyrius test` and regenerate `dist/`.
