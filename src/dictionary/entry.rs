//! Dictionary entry types: pronunciations with frequency and region metadata.

use alloc::vec::Vec;
use serde::{Deserialize, Serialize};
use svara::phoneme::Phoneme;

/// A single pronunciation of a word.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pronunciation {
    phonemes: Vec<Phoneme>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    frequency: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    region: Option<Region>,
}

impl Pronunciation {
    /// Creates a new pronunciation with no frequency or region metadata.
    #[must_use]
    pub fn new(phonemes: Vec<Phoneme>) -> Self {
        Self {
            phonemes,
            frequency: None,
            region: None,
        }
    }

    /// Sets the relative frequency (0.0–1.0) for this pronunciation.
    #[must_use]
    pub fn with_frequency(mut self, frequency: f32) -> Self {
        self.frequency = Some(frequency);
        self
    }

    /// Sets the regional variant for this pronunciation.
    #[must_use]
    pub fn with_region(mut self, region: Region) -> Self {
        self.region = Some(region);
        self
    }

    /// Returns the phoneme sequence.
    #[must_use]
    pub fn phonemes(&self) -> &[Phoneme] {
        &self.phonemes
    }

    /// Returns the relative frequency, if known.
    #[must_use]
    pub fn frequency(&self) -> Option<f32> {
        self.frequency
    }

    /// Returns the regional variant, if specified.
    #[must_use]
    pub fn region(&self) -> Option<Region> {
        self.region
    }
}

/// Regional pronunciation variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Region {
    /// General American English.
    GeneralAmerican,
    /// Received Pronunciation (British English).
    ReceivedPronunciation,
}

impl Region {
    /// Parses a region from its short code.
    #[must_use]
    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "GA" => Some(Self::GeneralAmerican),
            "RP" => Some(Self::ReceivedPronunciation),
            _ => None,
        }
    }

    /// Returns the short code for this region.
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::GeneralAmerican => "GA",
            Self::ReceivedPronunciation => "RP",
        }
    }
}

impl core::fmt::Display for Region {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.code())
    }
}

/// A dictionary entry containing one or more pronunciations of a word.
///
/// Pronunciations are ordered by frequency descending (highest first).
/// The invariant that `pronunciations` is never empty is maintained by
/// all constructors.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DictEntry {
    pronunciations: Vec<Pronunciation>,
}

impl DictEntry {
    /// Creates an entry with a single pronunciation.
    #[must_use]
    pub fn new(pronunciation: Pronunciation) -> Self {
        Self {
            pronunciations: alloc::vec![pronunciation],
        }
    }

    /// Creates an entry from a slice of phonemes (convenience for single pronunciation).
    #[must_use]
    pub fn from_phonemes(phonemes: &[Phoneme]) -> Self {
        Self::new(Pronunciation::new(phonemes.to_vec()))
    }

    /// Creates an entry from multiple pronunciations.
    ///
    /// Returns `None` if `pronunciations` is empty.
    /// Sorts by frequency descending (known frequencies first, `None` last).
    #[must_use]
    pub fn from_pronunciations(mut pronunciations: Vec<Pronunciation>) -> Option<Self> {
        if pronunciations.is_empty() {
            return None;
        }
        sort_by_frequency(&mut pronunciations);
        Some(Self { pronunciations })
    }

    /// Adds a pronunciation variant to this entry.
    ///
    /// Re-sorts by frequency after insertion.
    pub fn add_pronunciation(&mut self, pronunciation: Pronunciation) {
        self.pronunciations.push(pronunciation);
        sort_by_frequency(&mut self.pronunciations);
    }

    /// Returns the primary (highest-frequency) pronunciation.
    #[must_use]
    pub fn primary(&self) -> &Pronunciation {
        // SAFETY: invariant guarantees non-empty
        &self.pronunciations[0]
    }

    /// Returns the phonemes of the primary pronunciation.
    #[must_use]
    pub fn primary_phonemes(&self) -> &[Phoneme] {
        self.primary().phonemes()
    }

    /// Returns all pronunciations.
    #[must_use]
    pub fn all(&self) -> &[Pronunciation] {
        &self.pronunciations
    }

    /// Returns the first pronunciation matching the given region.
    #[must_use]
    pub fn for_region(&self, region: Region) -> Option<&Pronunciation> {
        self.pronunciations
            .iter()
            .find(|p| p.region() == Some(region))
    }

    /// Returns the number of pronunciation variants.
    #[must_use]
    pub fn len(&self) -> usize {
        self.pronunciations.len()
    }

    /// Always returns `false` (entries are guaranteed non-empty).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        false
    }

    /// Returns an iterator over all pronunciations.
    pub fn iter(&self) -> core::slice::Iter<'_, Pronunciation> {
        self.pronunciations.iter()
    }
}

/// Sorts pronunciations by frequency descending. Known frequencies first, `None` last.
fn sort_by_frequency(pronunciations: &mut [Pronunciation]) {
    pronunciations.sort_by(|a, b| {
        let fa = a.frequency.unwrap_or(-1.0);
        let fb = b.frequency.unwrap_or(-1.0);
        fb.partial_cmp(&fa).unwrap_or(core::cmp::Ordering::Equal)
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pronunciation_new() {
        let p = Pronunciation::new(alloc::vec![
            Phoneme::PlosiveK,
            Phoneme::VowelAsh,
            Phoneme::PlosiveT
        ]);
        assert_eq!(p.phonemes().len(), 3);
        assert_eq!(p.frequency(), None);
        assert_eq!(p.region(), None);
    }

    #[test]
    fn test_pronunciation_builder() {
        let p = Pronunciation::new(alloc::vec![Phoneme::PlosiveK])
            .with_frequency(0.75)
            .with_region(Region::GeneralAmerican);
        assert_eq!(p.frequency(), Some(0.75));
        assert_eq!(p.region(), Some(Region::GeneralAmerican));
    }

    #[test]
    fn test_region_code_roundtrip() {
        assert_eq!(Region::from_code("GA"), Some(Region::GeneralAmerican));
        assert_eq!(Region::from_code("RP"), Some(Region::ReceivedPronunciation));
        assert_eq!(Region::from_code("XX"), None);
        assert_eq!(Region::GeneralAmerican.code(), "GA");
        assert_eq!(Region::ReceivedPronunciation.code(), "RP");
    }

    #[test]
    fn test_dict_entry_single() {
        let entry = DictEntry::from_phonemes(&[Phoneme::PlosiveK]);
        assert_eq!(entry.len(), 1);
        assert!(!entry.is_empty());
        assert_eq!(entry.primary_phonemes(), &[Phoneme::PlosiveK]);
    }

    #[test]
    fn test_dict_entry_empty_rejected() {
        assert!(DictEntry::from_pronunciations(alloc::vec![]).is_none());
    }

    #[test]
    fn test_dict_entry_frequency_ordering() {
        let p1 = Pronunciation::new(alloc::vec![Phoneme::VowelA]).with_frequency(0.3);
        let p2 = Pronunciation::new(alloc::vec![Phoneme::VowelE]).with_frequency(0.7);
        let entry = DictEntry::from_pronunciations(alloc::vec![p1, p2]).unwrap();
        // Higher frequency should be first
        assert_eq!(entry.primary_phonemes(), &[Phoneme::VowelE]);
        assert_eq!(entry.all()[1].phonemes(), &[Phoneme::VowelA]);
    }

    #[test]
    fn test_dict_entry_none_frequency_sorts_last() {
        let p1 = Pronunciation::new(alloc::vec![Phoneme::VowelA]); // no frequency
        let p2 = Pronunciation::new(alloc::vec![Phoneme::VowelE]).with_frequency(0.5);
        let entry = DictEntry::from_pronunciations(alloc::vec![p1, p2]).unwrap();
        assert_eq!(entry.primary_phonemes(), &[Phoneme::VowelE]);
    }

    #[test]
    fn test_dict_entry_for_region() {
        let p1 =
            Pronunciation::new(alloc::vec![Phoneme::VowelA]).with_region(Region::GeneralAmerican);
        let p2 = Pronunciation::new(alloc::vec![Phoneme::VowelOpenA])
            .with_region(Region::ReceivedPronunciation);
        let entry = DictEntry::from_pronunciations(alloc::vec![p1, p2]).unwrap();
        let rp = entry.for_region(Region::ReceivedPronunciation).unwrap();
        assert_eq!(rp.phonemes(), &[Phoneme::VowelOpenA]);
    }

    #[test]
    fn test_dict_entry_add_pronunciation() {
        let mut entry = DictEntry::from_phonemes(&[Phoneme::VowelA]);
        entry.add_pronunciation(
            Pronunciation::new(alloc::vec![Phoneme::VowelE]).with_frequency(0.9),
        );
        assert_eq!(entry.len(), 2);
        // New high-frequency entry should become primary
        assert_eq!(entry.primary_phonemes(), &[Phoneme::VowelE]);
    }

    #[test]
    fn test_serde_roundtrip_pronunciation() {
        let p = Pronunciation::new(alloc::vec![Phoneme::PlosiveK, Phoneme::VowelAsh])
            .with_frequency(0.8)
            .with_region(Region::GeneralAmerican);
        let json = serde_json::to_string(&p).unwrap();
        let p2: Pronunciation = serde_json::from_str(&json).unwrap();
        assert_eq!(p, p2);
    }

    #[test]
    fn test_serde_roundtrip_dict_entry() {
        let p1 = Pronunciation::new(alloc::vec![Phoneme::VowelA]).with_frequency(0.7);
        let p2 = Pronunciation::new(alloc::vec![Phoneme::VowelE]).with_frequency(0.3);
        let entry = DictEntry::from_pronunciations(alloc::vec![p1, p2]).unwrap();
        let json = serde_json::to_string(&entry).unwrap();
        let entry2: DictEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(entry, entry2);
    }

    #[test]
    fn test_serde_roundtrip_region() {
        let r = Region::GeneralAmerican;
        let json = serde_json::to_string(&r).unwrap();
        let r2: Region = serde_json::from_str(&json).unwrap();
        assert_eq!(r, r2);
    }
}
