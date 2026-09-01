# 001 — The phoneme↔IPA bridge, and why `dict.validate()` reports so much

**Status**: current as of 3.0.5 (measured 2026-09-01). Describes how the world
is; the open decision it implies is tracked in
[`../development/backlog.md`](../development/backlog.md).

## The bridge has three parties, and they do not share a spelling

A pronunciation in this crate is a sequence of **svara** `SVARA_PH_*` ordinals.
Two different consumers turn those ordinals back into text, and they are not the
same map:

```
svara ordinal ──shbdk_phoneme_to_ipa (src/ipa.cyr)──→ IPA cstring
                                                        │
                                                        └──→ varna phoneme_has(inventory, ipa)
```

`shbdk_phoneme_to_ipa` is the port's *canonical* rendering — one string per
ordinal. `shbdk_ipa_to_phoneme` going the other way is deliberately **lenient**
and accepts several spellings per ordinal (`ɜ`|`ɝ`, `ɡ`|`g`, `ɹ`|`r`, `tʃ`|`t͡ʃ`,
`dʒ`|`d͡ʒ`). So the crate already *knows* those pairs are equivalent — but only
on the way in. Anything that compares the canonical output against a foreign
string table sees only the canonical half.

This is faithful to the oracle: `rust-old/src/ipa.rs` accepts both affricate
spellings in `ipa_to_phoneme` and emits the bare `tʃ` / `dʒ` from
`phoneme_to_ipa`, exactly as the port does.

## Consequence: `shbdk_dict_validate` has a high false-positive rate

`validate_inventory` asks varna "is this IPA string in the language's
inventory?", normalizing only the **length mark** `ː` (exact → `+ː` → strip `ː`).
Measured against the built-in English dictionary and varna 2.4.1's `en`
inventory:

```
shbdk_dict_validate(shbdk_dict_english())  →  5,468 of 10,617 entries "invalid"
```

varna's `en` inventory is 36 phonemes:

```
p b t d k ɡ f v θ ð s z ʃ ʒ h m n ŋ ɹ l w j t͡ʃ d͡ʒ iː ɪ eɪ ɛ æ ɑː ɔː oʊ ʊ uː ʌ ə
```

Every rejection falls into one of three buckets, and only the first is ours:

| rejected IPA | entries | why | whose |
|---|---|---|---|
| `tʃ`, `dʒ` | 1,070 | varna spells affricates with the **tie bar** (`t͡ʃ`, `d͡ʒ`); we emit the bare form. `phoneme_has` is an exact string match and the normalizer only handles `ː`. | **shabdakosh** — but see below |
| `e` | 2,306 | ARPABET `IY` maps to `SVARA_PH_VOWEL_E`, whose canonical IPA is `e`; varna's `en` has `iː`, not `e`. The `IY → VowelE` mapping is the oracle's (`rust-old/src/arpabet.rs:31`), inherited from svara's phoneme model. | svara model |
| `ɜ`, `aɪ`, `aʊ`, `ɔɪ` | 2,051 + 1,307 | varna's `en` inventory has **no NURSE vowel** (neither `ɜ` nor `ɜː`) and only two of the five English diphthongs (`eɪ`, `oʊ`). | **varna** data gap |

## What this is not

It is **not a port regression.** Rust's `inventory_has_normalized`
(`rust-old/src/dictionary/validate.rs:119`) normalizes the length mark and
nothing else, so the Rust crate reports the same entries against the same
inventory. The port is byte-for-byte faithful here.

## The open decision

Teaching `_shbdk_inventory_has_normalized` the tie-bar equivalence the crate
already encodes in `shbdk_ipa_to_phoneme` would retire 1,070 of the false
positives on the shabdakosh side alone. That is a **deliberate divergence from
`rust-old/`**, so per `CLAUDE.md` it needs an ADR before it lands — it is not a
bug fix. The remaining ~4,400 need either a wider varna `en` inventory or a
different `IY` mapping in svara, neither of which is this crate's to make.

Until then: treat `shbdk_dict_validate` as "does this dictionary use only
phonemes varna lists for the language", **not** as "is this dictionary correct".
For English specifically, a clean report is not the expected outcome.
