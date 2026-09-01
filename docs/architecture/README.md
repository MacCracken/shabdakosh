# Architecture notes

Non-obvious constraints, quirks, and invariants that a reader cannot derive from the code alone. Numbered chronologically — never renumber.

Not decisions (those live in [`../adr/`](../adr/)) and not guides (those live in [`../guides/`](../guides/)). An item here describes *how the world is*, not *what we chose* or *how to do something*.

## Items

- [001 — The phoneme↔IPA bridge, and why `dict.validate()` reports so much](001-phoneme-ipa-bridge-and-validation.md)
  — the canonical `phoneme_to_ipa` rendering vs the lenient `ipa_to_phoneme` parser, and why validating
  the built-in English dictionary against varna's `en` inventory flags 5,468 of 10,617 entries.

Add the next numbered entry (`002-kebab-case-title.md`) the next time the code has a non-obvious invariant a reader can't derive. Do not write entries for decisions — those are ADRs.
