# Contributing to shabdakosh

Thank you for your interest in contributing to shabdakosh.

## Getting Started

1. Fork the repository
2. Clone your fork
3. Create a feature branch: `git checkout -b feature/your-feature`
4. Make your changes following the guidelines below
5. Submit a pull request

## Development Requirements

- Rust 1.89+ (stable)
- cargo-deny (`cargo install cargo-deny`)
- cargo-audit (`cargo install cargo-audit`)

## Code Quality Requirements

Before submitting a PR, ensure all checks pass:

```sh
cargo fmt --check
cargo clippy --all-features --all-targets -- -D warnings
cargo test --all-features
cargo audit
cargo deny check
RUSTDOCFLAGS="-D warnings" cargo doc --all-features --no-deps
```

## Code Standards

- `#[non_exhaustive]` on all public enums
- `#[must_use]` on all pure functions
- Zero `unwrap`/`panic` in library code — use `Result` or safe defaults
- All public types must derive `Serialize`, `Deserialize`, `Debug`, `Clone`
- All new types require serde roundtrip tests
- Dictionary-first, accuracy over speed

## Adding Dictionary Entries

1. Add entries to `data/cmudict-5k.txt` in CMUdict ARPABET format
2. For heteronyms, use `WORD(n)` variant convention with `@freq` annotations
3. Ensure every vowel has a stress digit (0, 1, or 2)
4. Consonants never have stress digits
5. Run `cargo build` to verify the build script parses correctly
6. Add integration tests for new words

## Benchmarks

All performance-related changes must include benchmark results. Run:

```sh
cargo bench
# or
./scripts/bench-history.sh
```

## License

By contributing, you agree that your contributions will be licensed under GPL-3.0.
