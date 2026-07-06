# Architecture Decision Records

Decisions about shabdakosh — what we chose, the context, and the consequences we accept. Use these when a future reader would reasonably ask *"why did we do it this way?"*

## Conventions

- **Filename**: `NNN-kebab-case-title.md`, zero-padded to three digits (matching `001`–`004`). Never renumber.
- **One decision per ADR.** If a decision supersedes a prior one, add a new ADR and set the old one's status to `Superseded by NNNN`.
- **Status lifecycle**: `Proposed` → `Accepted` → (optionally) `Superseded` or `Deprecated`.
- Use [`template.md`](template.md) as the starting point.

## ADR vs. architecture note vs. guide

| Kind | Lives in | Answers |
|---|---|---|
| ADR | `docs/adr/` | *Why did we choose X over Y?* |
| Architecture note | `docs/architecture/` | *What non-obvious constraint is true about the code?* |
| Guide | `docs/guides/` | *How do I do X?* |

## Index

| ADR | Title | Status |
|---|---|---|
| [001](001-hashbrown-for-base-dictionary.md) | hashbrown for the base dictionary | Accepted (pre-port; the CYRIUS port uses `lib/hashmap`, the hashbrown replacement) |
| [002](002-two-layer-lookup.md) | Two-layer (base + user overlay) lookup | Accepted (carried into the port) |
| [003](003-varna-feature-gate.md) | varna feature gate | Accepted (pre-port; the port links varna unconditionally — no feature gate) |
| [004](004-cyrius-port-decisions.md) | CYRIUS port language-mapping decisions | Accepted |

001–003 predate the Rust→CYRIUS port and describe the Rust crate; 004 records the port decisions.
