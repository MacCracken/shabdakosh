//! Integration tests for shabdakosh.

use shabdakosh::PronunciationDict;
use shabdakosh::dictionary::format;
use shabdakosh::dictionary::{self, DictDiff};

// --- Dictionary size and coverage ---

#[test]
fn test_expanded_dictionary_size() {
    let dict = PronunciationDict::english();
    assert!(
        dict.len() >= 10000,
        "expanded dictionary should have 10000+ entries, got {}",
        dict.len()
    );
}

#[test]
fn test_expanded_dictionary_common_words() {
    let dict = PronunciationDict::english();
    let words = [
        "people",
        "because",
        "through",
        "enough",
        "beautiful",
        "colonel",
        "psychology",
        "knight",
        "thought",
        "language",
        "world",
        "hello",
        "the",
        "computer",
        "science",
        "music",
        "water",
        "friend",
        "school",
        "house",
    ];
    for word in words {
        assert!(
            dict.lookup(word).is_some(),
            "'{word}' should be in the dictionary"
        );
    }
}

#[test]
fn test_minimal_dictionary_still_works() {
    let minimal = PronunciationDict::english_minimal();
    assert_eq!(minimal.len(), 29);
    assert!(minimal.lookup("the").is_some());
    assert!(minimal.lookup("hello").is_some());
    assert!(minimal.lookup("computer").is_none());
}

// --- User overlay ---

#[test]
fn test_user_overlay_precedence() {
    let mut dict = PronunciationDict::english();
    let original = dict.lookup("hello").unwrap().to_vec();

    let custom = &[svara::phoneme::Phoneme::VowelA];
    dict.insert_user("hello", custom);

    assert_eq!(dict.lookup("hello").unwrap(), custom);

    assert!(dict.remove_user("hello"));
    assert_eq!(dict.lookup("hello").unwrap(), original.as_slice());
}

#[test]
fn test_user_overlay_new_word() {
    let mut dict = PronunciationDict::english();
    assert!(dict.lookup("xyzzy").is_none());

    dict.insert_user(
        "xyzzy",
        &[
            svara::phoneme::Phoneme::FricativeZ,
            svara::phoneme::Phoneme::VowelNearI,
            svara::phoneme::Phoneme::FricativeZ,
            svara::phoneme::Phoneme::VowelE,
        ],
    );
    assert!(dict.lookup("xyzzy").is_some());
    assert_eq!(dict.user_len(), 1);
}

#[test]
fn test_user_overlay_serde_roundtrip() {
    let mut dict = PronunciationDict::english_minimal();
    dict.insert_user("custom", &[svara::phoneme::Phoneme::VowelA]);

    let json = serde_json::to_string(&dict).unwrap();
    let dict2: PronunciationDict = serde_json::from_str(&json).unwrap();

    assert_eq!(
        dict2.lookup("custom").unwrap(),
        &[svara::phoneme::Phoneme::VowelA]
    );
    assert_eq!(dict2.user_len(), 1);
    assert_eq!(dict2.len(), dict.len());
}

// --- Format: CMUdict ---

#[test]
fn test_cmudict_parse_roundtrip() {
    let input = ";;; test dict\nhello  HH AH0 L OW1\nworld  W ER1 L D\n";
    let dict = format::parse_cmudict(input).unwrap();
    assert_eq!(dict.len(), 2);
    assert!(dict.lookup("hello").is_some());
    assert!(dict.lookup("world").is_some());
}

#[test]
fn test_cmudict_export() {
    let mut dict = PronunciationDict::new();
    dict.insert(
        "cat",
        &[
            svara::phoneme::Phoneme::PlosiveK,
            svara::phoneme::Phoneme::VowelAsh,
            svara::phoneme::Phoneme::PlosiveT,
        ],
    );
    let output = format::to_cmudict(&dict);
    assert!(output.contains("cat  K AE1 T"));
}

#[test]
fn test_cmudict_parse_error_missing_separator() {
    let input = "badline\n";
    let result = format::parse_cmudict(input);
    assert!(result.is_err());
}

#[test]
fn test_cmudict_parse_error_unknown_symbol() {
    let input = "word  XX1\n";
    let result = format::parse_cmudict(input);
    assert!(result.is_err());
}

// --- Variant pronunciations ---

#[test]
fn test_heteronym_read_has_variants() {
    let dict = PronunciationDict::english();
    let all = dict.lookup_all("read");
    assert!(all.is_some(), "read should be in dictionary");
    let pronunciations = all.unwrap();
    assert!(
        pronunciations.len() >= 2,
        "read should have 2+ variants, got {}",
        pronunciations.len()
    );
}

#[test]
fn test_heteronym_primary_is_highest_frequency() {
    let dict = PronunciationDict::english();
    let entry = dict.lookup_entry("read").unwrap();
    let primary = entry.primary();
    // Primary should have the highest frequency
    for pron in entry.all() {
        if let (Some(primary_freq), Some(other_freq)) = (primary.frequency(), pron.frequency()) {
            assert!(
                primary_freq >= other_freq,
                "primary should have highest frequency"
            );
        }
    }
}

#[test]
fn test_lookup_returns_primary_phonemes() {
    let dict = PronunciationDict::english();
    let primary = dict.lookup("read").unwrap();
    let entry = dict.lookup_entry("read").unwrap();
    assert_eq!(primary, entry.primary_phonemes());
}

#[test]
fn test_heteronym_wind_has_variants() {
    let dict = PronunciationDict::english();
    let all = dict.lookup_all("wind").unwrap();
    assert!(all.len() >= 2, "wind should have 2+ variants");
}

#[test]
fn test_lookup_entry_new_method() {
    let dict = PronunciationDict::english();
    let entry = dict.lookup_entry("the");
    assert!(entry.is_some());
    assert_eq!(entry.unwrap().len(), 1); // "the" has only one pronunciation
}

// --- Format: variants ---

#[test]
fn test_cmudict_parse_with_variants() {
    let input = "\
;;; @freq=0.7
read  R IY1 D
;;; @freq=0.3
read(2)  R EH1 D
";
    let dict = format::parse_cmudict(input).unwrap();
    assert_eq!(dict.len(), 1); // one word
    let all = dict.lookup_all("read").unwrap();
    assert_eq!(all.len(), 2);
    // Higher frequency should be primary (0.7 > 0.3)
    assert_eq!(all[0].frequency(), Some(0.7));
}

#[test]
fn test_cmudict_export_with_variants() {
    use shabdakosh::{DictEntry, Pronunciation};

    let mut dict = PronunciationDict::new();
    let entry = DictEntry::from_pronunciations(vec![
        Pronunciation::new(vec![
            svara::phoneme::Phoneme::ApproximantR,
            svara::phoneme::Phoneme::VowelE,
            svara::phoneme::Phoneme::PlosiveD,
        ])
        .with_frequency(0.7),
        Pronunciation::new(vec![
            svara::phoneme::Phoneme::ApproximantR,
            svara::phoneme::Phoneme::VowelOpenE,
            svara::phoneme::Phoneme::PlosiveD,
        ])
        .with_frequency(0.3),
    ])
    .unwrap();
    dict.insert_entry("read", entry);

    let output = format::to_cmudict(&dict);
    assert!(output.contains("read  "), "should have primary");
    assert!(output.contains("read(2)  "), "should have variant");
    assert!(
        output.contains("@freq="),
        "should have frequency annotation"
    );
}

// --- IPA format ---

#[test]
fn test_ipa_parse_roundtrip() {
    let input = "hello /hɛloʊ/\nworld /wɜld/\n";
    let dict = format::parse_ipa(input).unwrap();
    assert_eq!(dict.len(), 2);
    assert!(dict.lookup("hello").is_some());
    assert!(dict.lookup("world").is_some());
}

#[test]
fn test_ipa_export() {
    let mut dict = PronunciationDict::new();
    dict.insert(
        "cat",
        &[
            svara::phoneme::Phoneme::PlosiveK,
            svara::phoneme::Phoneme::VowelAsh,
            svara::phoneme::Phoneme::PlosiveT,
        ],
    );
    let output = format::to_ipa(&dict);
    assert!(output.contains("cat /kæt/"));
}

#[test]
fn test_ipa_parse_error_empty_transcription() {
    let input = "word\n";
    let result = format::parse_ipa(input);
    assert!(result.is_err());
}

// --- Serde roundtrip ---

#[test]
fn test_serde_roundtrip_error() {
    let err = shabdakosh::ShabdakoshError::DictParseError("test parse error".into());
    let json = serde_json::to_string(&err).unwrap();
    let e2: shabdakosh::ShabdakoshError = serde_json::from_str(&json).unwrap();
    assert_eq!(err.to_string(), e2.to_string());
}

#[test]
fn test_serde_roundtrip_dict_with_variants() {
    use shabdakosh::{DictEntry, Pronunciation};

    let mut dict = PronunciationDict::english_minimal();
    let entry = DictEntry::from_pronunciations(vec![
        Pronunciation::new(vec![svara::phoneme::Phoneme::VowelA]).with_frequency(0.6),
        Pronunciation::new(vec![svara::phoneme::Phoneme::VowelE]).with_frequency(0.4),
    ])
    .unwrap();
    dict.insert_entry("test", entry);

    let json = serde_json::to_string(&dict).unwrap();
    let dict2: PronunciationDict = serde_json::from_str(&json).unwrap();

    let all = dict2.lookup_all("test").unwrap();
    assert_eq!(all.len(), 2);
}

// --- Merge ---

#[test]
fn test_merge_override() {
    let mut base = PronunciationDict::new();
    base.insert("cat", &[svara::phoneme::Phoneme::PlosiveK]);

    let mut other = PronunciationDict::new();
    other.insert("cat", &[svara::phoneme::Phoneme::PlosiveT]);
    other.insert("dog", &[svara::phoneme::Phoneme::PlosiveD]);

    base.merge(&other);
    // "cat" overridden, "dog" added
    assert_eq!(
        base.lookup("cat").unwrap(),
        &[svara::phoneme::Phoneme::PlosiveT]
    );
    assert!(base.lookup("dog").is_some());
}

#[test]
fn test_merge_conservative() {
    let mut base = PronunciationDict::new();
    base.insert("cat", &[svara::phoneme::Phoneme::PlosiveK]);

    let mut other = PronunciationDict::new();
    other.insert("cat", &[svara::phoneme::Phoneme::PlosiveT]);
    other.insert("dog", &[svara::phoneme::Phoneme::PlosiveD]);

    base.merge_conservative(&other);
    // "cat" kept, "dog" added
    assert_eq!(
        base.lookup("cat").unwrap(),
        &[svara::phoneme::Phoneme::PlosiveK]
    );
    assert!(base.lookup("dog").is_some());
}

// --- Diff ---

#[test]
fn test_diff_identical() {
    let dict = PronunciationDict::english_minimal();
    let d = dictionary::diff(&dict, &dict);
    assert!(d.is_empty());
    assert_eq!(d.len(), 0);
}

#[test]
fn test_diff_changes() {
    let mut left = PronunciationDict::new();
    left.insert("cat", &[svara::phoneme::Phoneme::PlosiveK]);
    left.insert("dog", &[svara::phoneme::Phoneme::PlosiveD]);

    let mut right = PronunciationDict::new();
    right.insert("cat", &[svara::phoneme::Phoneme::PlosiveT]); // changed
    right.insert("fish", &[svara::phoneme::Phoneme::FricativeF]); // added

    let d = dictionary::diff(&left, &right);
    assert_eq!(d.added, vec!["fish"]);
    assert_eq!(d.removed, vec!["dog"]);
    assert_eq!(d.changed, vec!["cat"]);
    assert_eq!(d.len(), 3);
}

#[test]
fn test_serde_roundtrip_dict_diff() {
    let d = DictDiff {
        added: vec!["new".into()],
        removed: vec!["old".into()],
        changed: vec!["mod".into()],
    };
    let json = serde_json::to_string(&d).unwrap();
    let d2: DictDiff = serde_json::from_str(&json).unwrap();
    assert_eq!(d, d2);
}

// --- PLS ---

#[test]
fn test_pls_integration_roundtrip() {
    use format::pls;

    let mut dict = PronunciationDict::new();
    dict.insert(
        "hello",
        &[
            svara::phoneme::Phoneme::FricativeH,
            svara::phoneme::Phoneme::VowelOpenE,
            svara::phoneme::Phoneme::LateralL,
            svara::phoneme::Phoneme::DiphthongOU,
        ],
    );
    let xml = pls::to_pls(&dict, "en-US");
    assert!(xml.contains("<lexicon"));
    assert!(xml.contains("<grapheme>hello</grapheme>"));

    let dict2 = pls::parse_pls(&xml).unwrap();
    assert!(dict2.lookup("hello").is_some());
}

// --- SSML ---

#[test]
fn test_ssml_integration_roundtrip() {
    use format::ssml;

    let phonemes = [
        svara::phoneme::Phoneme::PlosiveK,
        svara::phoneme::Phoneme::VowelAsh,
        svara::phoneme::Phoneme::PlosiveT,
    ];
    let tag = ssml::to_ssml_phoneme("cat", &phonemes);
    assert!(tag.contains("ph="));
    assert!(tag.contains(">cat</phoneme>"));

    let (word, parsed) = ssml::parse_ssml_phoneme(&tag).unwrap();
    assert_eq!(word, "cat");
    assert_eq!(parsed, phonemes);
}

// --- Language field tests ---

#[test]
fn test_language_field_english() {
    let dict = PronunciationDict::english();
    assert_eq!(dict.language(), Some("en"));
}

#[test]
fn test_language_field_english_minimal() {
    let dict = PronunciationDict::english_minimal();
    assert_eq!(dict.language(), Some("en"));
}

#[test]
fn test_language_field_new() {
    let dict = PronunciationDict::new();
    assert_eq!(dict.language(), None);
}

#[test]
fn test_language_field_with_language() {
    let dict = PronunciationDict::new().with_language("fr");
    assert_eq!(dict.language(), Some("fr"));
}

#[test]
fn test_language_field_set_language() {
    let mut dict = PronunciationDict::new();
    dict.set_language("de");
    assert_eq!(dict.language(), Some("de"));
}

#[test]
fn test_language_field_serde_roundtrip() {
    let dict = PronunciationDict::english_minimal();
    let json = serde_json::to_string(&dict).unwrap();
    let dict2: PronunciationDict = serde_json::from_str(&json).unwrap();
    assert_eq!(dict2.language(), Some("en"));
}

#[test]
fn test_language_field_serde_absent() {
    // Simulate a v1.0.0 serialized dict (no language field).
    let json = r#"{"entries":{}}"#;
    let dict: PronunciationDict = serde_json::from_str(json).unwrap();
    assert_eq!(dict.language(), None);
}

// --- Varna integration tests ---

#[cfg(feature = "varna")]
mod varna_tests {
    use shabdakosh::PronunciationDict;
    use shabdakosh::dictionary::detect;
    use shabdakosh::dictionary::validate;

    #[test]
    fn test_from_lexicon_spanish() {
        let lexicon = varna::lexicon::swadesh::by_code("es").unwrap();
        let dict = PronunciationDict::from_lexicon(&lexicon);
        assert!(!dict.is_empty());
        assert_eq!(dict.language(), Some("es"));
    }

    #[test]
    fn test_from_lexicon_hindi() {
        let lexicon = varna::lexicon::swadesh::by_code("hi").unwrap();
        let dict = PronunciationDict::from_lexicon(&lexicon);
        assert!(!dict.is_empty());
        assert_eq!(dict.language(), Some("hi"));
    }

    #[test]
    fn test_from_lexicon_german() {
        let lexicon = varna::lexicon::swadesh::by_code("de").unwrap();
        let dict = PronunciationDict::from_lexicon(&lexicon);
        assert!(!dict.is_empty());
        assert_eq!(dict.language(), Some("de"));
    }

    #[test]
    fn test_from_lexicon_lookup() {
        let lexicon = varna::lexicon::swadesh::by_code("es").unwrap();
        let dict = PronunciationDict::from_lexicon(&lexicon);
        // "agua" (water) should be in the Spanish Swadesh list.
        assert!(
            dict.lookup("agua").is_some(),
            "expected 'agua' in Spanish seed dict"
        );
    }

    #[test]
    fn test_spanish_seed() {
        let dict = PronunciationDict::spanish();
        assert!(!dict.is_empty());
        assert_eq!(dict.language(), Some("es"));
    }

    #[test]
    fn test_hindi_seed() {
        let dict = PronunciationDict::hindi();
        assert!(!dict.is_empty());
        assert_eq!(dict.language(), Some("hi"));
    }

    #[test]
    fn test_german_seed() {
        let dict = PronunciationDict::german();
        assert!(!dict.is_empty());
        assert_eq!(dict.language(), Some("de"));
    }

    #[test]
    fn test_sanskrit_seed() {
        let dict = PronunciationDict::sanskrit();
        assert_eq!(dict.len(), 0);
        assert_eq!(dict.language(), Some("sa"));
    }

    #[test]
    fn test_validate_consonants_pass() {
        let mut dict = PronunciationDict::new();
        dict.insert(
            "test",
            &[
                svara::phoneme::Phoneme::PlosiveP,
                svara::phoneme::Phoneme::VowelAsh,
                svara::phoneme::Phoneme::PlosiveT,
            ],
        );
        let inventory = varna::phoneme::english();
        let report = validate::validate_inventory(&dict, &inventory);
        assert!(report.is_valid());
    }

    #[test]
    fn test_validate_convenience_method() {
        let mut dict = PronunciationDict::new().with_language("en");
        dict.insert(
            "pat",
            &[
                svara::phoneme::Phoneme::PlosiveP,
                svara::phoneme::Phoneme::VowelAsh,
                svara::phoneme::Phoneme::PlosiveT,
            ],
        );
        let report = dict.validate().unwrap();
        assert!(report.is_valid());
    }

    #[test]
    fn test_validate_no_language() {
        let dict = PronunciationDict::new();
        assert!(dict.validate().is_none());
    }

    #[test]
    fn test_detect_script_latin() {
        assert_eq!(detect::detect_script("hello"), Some("Latn"));
    }

    #[test]
    fn test_detect_script_devanagari() {
        assert_eq!(detect::detect_script("नमस्ते"), Some("Deva"));
    }

    #[test]
    fn test_detect_language_hint_latin() {
        let hints = detect::detect_language_hint("hello");
        assert!(hints.contains(&"en"));
        assert!(hints.contains(&"es"));
    }

    #[test]
    fn test_detect_language_hint_devanagari() {
        let hints = detect::detect_language_hint("नमस्ते");
        assert!(hints.contains(&"hi"));
        assert!(hints.contains(&"sa"));
    }

    #[test]
    fn test_validation_report_serde_roundtrip() {
        let report = validate::ValidationReport::new("en".into(), vec![]);
        let json = serde_json::to_string(&report).unwrap();
        let rt: validate::ValidationReport = serde_json::from_str(&json).unwrap();
        assert_eq!(report, rt);
    }

    #[test]
    fn test_from_lexicon_serde_roundtrip() {
        let lexicon = varna::lexicon::swadesh::by_code("es").unwrap();
        let dict = PronunciationDict::from_lexicon(&lexicon);
        let json = serde_json::to_string(&dict).unwrap();
        let dict2: PronunciationDict = serde_json::from_str(&json).unwrap();
        assert_eq!(dict.language(), dict2.language());
        assert_eq!(dict.len(), dict2.len());
    }
}
