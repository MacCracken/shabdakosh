//! Integration tests for shabdakosh.

use shabdakosh::dictionary::format;
use shabdakosh::PronunciationDict;

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
        "people", "because", "through", "enough", "beautiful", "colonel", "psychology",
        "knight", "thought", "language", "world", "hello", "the", "computer", "science",
        "music", "water", "friend", "school", "house",
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
    assert!(output.contains("@freq="), "should have frequency annotation");
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
