# 005 — Accuracy over Rust parity: `rust-old/` is the capability list, not the oracle

**Status**: Accepted
**Date**: 2026-11-17
**Amends**: [004](004-cyrius-port-decisions.md) ("the correctness bar is 'matches what Rust did'")
**Note**: `rust-old/` was removed after 3.1.3; it lives at that tag (`git show 3.1.3:rust-old/<path>`).

## Context

ADR 004 made `rust-old/` the correctness oracle: a result was right if the Rust produced it, and
the port could differ only with an ADR. That bar got the port to full parity, but it meant
copying the Rust's bugs. By 3.1.2 the roadmap held five *Open decisions*, each a place where the
Rust was wrong and the port faithfully reproduced it:

- `validate()` false positives;
- ARPABET notation dropping vowels;
- CMUdict keys with stray spaces;
- phonotactics for three languages only;
- a frequency of -1.0 reading back as none.

The M3 sweep before retiring `rust-old/` found many more. It ran seven parallel reviews, each
putting the same inputs through a built copy of the Rust crate and through the port:

- `IY` mapped to /e/;
- 54 of svara's 100 phonemes had no IPA;
- phonotactic checks fired mid-word, and all 182 English violations were false;
- `syllabify` was not Maximal Onset;
- readers silently defaulted malformed data;
- the Swadesh constructors invented frequencies.

The maintainer set the rule for the sweep: *rust-old is not the oracle but functionality coverage;
accuracy is the oracle. Do not bring over broken Rust.*

## Decision

**`rust-old/` defines what the port must be able to do; accuracy decides whether a result is
right.** Accuracy means:

- the linguistics (IPA and ARPABET as defined, the Maximal Onset Principle, phonotactic positions);
- the relevant specification (CMUdict, W3C PLS, SSML, XML, JSON, IEEE 754, Unicode);
- the documented contract (a frequency is 0.0–1.0);
- what a consumer would reasonably expect: a reader never silently invents or drops data, and a
  writer never produces a file its reader refuses.

Each difference from the Rust falls in one of three classes:

1. **A capability the port lacks** → port it. Examples: a JSON codec for every type the Rust
   serialized, parse diagnostics, `Clone` / `PartialEq`, and the v0.1 JSON shape.
2. **The port is inaccurate** → fix it. Examples: the Swadesh frequency, shared buffers, Unicode
   whitespace, the binary region crash.
3. **The Rust is inaccurate** → the port is accurate, and the CHANGELOG records the output change.

A consumer-visible output change is never silent. The CHANGELOG lists it, and this ADR is the
standing record of why the port's output differs from the Rust's.

### The output changes this decision made (3.1.3)

| Area | Rust / 3.1.2 | Now | Why |
|---|---|---|---|
| ARPABET `IY` | /e/ (`VOWEL_E`) | /iː/ (`VOWEL_LONG_I`) | IY is FLEECE; svara synthesised "see" as "say" |
| IPA bridge | 46 of 100 phonemes | all 100 | the rest were dropped on export, skipped on parse |
| IPA affricates | `tʃ` | `t͡ʃ` (bare `tʃ` still read) | the tie bar is the unambiguous IPA, varna's spelling |
| IPA `r` | the approximant ɹ | the alveolar trill | IPA's definition |
| IPA writer | `aɪ` for `[a, ɪ]` | `a.ɪ` | a transcription must read back as its phonemes |
| X-SAMPA | 46 phonemes, `r` approximant | all 100, `r` trill | X-SAMPA's definition |
| frequency | any f32 | 0.0–1.0, enforced | the documented contract; NaN broke sorting, -1.0 collided |
| `@freq` text | 6 decimals | shortest exact | a value must survive its own export |
| phonotactics | adjacent pairs vs onset rules; en/sa/ja | by syllable position; 12 languages | 182 of 182 English violations were false |
| `syllabify` | one consonant always coda | Maximal Onset; all svara vowels | "astray" was `əs.tɹeɪ`, "über" one syllable |
| ARPABET parse | no stress digits | stress digits | `parse(render(v))` lost every vowel |
| text | ASCII whitespace and case | Unicode | NBSP, U+3000, "ÜBER", curly quotes |
| CMUdict words | text before the first `(` | trailing `(n)` cut, then trimmed | `(PAREN` was empty, `read (1)` was `"read "` |
| readers | default on bad input | refuse, with a reason | silent VowelA / 0.0 / dropped regions |
| writers | write anything | refuse what would not load back | unreadable files, a u16 wrap |
| `from_lexicon` | 1/(1+Swadesh index) | 1/(1+frequency rank) | the index is a concept number |

## Consequences

- **Positive**:
  - The port is more correct than the Rust everywhere they differ, and each difference is written
    down.
  - `rust-old/` could be retired without losing anything, and was, after 3.1.3. Its 53 integration tests live on in
    `tests/integration.tcyr`, and its unit tests are carried in the module suites.
  - The five *Open decisions* are settled.
- **Negative**:
  - Outputs change for consumers (released as 3.1.3, with the changes listed in the CHANGELOG):
    - English pronunciations with `IY`;
    - IPA text: tie bars, the extended symbols, syllable breaks;
    - inputs the readers used to accept.
  - The readers and writers do more checking: `to_ipa` +12%, `to_cmudict` +8–10%, binary +1–14%.
- **Neutral**:
  - Some inaccuracies are not this crate's to fix, and are tracked as upstream items in the
    roadmap:
    - varna's English inventory lacks /ɜ/, /aɪ/, /aʊ/ and /ɔɪ/;
    - varna's Latin range stops at U+024F;
    - Japanese kanji-only words never hint `ja`.
  - svara has no nasal-vowel phonemes, so the IPA reader keeps a nasal vowel's base vowel.

## Alternatives considered

- **Keep Rust parity, record each defect as an ADR'd divergence** (ADR 004's rule). It was
  rejected because it ships the Rust's bugs to every AGNOS consumer, and an ADR per bug documents a
  defect instead of removing it.
- **Strict IPA reading (a bare `tʃ` is t + ʃ).** It was rejected because files written before 3.1.3,
  and the common broad transcriptions, spell the affricate without a tie bar. The writer's syllable
  break keeps the port's own round trip lossless instead.
- **Clamping an out-of-range frequency.** It was rejected because a silent change to the value is
  worse than refusing it; the builder keeps the previous frequency, and the readers refuse.
