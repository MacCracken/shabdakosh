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
pub struct InvalidEntry {
    /// The word with invalid phonemes.
    pub word: String,
    /// Phonemes not found in the target language's inventory.
    pub invalid_phonemes: Vec<Phoneme>,
}

/// Result of validating a dictionary against a phoneme inventory.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidationReport {
    /// The language code of the inventory used for validation.
    pub language: String,
    /// Entries containing phonemes outside the inventory.
    pub invalid_entries: Vec<InvalidEntry>,
}

impl ValidationReport {
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
}
