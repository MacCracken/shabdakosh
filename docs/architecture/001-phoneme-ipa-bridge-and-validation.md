# 001 — The phoneme↔IPA bridge, and what `dict.validate()` still reports

**Status**: current as of the 3.2 work (2026-11-17), measured on varna 2.6.0. English
`validate()` flags **3,113 of 10,617** entries (5,468 through 3.1.2). Every remaining flag is a
gap in varna's English inventory, which is varna's to fill (roadmap, *Gated on something outside
this repository*).

## The bridge

A pronunciation in this crate is a sequence of **svara** `SVARA_PH_*` ordinals. Validation turns
each ordinal into IPA and asks varna whether the language's inventory has it:

```
svara ordinal ──shbdk_phoneme_to_ipa (src/ipa.cyr)──→ IPA cstring
                                                        │
                                                        └──→ varna phoneme_has(inventory, ipa)
```

`shbdk_phoneme_to_ipa` writes one canonical symbol per ordinal, and covers all 100 svara phonemes
that have one (silence has none). Affricates carry their tie bar (`t͡ʃ`, `d͡ʒ`), as varna spells
them. `shbdk_ipa_to_phoneme` and `shbdk_parse_ipa_word` are lenient on the way in and accept the
common alternates: `ɝ`/`ɚ` for `ɜ`, `g` for `ɡ`, a bare `tʃ` / `dʒ`, the tie below `t͜ʃ`, the
ligatures `ʧ ʤ ʦ ʣ`, and `ɫ` for `l`.

Membership (`_shbdk_inventory_has_normalized`) treats two spellings as one phoneme:

- **length:** exact, then `+ː`, then without a final `ː` (svara's English vowels are unmarked, while
  varna's `en` lists `ɑː ɔː uː`);
- **the affricate tie, either way:** `t͡ʃ` and `tʃ`.

## What changed in 3.2, and why the count fell

| Rejected IPA | 3.1.2 | 3.2 | What happened |
|---|---:|---:|---|
| `tʃ`, `dʒ` | 1,070 | 0 | The port wrote bare affricates and varna lists tied ones. The writer now ties them, and membership accepts either. |
| `e` | 2,306 | 0 | ARPABET `IY` mapped to `SVARA_PH_VOWEL_E` (/e/). It is the FLEECE vowel, `SVARA_PH_VOWEL_LONG_I` (/iː/), which varna's `en` has. |
| `ɜ` | 2,051 | 2,051 | varna's `en` has no NURSE vowel (neither `ɜ` nor `ɜː`) |
| `aɪ` | 970 | 970 | varna's `en` lists only two of the five English diphthongs (`eɪ`, `oʊ`) |
| `aʊ` | 250 | 250 | (same) |
| `ɔɪ` | 87 | 87 | (same) |

Both fixed rows were the Rust's behaviour too: `rust-old/` emitted bare affricates and mapped `IY`
to /e/. They are fixed for accuracy ([ADR 005](../adr/005-accuracy-over-rust-parity.md)), not
parity.

varna's English inventory is 36 phonemes:

```
p b t d k ɡ f v θ ð s z ʃ ʒ h m n ŋ ɹ l w j t͡ʃ d͡ʒ iː ɪ eɪ ɛ æ ɑː ɔː oʊ ʊ uː ʌ ə
```

The four remaining rejections are all its gaps: the NURSE vowel and the diphthongs `aɪ aʊ ɔɪ`.
They are reported upstream rather than worked around. Dropping them from the check here would
hide a real inventory question from every other varna consumer.

## Reading a report

Treat `shbdk_dict_validate` as "does this dictionary use only phonemes varna lists for the
language". For English on varna 2.6.0, the 3,113 flags are the inventory gap above, not
transcription errors. Phonotactic validation (`shbdk_dict_validate_phonotactics`) has no such
noise: the built-in English dictionary has 0 violations against varna's English profile.
