# Usage Guide

## Getting Started

Add `shabdakosh` to `Cargo.toml`:

```toml
[dependencies]
shabdakosh = "1.1"
```

Feature flags:

| Flag | Enables |
|------|---------|
| `std` (default) | Standard library, file I/O |
| `json` | JSON import/export via serde_json |
| `varna` | Multi-language: validation, lexicon ingestion, script detection |
| `full` | All features |

```toml
# Multi-language support
shabdakosh = { version = "1.1", features = ["varna"] }

# Everything
shabdakosh = { version = "1.1", features = ["full"] }
```

---

## Dictionary Lookup

The built-in English dictionary has 10,600+ entries compiled from CMUdict:

```rust
use shabdakosh::PronunciationDict;

let dict = PronunciationDict::english();

// Simple lookup returns primary pronunciation phonemes
if let Some(phonemes) = dict.lookup("hello") {
    println!("hello has {} phonemes", phonemes.len());
}

// Full entry access for variant pronunciations
if let Some(entry) = dict.lookup_entry("read") {
    println!("read has {} pronunciations", entry.len());
    for p in entry.all() {
        println!("  {:?} (freq: {:?})", p.phonemes(), p.frequency());
    }
}

// Region-specific lookup
use shabdakosh::Region;
if let Some(entry) = dict.lookup_entry("read") {
    if let Some(ga) = entry.for_region(Region::GeneralAmerican) {
        println!("GA pronunciation: {:?}", ga.phonemes());
    }
}
```

---

## User Overlay

Override or extend the base dictionary. User entries take precedence during lookup:

```rust
use shabdakosh::PronunciationDict;
use svara::phoneme::Phoneme;

let mut dict = PronunciationDict::english();

// Add a custom word
dict.insert_user("agnos", &[
    Phoneme::VowelAsh, Phoneme::PlosiveG,
    Phoneme::NasalN, Phoneme::VowelO, Phoneme::FricativeS,
]);
assert!(dict.lookup("agnos").is_some());

// Override an existing word
dict.insert_user("the", &[Phoneme::FricativeDh, Phoneme::VowelE]);

// Remove override (restores base pronunciation)
dict.remove_user("the");
```

---

## Import and Export

### CMUdict Format

```rust
use shabdakosh::dictionary::format;

let input = "hello  HH AH0 L OW1\nworld  W ER1 L D\n";
let dict = format::parse_cmudict(input).unwrap();
let output = format::to_cmudict(&dict);
```

Extended format supports `@freq` and `@region` annotations:

```text
;;; @freq=0.7
read  R IY1 D
;;; @freq=0.3
read(2)  R EH1 D
;;; @region=RP
water(2)  W AO1 T AH0
```

### IPA Format

```rust
use shabdakosh::dictionary::format;

let input = "hello /hɛloʊ/\nworld /wɜld/\n";
let dict = format::parse_ipa(input).unwrap();
let output = format::to_ipa(&dict);
```

### W3C PLS (Pronunciation Lexicon Specification)

```rust
use shabdakosh::dictionary::format::pls;

let xml = r#"<lexicon version="1.0" alphabet="ipa" xml:lang="en-US">
  <lexeme>
    <grapheme>hello</grapheme>
    <phoneme>hɛloʊ</phoneme>
  </lexeme>
</lexicon>"#;

let dict = pls::parse_pls(xml).unwrap();
let output = pls::to_pls(&dict, "en-US");
```

### SSML Phoneme Tags

```rust
use shabdakosh::dictionary::format::ssml;
use svara::phoneme::Phoneme;

// Generate SSML tag
let tag = ssml::to_ssml_phoneme("cat", &[
    Phoneme::PlosiveK, Phoneme::VowelAsh, Phoneme::PlosiveT,
]);
// <phoneme alphabet="ipa" ph="kæt">cat</phoneme>

// Parse SSML tag
let (word, phonemes) = ssml::parse_ssml_phoneme(&tag).unwrap();
```

### JSON (requires `json` feature)

```rust
use shabdakosh::dictionary::format;

let dict = shabdakosh::PronunciationDict::english_minimal();
let json = format::to_json(&dict).unwrap();
let roundtripped = format::from_json(&json).unwrap();
```

---

## Dictionary Operations

### Merge

```rust
use shabdakosh::PronunciationDict;

let mut base = PronunciationDict::english();
let medical = PronunciationDict::new(); // domain-specific dict

// Override merge: medical entries replace base on conflict
base.merge(&medical);

// Conservative merge: keep base entries on conflict
base.merge_conservative(&medical);
```

### Diff

```rust
use shabdakosh::dictionary::diff;

let v1 = shabdakosh::PronunciationDict::english_minimal();
let mut v2 = v1.clone();
v2.insert("newword", &[svara::phoneme::Phoneme::VowelA]);

let changes = diff(&v1, &v2);
println!("added: {:?}", changes.added);     // ["newword"]
println!("removed: {:?}", changes.removed); // []
println!("changed: {:?}", changes.changed); // []
```

---

## Language Tagging

Dictionaries can be tagged with an ISO 639 language code:

```rust
use shabdakosh::PronunciationDict;

// Built-in English dictionaries are tagged automatically
let dict = PronunciationDict::english();
assert_eq!(dict.language(), Some("en"));

// Tag a custom dictionary
let dict = PronunciationDict::new().with_language("fr");
assert_eq!(dict.language(), Some("fr"));

// Mutable setter
let mut dict = PronunciationDict::new();
dict.set_language("de");
```

---

## Multi-Language Support (varna feature)

Enable with `features = ["varna"]` in Cargo.toml.

### Inventory Validation

Check that a dictionary's phonemes are valid for its target language:

```rust
use shabdakosh::PronunciationDict;
use svara::phoneme::Phoneme;

// Validate against the dictionary's own language
let mut dict = PronunciationDict::new().with_language("en");
dict.insert("pat", &[Phoneme::PlosiveP, Phoneme::VowelAsh, Phoneme::PlosiveT]);
let report = dict.validate().unwrap();
assert!(report.is_valid());

// Validate against a specific inventory
use shabdakosh::dictionary::validate::validate_inventory;
let inventory = varna::phoneme::inventories::spanish();
let report = validate_inventory(&dict, &inventory);
// report.invalid_entries lists phonemes not in Spanish
```

### Lexicon Ingestion

Convert a varna `Lexicon` (e.g., Swadesh word list) into a pronunciation dictionary:

```rust
use shabdakosh::PronunciationDict;

let lexicon = varna::lexicon::swadesh::by_code("es").unwrap();
let dict = PronunciationDict::from_lexicon(&lexicon);
assert_eq!(dict.language(), Some("es"));
assert!(dict.lookup("agua").is_some());
```

### Seed Dictionaries

Convenience constructors for common languages (built from varna Swadesh lists):

```rust
use shabdakosh::PronunciationDict;

let es = PronunciationDict::spanish();   // ~25 entries
let hi = PronunciationDict::hindi();     // ~25 entries
let de = PronunciationDict::german();    // ~25 entries
let sa = PronunciationDict::sanskrit();  // empty, language-tagged
```

### Script and Language Detection

Identify writing systems and suggest languages from Unicode code points:

```rust
use shabdakosh::dictionary::detect;

// Detect script
assert_eq!(detect::detect_script("hello"), Some("Latn"));
assert_eq!(detect::detect_script("नमस्ते"), Some("Deva"));
assert_eq!(detect::detect_script("مرحبا"), Some("Arab"));

// Suggest languages for a script
let hints = detect::detect_language_hint("नमस्ते");
assert!(hints.contains(&"hi"));
assert!(hints.contains(&"sa"));

// Script name
let (code, name) = detect::detect_script_name("γεια").unwrap();
assert_eq!(name, "Greek");
```

---

## IPA Conversion

Bidirectional mapping between IPA strings and svara phonemes:

```rust
use shabdakosh::ipa;
use svara::phoneme::Phoneme;

// Single symbol
assert_eq!(ipa::ipa_to_phoneme("ʃ"), Some(Phoneme::FricativeSh));
assert_eq!(ipa::phoneme_to_ipa(&Phoneme::FricativeSh), Some("ʃ"));

// Full transcription (greedy longest-match parser)
let phonemes = ipa::parse_ipa_word("hɛˈloʊ");
assert_eq!(phonemes.len(), 4);

// Phoneme sequence to IPA string
let ipa_str = ipa::phonemes_to_ipa(&phonemes);
```

---

## ARPABET Conversion

Bidirectional mapping between CMU ARPABET notation and svara phonemes:

```rust
use shabdakosh::arpabet;
use svara::phoneme::Phoneme;

assert_eq!(arpabet::arpabet_to_phoneme("SH"), Some(Phoneme::FricativeSh));
assert_eq!(arpabet::phoneme_to_arpabet(&Phoneme::FricativeSh), Some("SH"));

// Stress-aware conversion (AH0 -> schwa, AH1/AH2 -> cup-v)
assert_eq!(arpabet::arpabet_to_phoneme_with_stress("AH0"), Some(Phoneme::VowelSchwa));
assert_eq!(arpabet::arpabet_to_phoneme_with_stress("AH1"), Some(Phoneme::VowelCupV));
```

---

## Serde Serialization

All types implement `Serialize` and `Deserialize`. Backward-compatible with v0.1 format:

```rust
let dict = shabdakosh::PronunciationDict::english_minimal();
let json = serde_json::to_string(&dict).unwrap();
let roundtripped: shabdakosh::PronunciationDict = serde_json::from_str(&json).unwrap();
assert_eq!(dict.len(), roundtripped.len());
```
