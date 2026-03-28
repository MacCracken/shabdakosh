//! Pronunciation dictionary for common/irregular words.
//!
//! English has many irregular pronunciations (e.g., "one" → /wʌn/, "colonel" → /kɜːnəl/).
//! The dictionary provides known-correct phoneme sequences for these words.
//!
//! The full English dictionary is generated at compile time from `data/cmudict-5k.txt`
//! by the build script. A minimal 28-entry variant is available via [`PronunciationDict::english_minimal`].
//!
//! ## User overlay
//!
//! Application-specific pronunciations can be added via [`PronunciationDict::insert_user`].
//! User entries take precedence over base entries during [`PronunciationDict::lookup`].
//!
//! ## Variant pronunciations
//!
//! Words with multiple pronunciations (heteronyms like "read", "live", "wind") are
//! represented as [`DictEntry`] values containing multiple [`Pronunciation`] variants.
//! Use [`PronunciationDict::lookup_entry`] or [`PronunciationDict::lookup_all`]
//! to access all variants.

pub mod entry;
pub mod format;

use alloc::{collections::BTreeMap, string::String, vec::Vec};
use hashbrown::HashMap;
use serde::{Deserialize, Serialize};
use svara::phoneme::Phoneme;

use entry::{DictEntry, Pronunciation};

// Pull in the generated dictionary function from build.rs output.
include!(concat!(env!("OUT_DIR"), "/generated_dict.rs"));

/// A pronunciation dictionary mapping words to phoneme sequences.
///
/// Supports a two-layer lookup: user entries (overlay) take precedence over
/// base entries. This allows applications to override or extend the built-in
/// dictionary without modifying it.
///
/// Each word maps to a [`DictEntry`] containing one or more [`Pronunciation`]
/// variants with optional frequency and region metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PronunciationDict {
    #[serde(deserialize_with = "deserialize_entries_compat")]
    entries: HashMap<String, DictEntry>,
    #[serde(
        default,
        skip_serializing_if = "BTreeMap::is_empty",
        deserialize_with = "deserialize_user_entries_compat"
    )]
    user_entries: BTreeMap<String, DictEntry>,
}

impl PronunciationDict {
    /// Creates a new empty dictionary.
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            user_entries: BTreeMap::new(),
        }
    }

    /// Creates the built-in English pronunciation dictionary (10,000+ entries).
    ///
    /// Generated at compile time from `data/cmudict-5k.txt`.
    #[must_use]
    pub fn english() -> Self {
        Self {
            entries: generated_english_entries(),
            user_entries: BTreeMap::new(),
        }
    }

    /// Creates a minimal English dictionary with ~29 common function words.
    ///
    /// Useful for testing or memory-constrained environments.
    #[must_use]
    pub fn english_minimal() -> Self {
        let mut dict = Self::new();

        dict.insert("the", &[Phoneme::FricativeDh, Phoneme::VowelSchwa]);
        dict.insert("a", &[Phoneme::VowelSchwa]);
        dict.insert("an", &[Phoneme::VowelSchwa, Phoneme::NasalN]);
        dict.insert("i", &[Phoneme::DiphthongAI]);
        dict.insert("is", &[Phoneme::VowelNearI, Phoneme::FricativeZ]);
        dict.insert(
            "was",
            &[
                Phoneme::ApproximantW,
                Phoneme::VowelOpenO,
                Phoneme::FricativeZ,
            ],
        );
        dict.insert("are", &[Phoneme::VowelOpenA, Phoneme::ApproximantR]);
        dict.insert("to", &[Phoneme::PlosiveT, Phoneme::VowelU]);
        dict.insert("of", &[Phoneme::VowelOpenO, Phoneme::FricativeV]);
        dict.insert("in", &[Phoneme::VowelNearI, Phoneme::NasalN]);
        dict.insert("it", &[Phoneme::VowelNearI, Phoneme::PlosiveT]);
        dict.insert(
            "and",
            &[Phoneme::VowelAsh, Phoneme::NasalN, Phoneme::PlosiveD],
        );
        dict.insert(
            "that",
            &[Phoneme::FricativeDh, Phoneme::VowelAsh, Phoneme::PlosiveT],
        );
        dict.insert(
            "for",
            &[
                Phoneme::FricativeF,
                Phoneme::VowelOpenO,
                Phoneme::ApproximantR,
            ],
        );
        dict.insert("you", &[Phoneme::ApproximantJ, Phoneme::VowelU]);
        dict.insert("he", &[Phoneme::FricativeH, Phoneme::VowelE]);
        dict.insert("she", &[Phoneme::FricativeSh, Phoneme::VowelE]);
        dict.insert("we", &[Phoneme::ApproximantW, Phoneme::VowelE]);
        dict.insert("they", &[Phoneme::FricativeDh, Phoneme::DiphthongEI]);
        dict.insert(
            "this",
            &[
                Phoneme::FricativeDh,
                Phoneme::VowelNearI,
                Phoneme::FricativeS,
            ],
        );
        dict.insert(
            "with",
            &[
                Phoneme::ApproximantW,
                Phoneme::VowelNearI,
                Phoneme::FricativeTh,
            ],
        );
        dict.insert(
            "not",
            &[Phoneme::NasalN, Phoneme::VowelOpenO, Phoneme::PlosiveT],
        );
        dict.insert(
            "but",
            &[Phoneme::PlosiveB, Phoneme::VowelCupV, Phoneme::PlosiveT],
        );
        dict.insert(
            "have",
            &[Phoneme::FricativeH, Phoneme::VowelAsh, Phoneme::FricativeV],
        );
        dict.insert(
            "one",
            &[Phoneme::ApproximantW, Phoneme::VowelCupV, Phoneme::NasalN],
        );
        dict.insert(
            "hello",
            &[
                Phoneme::FricativeH,
                Phoneme::VowelOpenE,
                Phoneme::LateralL,
                Phoneme::DiphthongOU,
            ],
        );
        dict.insert(
            "world",
            &[
                Phoneme::ApproximantW,
                Phoneme::VowelBird,
                Phoneme::LateralL,
                Phoneme::PlosiveD,
            ],
        );
        dict.insert(
            "yes",
            &[
                Phoneme::ApproximantJ,
                Phoneme::VowelOpenE,
                Phoneme::FricativeS,
            ],
        );
        dict.insert("no", &[Phoneme::NasalN, Phoneme::DiphthongOU]);

        dict
    }

    /// Creates a dictionary from a pre-built entries map.
    #[must_use]
    pub fn from_entries(entries: HashMap<String, DictEntry>) -> Self {
        Self {
            entries,
            user_entries: BTreeMap::new(),
        }
    }

    /// Creates a dictionary from a simple map of word -> phonemes.
    ///
    /// Each entry is wrapped into a single-pronunciation [`DictEntry`].
    #[must_use]
    pub fn from_simple_entries(entries: HashMap<String, Vec<Phoneme>>) -> Self {
        let entries = entries
            .into_iter()
            .map(|(word, phonemes)| (word, DictEntry::from_phonemes(&phonemes)))
            .collect();
        Self {
            entries,
            user_entries: BTreeMap::new(),
        }
    }

    /// Inserts a word into the base dictionary with a single pronunciation.
    pub fn insert(&mut self, word: &str, phonemes: &[Phoneme]) {
        self.entries.insert(
            alloc::string::ToString::to_string(&word.to_lowercase()),
            DictEntry::from_phonemes(phonemes),
        );
    }

    /// Inserts a full [`DictEntry`] into the base dictionary.
    pub fn insert_entry(&mut self, word: &str, entry: DictEntry) {
        self.entries.insert(
            alloc::string::ToString::to_string(&word.to_lowercase()),
            entry,
        );
    }

    /// Inserts a word into the user overlay with a single pronunciation.
    ///
    /// User entries take precedence over base entries during lookup.
    pub fn insert_user(&mut self, word: &str, phonemes: &[Phoneme]) {
        self.user_entries.insert(
            alloc::string::ToString::to_string(&word.to_lowercase()),
            DictEntry::from_phonemes(phonemes),
        );
    }

    /// Inserts a full [`DictEntry`] into the user overlay.
    pub fn insert_user_entry(&mut self, word: &str, entry: DictEntry) {
        self.user_entries.insert(
            alloc::string::ToString::to_string(&word.to_lowercase()),
            entry,
        );
    }

    /// Removes a word from the user overlay.
    ///
    /// Returns `true` if the word was present in the user overlay.
    pub fn remove_user(&mut self, word: &str) -> bool {
        self.user_entries
            .remove(&alloc::string::ToString::to_string(&word.to_lowercase()))
            .is_some()
    }

    /// Returns a reference to the user overlay entries.
    #[must_use]
    pub fn user_entries(&self) -> &BTreeMap<String, DictEntry> {
        &self.user_entries
    }

    /// Returns the number of user overlay entries.
    #[must_use]
    pub fn user_len(&self) -> usize {
        self.user_entries.len()
    }

    /// Looks up the primary pronunciation of a word.
    ///
    /// Checks the user overlay first, then the base dictionary.
    /// Returns the phonemes of the highest-frequency pronunciation.
    #[must_use]
    pub fn lookup(&self, word: &str) -> Option<&[Phoneme]> {
        self.lookup_entry(word)
            .map(|entry| entry.primary_phonemes())
    }

    /// Looks up the full dictionary entry for a word.
    ///
    /// Checks the user overlay first, then the base dictionary.
    /// Returns the [`DictEntry`] with all pronunciation variants.
    #[must_use]
    pub fn lookup_entry(&self, word: &str) -> Option<&DictEntry> {
        let key = alloc::string::ToString::to_string(&word.to_lowercase());
        self.user_entries
            .get(&key)
            .or_else(|| self.entries.get(&key))
    }

    /// Looks up all pronunciations of a word.
    ///
    /// Checks the user overlay first, then the base dictionary.
    #[must_use]
    pub fn lookup_all(&self, word: &str) -> Option<&[Pronunciation]> {
        self.lookup_entry(word).map(|entry| entry.all())
    }

    /// Returns the number of base dictionary entries.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns whether the base dictionary is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns a reference to the base entries.
    #[must_use]
    pub fn entries(&self) -> &HashMap<String, DictEntry> {
        &self.entries
    }

    /// Merges another dictionary into this one.
    ///
    /// For words present in both, entries from `other` replace entries in `self`.
    /// Base and user entries are merged into their respective layers.
    pub fn merge(&mut self, other: &PronunciationDict) {
        for (word, entry) in other.entries() {
            self.entries.insert(word.clone(), entry.clone());
        }
        for (word, entry) in other.user_entries() {
            self.user_entries.insert(word.clone(), entry.clone());
        }
    }

    /// Merges another dictionary, keeping self's entries on conflict.
    ///
    /// Only entries for words not already in `self` are added.
    pub fn merge_conservative(&mut self, other: &PronunciationDict) {
        for (word, entry) in other.entries() {
            if !self.entries.contains_key(word) {
                self.entries.insert(word.clone(), entry.clone());
            }
        }
        for (word, entry) in other.user_entries() {
            if !self.user_entries.contains_key(word) {
                self.user_entries.insert(word.clone(), entry.clone());
            }
        }
    }
}

/// Differences between two pronunciation dictionaries.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DictDiff {
    /// Words present in `right` but not `left`.
    pub added: Vec<String>,
    /// Words present in `left` but not `right`.
    pub removed: Vec<String>,
    /// Words in both but with different primary pronunciations.
    pub changed: Vec<String>,
}

impl DictDiff {
    /// Returns true if the dictionaries are identical.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.added.is_empty() && self.removed.is_empty() && self.changed.is_empty()
    }

    /// Total number of differences.
    #[must_use]
    pub fn len(&self) -> usize {
        self.added.len() + self.removed.len() + self.changed.len()
    }
}

/// Computes the differences between two dictionaries.
///
/// Compares the effective lookup result for each word (user overlay takes
/// precedence over base, mirroring [`PronunciationDict::lookup_entry`] behavior).
/// Results are sorted alphabetically.
#[must_use]
pub fn diff(left: &PronunciationDict, right: &PronunciationDict) -> DictDiff {
    let mut all_words = alloc::collections::BTreeSet::new();
    for word in left.entries().keys() {
        all_words.insert(word.as_str());
    }
    for word in left.user_entries().keys() {
        all_words.insert(word.as_str());
    }
    for word in right.entries().keys() {
        all_words.insert(word.as_str());
    }
    for word in right.user_entries().keys() {
        all_words.insert(word.as_str());
    }

    let mut result = DictDiff::default();

    for word in all_words {
        let l = left.lookup_entry(word);
        let r = right.lookup_entry(word);
        match (l, r) {
            (None, Some(_)) => result.added.push(alloc::string::ToString::to_string(word)),
            (Some(_), None) => result
                .removed
                .push(alloc::string::ToString::to_string(word)),
            (Some(le), Some(re)) if le != re => {
                result
                    .changed
                    .push(alloc::string::ToString::to_string(word));
            }
            _ => {}
        }
    }

    result
}

impl Default for PronunciationDict {
    fn default() -> Self {
        Self::new()
    }
}

// --- Serde backward compatibility ---
//
// v0.1.0 serialized entries as BTreeMap<String, Vec<Phoneme>>.
// v0.2.0 uses BTreeMap<String, DictEntry>.
// We use an untagged enum to accept either format per map value.

#[derive(Deserialize)]
#[serde(untagged)]
enum EntryCompat {
    New(DictEntry),
    Old(Vec<Phoneme>),
}

impl EntryCompat {
    fn into_entry(self) -> DictEntry {
        match self {
            Self::New(entry) => entry,
            Self::Old(phonemes) => DictEntry::from_phonemes(&phonemes),
        }
    }
}

fn deserialize_entries_compat<'de, D>(
    deserializer: D,
) -> core::result::Result<HashMap<String, DictEntry>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    // Deserialize as BTreeMap first (handles both JSON object key ordering variants)
    let raw: BTreeMap<String, EntryCompat> = BTreeMap::deserialize(deserializer)?;
    Ok(raw.into_iter().map(|(k, v)| (k, v.into_entry())).collect())
}

fn deserialize_user_entries_compat<'de, D>(
    deserializer: D,
) -> core::result::Result<BTreeMap<String, DictEntry>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let raw: BTreeMap<String, EntryCompat> = BTreeMap::deserialize(deserializer)?;
    Ok(raw.into_iter().map(|(k, v)| (k, v.into_entry())).collect())
}
