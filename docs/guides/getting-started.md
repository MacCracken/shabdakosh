# Getting started with shabdakosh

## Build

```sh
cyrius deps                              # resolve dependencies
cyrius build src/main.cyr build/shabdakosh    # compile
cyrius test tests                        # run every .tcyr suite
```

## Layout

- `src/main.cyr` — entry point. Top-level `var r = main(); syscall(SYS_EXIT, r);`.
- `tests/` — test suite (`.tcyr` files, auto-discovered by `cyrius test tests`).
- The original Rust source was removed after 3.1.3 and lives at that tag: `git show 3.1.3:rust-old/<path>`. It is
  the functionality reference (what the port must be able to do); accuracy decides whether a result is right.

## Adding a feature

1. Edit `src/main.cyr` (or add a new module and `include` it).
2. If the Rust had the capability, read it at tag 3.1.3 (`git show 3.1.3:rust-old/<path>`); make the port accurate,
   not merely the same ([ADR 005](../adr/005-accuracy-over-rust-parity.md)).
3. Add test cases to the module's suite, `tests/<module>.tcyr`.
4. Run `cyrius test tests/<suite>.tcyr` (one suite) or `cyrius test tests` (all).
5. Bump `VERSION` and add a CHANGELOG entry before tagging.

See [`../adr/template.md`](../adr/template.md) when a non-trivial design choice deserves an ADR.
