//! Morphological decomposition for dictionary entries.
//!
//! Enables productive pronunciation of derived forms by tagging entries
//! with their morphological structure (e.g., "un+happy", "re+write").
//!
//! # Examples
//!
//! ```rust
//! use shabdakosh::dictionary::morphology::{Morpheme, MorphemeKind, Decomposition};
//! use svara::phoneme::Phoneme;
//!
//! let decomp = Decomposition::new(vec![
//!     Morpheme::new("un", MorphemeKind::Prefix, vec![Phoneme::VowelCupV, Phoneme::NasalN]),
//!     Morpheme::new("happy", MorphemeKind::Root, vec![
//!         Phoneme::FricativeH, Phoneme::VowelAsh, Phoneme::PlosiveP, Phoneme::VowelE,
//!     ]),
//! ]);
//! assert_eq!(decomp.morphemes().len(), 2);
//! assert_eq!(decomp.composite_phonemes().len(), 6);
//! ```

use alloc::{string::String, vec::Vec};
use serde::{Deserialize, Serialize};
use svara::phoneme::Phoneme;

/// The kind of morpheme within a word.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum MorphemeKind {
    /// A prefix (e.g., "un-", "re-", "pre-").
    Prefix,
    /// The root/stem morpheme.
    Root,
    /// A suffix (e.g., "-ing", "-ed", "-ly").
    Suffix,
    /// An infix (inserted within a root).
    Infix,
}

/// A single morpheme with its pronunciation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Morpheme {
    /// The written form of the morpheme.
    text: String,
    /// The morpheme's role in the word.
    kind: MorphemeKind,
    /// The pronunciation of this morpheme in isolation.
    phonemes: Vec<Phoneme>,
}

impl Morpheme {
    /// Creates a new morpheme.
    #[must_use]
    pub fn new(text: &str, kind: MorphemeKind, phonemes: Vec<Phoneme>) -> Self {
        Self {
            text: alloc::string::ToString::to_string(text),
            kind,
            phonemes,
        }
    }

    /// Returns the written form.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Returns the morpheme kind.
    #[must_use]
    pub fn kind(&self) -> MorphemeKind {
        self.kind
    }

    /// Returns the pronunciation of this morpheme.
    #[must_use]
    pub fn phonemes(&self) -> &[Phoneme] {
        &self.phonemes
    }
}

/// Morphological decomposition of a word.
///
/// Contains an ordered sequence of morphemes whose concatenation
/// forms the complete word.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Decomposition {
    morphemes: Vec<Morpheme>,
}

impl Decomposition {
    /// Creates a new decomposition from a list of morphemes.
    #[must_use]
    pub fn new(morphemes: Vec<Morpheme>) -> Self {
        Self { morphemes }
    }

    /// Returns the morphemes in order.
    #[must_use]
    pub fn morphemes(&self) -> &[Morpheme] {
        &self.morphemes
    }

    /// Returns the composite pronunciation by concatenating all morpheme phonemes.
    #[must_use]
    pub fn composite_phonemes(&self) -> Vec<Phoneme> {
        self.morphemes
            .iter()
            .flat_map(|m| m.phonemes.iter().copied())
            .collect()
    }

    /// Returns the full written form by concatenating morpheme texts.
    #[must_use]
    pub fn text(&self) -> String {
        self.morphemes.iter().map(|m| m.text.as_str()).collect()
    }

    /// Returns the number of morphemes.
    #[must_use]
    pub fn len(&self) -> usize {
        self.morphemes.len()
    }

    /// Returns whether the decomposition is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.morphemes.is_empty()
    }

    /// Returns the root morpheme, if any.
    #[must_use]
    pub fn root(&self) -> Option<&Morpheme> {
        self.morphemes.iter().find(|m| m.kind == MorphemeKind::Root)
    }

    /// Returns all prefix morphemes.
    #[must_use]
    pub fn prefixes(&self) -> Vec<&Morpheme> {
        self.morphemes
            .iter()
            .filter(|m| m.kind == MorphemeKind::Prefix)
            .collect()
    }

    /// Returns all suffix morphemes.
    #[must_use]
    pub fn suffixes(&self) -> Vec<&Morpheme> {
        self.morphemes
            .iter()
            .filter(|m| m.kind == MorphemeKind::Suffix)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn example_decomposition() -> Decomposition {
        Decomposition::new(alloc::vec![
            Morpheme::new(
                "un",
                MorphemeKind::Prefix,
                alloc::vec![Phoneme::VowelCupV, Phoneme::NasalN],
            ),
            Morpheme::new(
                "happi",
                MorphemeKind::Root,
                alloc::vec![
                    Phoneme::FricativeH,
                    Phoneme::VowelAsh,
                    Phoneme::PlosiveP,
                    Phoneme::VowelE,
                ],
            ),
            Morpheme::new(
                "ness",
                MorphemeKind::Suffix,
                alloc::vec![Phoneme::NasalN, Phoneme::VowelOpenE, Phoneme::FricativeS],
            ),
        ])
    }

    #[test]
    fn test_morpheme_new() {
        let m = Morpheme::new("un", MorphemeKind::Prefix, alloc::vec![Phoneme::VowelCupV]);
        assert_eq!(m.text(), "un");
        assert_eq!(m.kind(), MorphemeKind::Prefix);
        assert_eq!(m.phonemes(), &[Phoneme::VowelCupV]);
    }

    #[test]
    fn test_decomposition_composite() {
        let d = example_decomposition();
        let composite = d.composite_phonemes();
        assert_eq!(composite.len(), 9); // 2 + 4 + 3
    }

    #[test]
    fn test_decomposition_text() {
        let d = example_decomposition();
        assert_eq!(d.text(), "unhappiness");
    }

    #[test]
    fn test_decomposition_root() {
        let d = example_decomposition();
        let root = d.root().unwrap();
        assert_eq!(root.text(), "happi");
        assert_eq!(root.kind(), MorphemeKind::Root);
    }

    #[test]
    fn test_decomposition_prefixes_suffixes() {
        let d = example_decomposition();
        assert_eq!(d.prefixes().len(), 1);
        assert_eq!(d.suffixes().len(), 1);
        assert_eq!(d.prefixes()[0].text(), "un");
        assert_eq!(d.suffixes()[0].text(), "ness");
    }

    #[test]
    fn test_decomposition_len() {
        let d = example_decomposition();
        assert_eq!(d.len(), 3);
        assert!(!d.is_empty());
    }

    #[test]
    fn test_empty_decomposition() {
        let d = Decomposition::new(alloc::vec![]);
        assert!(d.is_empty());
        assert!(d.root().is_none());
        assert!(d.composite_phonemes().is_empty());
    }

    #[test]
    fn test_morpheme_serde_roundtrip() {
        let m = Morpheme::new(
            "un",
            MorphemeKind::Prefix,
            alloc::vec![Phoneme::VowelCupV, Phoneme::NasalN],
        );
        let json = serde_json::to_string(&m).unwrap();
        let m2: Morpheme = serde_json::from_str(&json).unwrap();
        assert_eq!(m, m2);
    }

    #[test]
    fn test_decomposition_serde_roundtrip() {
        let d = example_decomposition();
        let json = serde_json::to_string(&d).unwrap();
        let d2: Decomposition = serde_json::from_str(&json).unwrap();
        assert_eq!(d, d2);
    }

    #[test]
    fn test_morpheme_kind_serde_roundtrip() {
        for kind in [
            MorphemeKind::Prefix,
            MorphemeKind::Root,
            MorphemeKind::Suffix,
            MorphemeKind::Infix,
        ] {
            let json = serde_json::to_string(&kind).unwrap();
            let kind2: MorphemeKind = serde_json::from_str(&json).unwrap();
            assert_eq!(kind, kind2);
        }
    }
}
