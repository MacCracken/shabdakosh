//! Syllable representation and syllabification.
//!
//! Provides [`Syllable`] for representing syllable structure within
//! pronunciation data, and a basic syllabification algorithm based on
//! the Maximal Onset Principle.
//!
//! # Examples
//!
//! ```rust
//! use shabdakosh::dictionary::syllable::{Syllable, StressLevel, syllabify};
//! use svara::phoneme::Phoneme;
//!
//! let phonemes = [Phoneme::PlosiveK, Phoneme::VowelAsh, Phoneme::PlosiveT];
//! let syllables = syllabify(&phonemes);
//! assert_eq!(syllables.len(), 1); // "cat" = one syllable
//! assert_eq!(syllables[0].phonemes(), &phonemes);
//! ```

use alloc::vec::Vec;
use serde::{Deserialize, Serialize};
use svara::phoneme::Phoneme;

/// Stress level of a syllable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum StressLevel {
    /// No stress.
    Unstressed,
    /// Primary stress (IPA: ˈ).
    Primary,
    /// Secondary stress (IPA: ˌ).
    Secondary,
}

/// A single syllable within a pronunciation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Syllable {
    phonemes: Vec<Phoneme>,
    stress: StressLevel,
}

impl Syllable {
    /// Creates a new unstressed syllable.
    #[must_use]
    pub fn new(phonemes: Vec<Phoneme>) -> Self {
        Self {
            phonemes,
            stress: StressLevel::Unstressed,
        }
    }

    /// Sets the stress level (builder pattern).
    #[must_use]
    pub fn with_stress(mut self, stress: StressLevel) -> Self {
        self.stress = stress;
        self
    }

    /// Returns the phonemes in this syllable.
    #[must_use]
    pub fn phonemes(&self) -> &[Phoneme] {
        &self.phonemes
    }

    /// Returns the stress level.
    #[must_use]
    pub fn stress(&self) -> StressLevel {
        self.stress
    }

    /// Returns the number of phonemes in this syllable.
    #[must_use]
    pub fn len(&self) -> usize {
        self.phonemes.len()
    }

    /// Returns whether this syllable is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.phonemes.is_empty()
    }
}

/// Returns `true` if the phoneme is a vowel or diphthong (syllable nucleus).
#[must_use]
fn is_nucleus(phoneme: &Phoneme) -> bool {
    matches!(
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
            | Phoneme::VowelLongI
            | Phoneme::DiphthongAI
            | Phoneme::DiphthongAU
            | Phoneme::DiphthongEI
            | Phoneme::DiphthongOI
            | Phoneme::DiphthongOU
    )
}

/// Syllabifies a phoneme sequence using the Maximal Onset Principle.
///
/// The algorithm:
/// 1. Identify vowel nuclei (vowels and diphthongs).
/// 2. Assign consonants between vowels to maximize the onset of the following
///    syllable (Maximal Onset Principle), while respecting that at least one
///    consonant stays as the coda of the preceding syllable when possible.
/// 3. Initial consonants go to the first syllable; final consonants go to the last.
///
/// # Examples
///
/// ```rust
/// use shabdakosh::dictionary::syllable::syllabify;
/// use svara::phoneme::Phoneme;
///
/// // "hello" = HH-EH-L-OW → 2 syllables: [HH,EH] [L,OW]
/// let phonemes = [
///     Phoneme::FricativeH, Phoneme::VowelOpenE,
///     Phoneme::LateralL, Phoneme::DiphthongOU,
/// ];
/// let syllables = syllabify(&phonemes);
/// assert_eq!(syllables.len(), 2);
/// ```
#[must_use]
pub fn syllabify(phonemes: &[Phoneme]) -> Vec<Syllable> {
    if phonemes.is_empty() {
        return Vec::new();
    }

    // Find indices of vowel nuclei.
    let nuclei: Vec<usize> = phonemes
        .iter()
        .enumerate()
        .filter(|(_, p)| is_nucleus(p))
        .map(|(i, _)| i)
        .collect();

    // No vowels — entire sequence is one syllable.
    if nuclei.is_empty() {
        return alloc::vec![Syllable::new(phonemes.to_vec())];
    }

    let mut syllables = Vec::with_capacity(nuclei.len());
    let mut start = 0_usize;

    for (i, &nucleus_idx) in nuclei.iter().enumerate() {
        let end = if i + 1 < nuclei.len() {
            let next_nucleus = nuclei[i + 1];
            // Consonants between this nucleus and the next.
            // Maximal Onset Principle: give as many consonants as possible
            // to the next syllable's onset, keeping at least one in coda
            // of the current syllable (if there's more than one consonant).
            let gap_start = nucleus_idx + 1;
            let gap_len = next_nucleus - gap_start;

            if gap_len <= 1 {
                // 0 or 1 consonant between vowels: all go to next onset.
                gap_start
            } else {
                // Multiple consonants: split — first stays as coda, rest go to onset.
                gap_start + 1
            }
        } else {
            // Last syllable: take everything to the end.
            phonemes.len()
        };

        syllables.push(Syllable::new(phonemes[start..end].to_vec()));
        start = end;
    }

    syllables
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syllable_new() {
        let s = Syllable::new(alloc::vec![Phoneme::PlosiveK, Phoneme::VowelAsh]);
        assert_eq!(s.len(), 2);
        assert_eq!(s.stress(), StressLevel::Unstressed);
        assert!(!s.is_empty());
    }

    #[test]
    fn test_syllable_with_stress() {
        let s = Syllable::new(alloc::vec![Phoneme::VowelA]).with_stress(StressLevel::Primary);
        assert_eq!(s.stress(), StressLevel::Primary);
    }

    #[test]
    fn test_syllabify_empty() {
        assert!(syllabify(&[]).is_empty());
    }

    #[test]
    fn test_syllabify_single_vowel() {
        let syls = syllabify(&[Phoneme::VowelA]);
        assert_eq!(syls.len(), 1);
        assert_eq!(syls[0].phonemes(), &[Phoneme::VowelA]);
    }

    #[test]
    fn test_syllabify_cvc() {
        // "cat" = K-AE-T → 1 syllable
        let syls = syllabify(&[Phoneme::PlosiveK, Phoneme::VowelAsh, Phoneme::PlosiveT]);
        assert_eq!(syls.len(), 1);
    }

    #[test]
    fn test_syllabify_cvcv() {
        // "mama" = M-A-M-A → 2 syllables: [M,A] [M,A]
        let syls = syllabify(&[
            Phoneme::NasalM,
            Phoneme::VowelA,
            Phoneme::NasalM,
            Phoneme::VowelA,
        ]);
        assert_eq!(syls.len(), 2);
        assert_eq!(syls[0].phonemes(), &[Phoneme::NasalM, Phoneme::VowelA]);
        assert_eq!(syls[1].phonemes(), &[Phoneme::NasalM, Phoneme::VowelA]);
    }

    #[test]
    fn test_syllabify_hello() {
        // HH-EH-L-OW → 2 syllables
        let syls = syllabify(&[
            Phoneme::FricativeH,
            Phoneme::VowelOpenE,
            Phoneme::LateralL,
            Phoneme::DiphthongOU,
        ]);
        assert_eq!(syls.len(), 2);
    }

    #[test]
    fn test_syllabify_consonant_cluster() {
        // S-T-R-IY-T → "street" = 1 syllable (all consonants before single vowel)
        let syls = syllabify(&[
            Phoneme::FricativeS,
            Phoneme::PlosiveT,
            Phoneme::ApproximantR,
            Phoneme::VowelE,
            Phoneme::PlosiveT,
        ]);
        assert_eq!(syls.len(), 1);
    }

    #[test]
    fn test_syllabify_no_vowels() {
        // All consonants — one syllable.
        let syls = syllabify(&[Phoneme::PlosiveP, Phoneme::FricativeS]);
        assert_eq!(syls.len(), 1);
    }

    #[test]
    fn test_syllable_serde_roundtrip() {
        let s = Syllable::new(alloc::vec![
            Phoneme::PlosiveK,
            Phoneme::VowelAsh,
            Phoneme::PlosiveT
        ])
        .with_stress(StressLevel::Primary);
        let json = serde_json::to_string(&s).unwrap();
        let s2: Syllable = serde_json::from_str(&json).unwrap();
        assert_eq!(s, s2);
    }

    #[test]
    fn test_stress_level_serde_roundtrip() {
        for level in [
            StressLevel::Unstressed,
            StressLevel::Primary,
            StressLevel::Secondary,
        ] {
            let json = serde_json::to_string(&level).unwrap();
            let level2: StressLevel = serde_json::from_str(&json).unwrap();
            assert_eq!(level, level2);
        }
    }

    #[test]
    fn test_is_nucleus() {
        assert!(is_nucleus(&Phoneme::VowelA));
        assert!(is_nucleus(&Phoneme::DiphthongAI));
        assert!(is_nucleus(&Phoneme::VowelSchwa));
        assert!(!is_nucleus(&Phoneme::PlosiveK));
        assert!(!is_nucleus(&Phoneme::NasalM));
        assert!(!is_nucleus(&Phoneme::Silence));
    }
}
