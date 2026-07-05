//! Validation of dictionary entries against varna phoneme inventories.
//!
//! When the `varna` feature is enabled, dictionary entries can be validated
//! against a language's phoneme inventory to catch transcription errors —
//! phonemes that don't exist in the target language.

use alloc::{string::String, vec::Vec};
use serde::{Deserialize, Serialize};
use svara::phoneme::Phoneme;

use crate::ipa::phoneme_to_ipa;

/// A dictionary entry that contains phonemes not in the target inventory.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct InvalidEntry {
    /// The word with invalid phonemes.
    pub word: String,
    /// Phonemes not found in the target language's inventory.
    pub invalid_phonemes: Vec<Phoneme>,
}

impl InvalidEntry {
    /// Creates a new invalid entry record.
    #[must_use]
    pub fn new(word: String, invalid_phonemes: Vec<Phoneme>) -> Self {
        Self {
            word,
            invalid_phonemes,
        }
    }
}

/// Result of validating a dictionary against a phoneme inventory.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ValidationReport {
    /// The language code of the inventory used for validation.
    pub language: String,
    /// Entries containing phonemes outside the inventory.
    pub invalid_entries: Vec<InvalidEntry>,
}

impl ValidationReport {
    /// Creates a new validation report.
    #[must_use]
    pub fn new(language: String, invalid_entries: Vec<InvalidEntry>) -> Self {
        Self {
            language,
            invalid_entries,
        }
    }

    /// Returns `true` if all entries passed validation.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.invalid_entries.is_empty()
    }

    /// Returns the number of entries with invalid phonemes.
    #[must_use]
    pub fn invalid_count(&self) -> usize {
        self.invalid_entries.len()
    }
}

/// Validates all entries in a dictionary against a varna phoneme inventory.
///
/// For each entry, every phoneme is converted to IPA and checked against
/// the inventory. Phonemes that don't exist in the inventory are collected.
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "varna")]
/// # {
/// use shabdakosh::PronunciationDict;
/// use shabdakosh::dictionary::validate::validate_inventory;
/// use svara::phoneme::Phoneme;
///
/// let mut dict = PronunciationDict::new();
/// dict.insert("pat", &[Phoneme::PlosiveP, Phoneme::VowelAsh, Phoneme::PlosiveT]);
/// let inventory = varna::phoneme::english();
/// let report = validate_inventory(&dict, &inventory);
/// assert!(report.is_valid());
/// # }
/// ```
#[must_use]
pub fn validate_inventory(
    dict: &super::PronunciationDict,
    inventory: &varna::phoneme::PhonemeInventory,
) -> ValidationReport {
    let mut invalid_entries = Vec::new();

    // Check base entries.
    for (word, entry) in dict.entries() {
        if let Some(invalid) = check_entry(word, entry, inventory) {
            invalid_entries.push(invalid);
        }
    }

    // Check user overlay entries.
    for (word, entry) in dict.user_entries() {
        if let Some(invalid) = check_entry(word, entry, inventory) {
            invalid_entries.push(invalid);
        }
    }

    invalid_entries.sort_by(|a, b| a.word.cmp(&b.word));

    ValidationReport {
        language: alloc::string::ToString::to_string(&inventory.language_code),
        invalid_entries,
    }
}

/// Checks whether an IPA symbol is in the inventory, accounting for
/// length mark (ː) differences between transcription conventions.
fn inventory_has_normalized(inventory: &varna::phoneme::PhonemeInventory, ipa: &str) -> bool {
    // Exact match.
    if inventory.has(ipa) {
        return true;
    }
    // Try adding a length mark (e.g., "ɔ" → "ɔː").
    let with_long = alloc::format!("{ipa}ː");
    if inventory.has(&with_long) {
        return true;
    }
    // Try stripping a length mark (e.g., "iː" → "i").
    if let Some(base) = ipa.strip_suffix('ː')
        && inventory.has(base)
    {
        return true;
    }
    false
}

/// Check a single entry's phonemes against an inventory.
fn check_entry(
    word: &str,
    entry: &super::entry::DictEntry,
    inventory: &varna::phoneme::PhonemeInventory,
) -> Option<InvalidEntry> {
    let mut invalid_phonemes = Vec::new();

    for pronunciation in entry.all() {
        for phoneme in pronunciation.phonemes() {
            if let Some(ipa) = phoneme_to_ipa(phoneme)
                && !inventory_has_normalized(inventory, ipa)
                && !invalid_phonemes.contains(phoneme)
            {
                invalid_phonemes.push(*phoneme);
            }
            // Phonemes without IPA mapping (e.g., Silence) are skipped —
            // they're not language-specific.
        }
    }

    if invalid_phonemes.is_empty() {
        None
    } else {
        Some(InvalidEntry {
            word: alloc::string::ToString::to_string(word),
            invalid_phonemes,
        })
    }
}

// --- Phonotactic validation ---

/// A phonotactic violation found in a dictionary entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PhonotacticViolation {
    /// The word containing the violation.
    pub word: String,
    /// Description of the violation.
    pub description: String,
    /// The IPA sequence that violated phonotactic constraints.
    pub sequence: String,
}

/// Result of phonotactic validation across a dictionary.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PhonotacticReport {
    /// The language code of the phonotactic profile used.
    pub language: String,
    /// All violations found.
    pub violations: Vec<PhonotacticViolation>,
}

impl PhonotacticReport {
    /// Returns `true` if no phonotactic violations were found.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.violations.is_empty()
    }

    /// Returns the number of violations.
    #[must_use]
    pub fn violation_count(&self) -> usize {
        self.violations.len()
    }
}

/// Validates dictionary entries against phonotactic constraints.
///
/// Checks each pronunciation for forbidden phoneme sequences as defined
/// by the varna [`Phonotactics`](varna::phoneme::syllable::Phonotactics) profile.
///
/// # Examples
///
/// ```rust
/// # #[cfg(feature = "varna")]
/// # {
/// use shabdakosh::PronunciationDict;
/// use shabdakosh::dictionary::validate::validate_phonotactics;
/// use varna::phoneme::syllable::english_phonotactics;
///
/// let dict = PronunciationDict::english();
/// let phonotactics = english_phonotactics();
/// let report = validate_phonotactics(&dict, &phonotactics);
/// // Most entries should pass phonotactic constraints.
/// println!("violations: {}", report.violation_count());
/// # }
/// ```
#[must_use]
pub fn validate_phonotactics(
    dict: &super::PronunciationDict,
    phonotactics: &varna::phoneme::syllable::Phonotactics,
) -> PhonotacticReport {
    let mut violations = Vec::new();

    // Check both base and user entries.
    for (word, entry) in dict.entries().iter().chain(dict.user_entries().iter()) {
        for pron in entry.all() {
            check_pronunciation(word, pron.phonemes(), phonotactics, &mut violations);
        }
    }

    violations.sort_by(|a, b| a.word.cmp(&b.word));

    PhonotacticReport {
        language: alloc::string::ToString::to_string(&phonotactics.language_code),
        violations,
    }
}

/// Checks a single pronunciation against phonotactic constraints.
fn check_pronunciation(
    word: &str,
    phonemes: &[Phoneme],
    phonotactics: &varna::phoneme::syllable::Phonotactics,
    violations: &mut Vec<PhonotacticViolation>,
) {
    use varna::phoneme::syllable::SyllablePosition;

    // Check consecutive consonant pairs against onset/coda constraints.
    // This is a simplified check — full syllabification would be more accurate
    // but is a v2.0 feature.
    for window in phonemes.windows(2) {
        let ipa_a = crate::ipa::phoneme_to_ipa(&window[0]);
        let ipa_b = crate::ipa::phoneme_to_ipa(&window[1]);

        if let (Some(a), Some(b)) = (ipa_a, ipa_b) {
            let sequence = alloc::format!("{a}{b}");

            // Check if this sequence is explicitly forbidden in any syllable position.
            // Without full syllabification (a v2.0 feature), we check both onset
            // and coda and report a single violation if forbidden in either.
            let forbidden_onset =
                phonotactics.is_permitted(&sequence, SyllablePosition::Onset) == Some(false);
            let forbidden_coda =
                phonotactics.is_permitted(&sequence, SyllablePosition::Coda) == Some(false);

            if forbidden_onset || forbidden_coda {
                let position = match (forbidden_onset, forbidden_coda) {
                    (true, true) => "onset/coda",
                    (true, false) => "onset",
                    (false, true) => "coda",
                    _ => unreachable!(),
                };
                violations.push(PhonotacticViolation {
                    word: alloc::string::ToString::to_string(word),
                    description: alloc::format!("forbidden {position} sequence '{sequence}'"),
                    sequence,
                });
            }
        }
    }

    // Check max onset/coda cluster length.
    // Count consecutive consonants (simplified: any non-vowel phoneme).
    let max_onset = phonotactics.syllable.max_onset as usize;
    let max_coda = phonotactics.syllable.max_coda as usize;

    let mut consonant_run = 0_usize;
    for phoneme in phonemes {
        if is_consonant(phoneme) {
            consonant_run += 1;
            // Use max of onset + coda as a generous upper bound.
            // Without syllabification, we can't distinguish onset from coda.
            if consonant_run > max_onset + max_coda {
                violations.push(PhonotacticViolation {
                    word: alloc::string::ToString::to_string(word),
                    description: alloc::format!(
                        "consonant cluster of {consonant_run} exceeds max onset ({max_onset}) + coda ({max_coda})"
                    ),
                    sequence: alloc::format!("({consonant_run} consonants)"),
                });
                break;
            }
        } else {
            consonant_run = 0;
        }
    }
}

/// Simple consonant classification based on phoneme variant.
fn is_consonant(phoneme: &Phoneme) -> bool {
    !matches!(
        phoneme,
        Phoneme::VowelA
            | Phoneme::VowelE
            | Phoneme::VowelI
            | Phoneme::VowelO
            | Phoneme::VowelU
            | Phoneme::VowelOpenA
            | Phoneme::VowelOpenE
            | Phoneme::VowelOpenO
            | Phoneme::VowelNearI
            | Phoneme::VowelNearU
            | Phoneme::VowelAsh
            | Phoneme::VowelSchwa
            | Phoneme::VowelCupV
            | Phoneme::VowelBird
            | Phoneme::DiphthongAI
            | Phoneme::DiphthongAU
            | Phoneme::DiphthongEI
            | Phoneme::DiphthongOI
            | Phoneme::DiphthongOU
            | Phoneme::Silence
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PronunciationDict;
    use crate::dictionary::entry::DictEntry;

    #[test]
    fn test_length_normalization() {
        // "ɔ" should match varna's "ɔː" via normalization.
        let mut dict = PronunciationDict::new();
        dict.insert("caught", &[Phoneme::VowelOpenO]);
        let inventory = varna::phoneme::english();
        let report = validate_inventory(&dict, &inventory);
        assert!(report.is_valid(), "length normalization failed: {report:?}");
    }

    #[test]
    fn test_consonants_validate_english() {
        // All English consonants should validate.
        let mut dict = PronunciationDict::new();
        dict.insert(
            "test",
            &[
                Phoneme::PlosiveP,
                Phoneme::PlosiveB,
                Phoneme::PlosiveT,
                Phoneme::PlosiveD,
                Phoneme::PlosiveK,
                Phoneme::PlosiveG,
                Phoneme::FricativeF,
                Phoneme::FricativeV,
                Phoneme::FricativeS,
                Phoneme::FricativeZ,
                Phoneme::FricativeSh,
                Phoneme::FricativeZh,
                Phoneme::FricativeTh,
                Phoneme::FricativeDh,
                Phoneme::FricativeH,
                Phoneme::NasalM,
                Phoneme::NasalN,
                Phoneme::NasalNg,
                Phoneme::LateralL,
                Phoneme::ApproximantW,
                Phoneme::ApproximantJ,
            ],
        );
        let inventory = varna::phoneme::english();
        let report = validate_inventory(&dict, &inventory);
        assert!(report.is_valid(), "consonant validation failed: {report:?}");
    }

    #[test]
    fn test_invalid_phoneme_detected() {
        // ŋ (velar nasal) is in English but not in Spanish inventory.
        let mut dict = PronunciationDict::new();
        dict.insert("sing", &[Phoneme::NasalNg]);
        let inventory = varna::phoneme::inventories::spanish();
        let report = validate_inventory(&dict, &inventory);
        assert!(!report.is_valid());
        assert_eq!(report.invalid_count(), 1);
        assert_eq!(report.invalid_entries[0].word, "sing");
        assert!(
            report.invalid_entries[0]
                .invalid_phonemes
                .contains(&Phoneme::NasalNg)
        );
    }

    #[test]
    fn test_empty_dict_is_valid() {
        let dict = PronunciationDict::new();
        let inventory = varna::phoneme::english();
        let report = validate_inventory(&dict, &inventory);
        assert!(report.is_valid());
        assert_eq!(report.invalid_count(), 0);
    }

    #[test]
    fn test_user_overlay_validated() {
        // ŋ is not in Spanish inventory.
        let mut dict = PronunciationDict::new();
        dict.insert_user("test", &[Phoneme::NasalNg]);
        let inventory = varna::phoneme::inventories::spanish();
        let report = validate_inventory(&dict, &inventory);
        assert!(!report.is_valid());
    }

    #[test]
    fn test_validation_report_serde_roundtrip() {
        let report = ValidationReport {
            language: alloc::string::ToString::to_string("es"),
            invalid_entries: alloc::vec![InvalidEntry {
                word: alloc::string::ToString::to_string("sing"),
                invalid_phonemes: alloc::vec![Phoneme::NasalNg],
            }],
        };
        let json = serde_json::to_string(&report).unwrap();
        let roundtripped: ValidationReport = serde_json::from_str(&json).unwrap();
        assert_eq!(report, roundtripped);
    }

    #[test]
    fn test_validate_method_with_language() {
        // Use a dict with only phonemes known to be in English inventory.
        let mut dict = PronunciationDict::new().with_language("en");
        dict.insert(
            "pat",
            &[Phoneme::PlosiveP, Phoneme::VowelAsh, Phoneme::PlosiveT],
        );
        let report = dict.validate().unwrap();
        assert!(report.is_valid(), "validation failed: {report:?}");
    }

    #[test]
    fn test_validate_method_without_language() {
        let dict = PronunciationDict::new();
        assert!(dict.validate().is_none());
    }

    #[test]
    fn test_multiple_pronunciations_checked() {
        use crate::dictionary::entry::Pronunciation;
        let mut dict = PronunciationDict::new();
        let entry = DictEntry::from_pronunciations(alloc::vec![
            Pronunciation::new(alloc::vec![Phoneme::NasalNg]),
            Pronunciation::new(alloc::vec![Phoneme::PlosiveP]),
        ])
        .unwrap();
        dict.insert_entry("test", entry);
        let inventory = varna::phoneme::inventories::spanish();
        let report = validate_inventory(&dict, &inventory);
        assert_eq!(report.invalid_count(), 1);
        // Only ŋ should be invalid, p is valid in Spanish.
        assert_eq!(report.invalid_entries[0].invalid_phonemes.len(), 1);
    }

    // --- Phonotactic validation tests ---

    #[test]
    fn test_phonotactic_report_serde_roundtrip() {
        let report = PhonotacticReport {
            language: alloc::string::ToString::to_string("en"),
            violations: alloc::vec![PhonotacticViolation {
                word: alloc::string::ToString::to_string("test"),
                description: alloc::string::ToString::to_string("forbidden onset"),
                sequence: alloc::string::ToString::to_string("tl"),
            }],
        };
        let json = serde_json::to_string(&report).unwrap();
        let roundtripped: PhonotacticReport = serde_json::from_str(&json).unwrap();
        assert_eq!(report, roundtripped);
    }

    #[test]
    fn test_phonotactic_empty_dict() {
        let dict = PronunciationDict::new();
        let phonotactics = varna::phoneme::syllable::english_phonotactics();
        let report = validate_phonotactics(&dict, &phonotactics);
        assert!(report.is_valid());
    }

    #[test]
    fn test_phonotactic_valid_english() {
        // "cat" = /k æ t/ — should have no violations.
        let mut dict = PronunciationDict::new();
        dict.insert(
            "cat",
            &[Phoneme::PlosiveK, Phoneme::VowelAsh, Phoneme::PlosiveT],
        );
        let phonotactics = varna::phoneme::syllable::english_phonotactics();
        let report = validate_phonotactics(&dict, &phonotactics);
        assert!(
            report.is_valid(),
            "cat should be phonotactically valid, but got violations: {:?}",
            report.violations
        );
    }

    #[test]
    fn test_phonotactic_convenience_method() {
        let mut dict = PronunciationDict::new().with_language("en");
        dict.insert(
            "cat",
            &[Phoneme::PlosiveK, Phoneme::VowelAsh, Phoneme::PlosiveT],
        );
        let report = dict.validate_phonotactics().unwrap();
        assert!(report.is_valid());
    }

    #[test]
    fn test_phonotactic_no_language() {
        let dict = PronunciationDict::new();
        assert!(dict.validate_phonotactics().is_none());
    }

    #[test]
    fn test_phonotactic_unknown_language() {
        let dict = PronunciationDict::new().with_language("xx");
        assert!(dict.validate_phonotactics().is_none());
    }

    #[test]
    fn test_is_consonant() {
        assert!(is_consonant(&Phoneme::PlosiveK));
        assert!(is_consonant(&Phoneme::FricativeS));
        assert!(is_consonant(&Phoneme::NasalM));
        assert!(!is_consonant(&Phoneme::VowelA));
        assert!(!is_consonant(&Phoneme::DiphthongAI));
        assert!(!is_consonant(&Phoneme::VowelSchwa));
    }
}
