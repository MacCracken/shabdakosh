//! Unified phoneme notation abstraction.
//!
//! The [`PhonemeNotation`] trait provides a common interface for converting
//! between svara [`Phoneme`] values and notation-specific string symbols
//! (ARPABET, IPA, X-SAMPA, etc.).
//!
//! shabdakosh stores phonemes as notation-agnostic [`Phoneme`] IDs internally.
//! This module allows reading and writing phonemes in any supported notation.
//!
//! # Built-in notations
//!
//! | Notation | Type | Example |
//! |----------|------|---------|
//! | ARPABET  | [`Arpabet`] | `K AE1 T` |
//! | IPA      | [`Ipa`] | `kæt` |
//! | X-SAMPA  | [`XSampa`] | `k{t` |
//!
//! # Examples
//!
//! ```rust
//! use shabdakosh::notation::{PhonemeNotation, Ipa, Arpabet, XSampa};
//! use svara::phoneme::Phoneme;
//!
//! // Convert phoneme to different notations.
//! let p = Phoneme::VowelAsh;
//! assert_eq!(Ipa.phoneme_to_str(p), Some("æ"));
//! assert_eq!(Arpabet.phoneme_to_str(p), Some("AE1"));
//! assert_eq!(XSampa.phoneme_to_str(p), Some("{"));
//!
//! // Parse from notation.
//! assert_eq!(Ipa.str_to_phoneme("æ"), Some(Phoneme::VowelAsh));
//! assert_eq!(XSampa.str_to_phoneme("{"), Some(Phoneme::VowelAsh));
//! ```

use alloc::{string::String, vec::Vec};
use svara::phoneme::Phoneme;

/// A phoneme notation system (ARPABET, IPA, X-SAMPA, etc.).
///
/// Implementations provide bidirectional conversion between notation-specific
/// string symbols and svara [`Phoneme`] values.
pub trait PhonemeNotation: Send + Sync {
    /// The short name of this notation (e.g., "ipa", "arpabet", "x-sampa").
    fn name(&self) -> &'static str;

    /// Converts a notation-specific string symbol to a [`Phoneme`].
    ///
    /// Returns `None` if the symbol is not recognized.
    fn str_to_phoneme(&self, symbol: &str) -> Option<Phoneme>;

    /// Converts a [`Phoneme`] to its notation-specific string representation.
    ///
    /// Returns `None` if the phoneme has no representation in this notation.
    fn phoneme_to_str(&self, phoneme: Phoneme) -> Option<&'static str>;

    /// Parses a full transcription string into a phoneme sequence.
    ///
    /// The default implementation splits on whitespace and converts each token.
    /// Notations with special parsing rules (e.g., IPA multi-char sequences)
    /// should override this.
    fn parse(&self, transcription: &str) -> Vec<Phoneme> {
        transcription
            .split_whitespace()
            .filter_map(|s| self.str_to_phoneme(s))
            .collect()
    }

    /// Renders a phoneme sequence to a notation string.
    ///
    /// The default implementation joins symbols with spaces.
    fn render(&self, phonemes: &[Phoneme]) -> String {
        let symbols: Vec<&str> = phonemes
            .iter()
            .filter_map(|p| self.phoneme_to_str(*p))
            .collect();
        symbols.join(" ")
    }
}

// --- ARPABET notation ---

/// ARPABET phoneme notation (as used in CMUdict).
///
/// Symbols are uppercase ASCII (e.g., `K`, `AE`, `T`).
/// Stress digits (0/1/2) are not included in symbol mapping.
#[derive(Debug, Clone, Copy)]
pub struct Arpabet;

impl PhonemeNotation for Arpabet {
    fn name(&self) -> &'static str {
        "arpabet"
    }

    fn str_to_phoneme(&self, symbol: &str) -> Option<Phoneme> {
        crate::arpabet::arpabet_to_phoneme(symbol)
    }

    fn phoneme_to_str(&self, phoneme: Phoneme) -> Option<&'static str> {
        crate::arpabet::phoneme_to_arpabet(&phoneme)
    }
}

// --- IPA notation ---

/// International Phonetic Alphabet notation.
///
/// Symbols are Unicode characters (e.g., `k`, `æ`, `t`).
#[derive(Debug, Clone, Copy)]
pub struct Ipa;

impl PhonemeNotation for Ipa {
    fn name(&self) -> &'static str {
        "ipa"
    }

    fn str_to_phoneme(&self, symbol: &str) -> Option<Phoneme> {
        crate::ipa::ipa_to_phoneme(symbol)
    }

    fn phoneme_to_str(&self, phoneme: Phoneme) -> Option<&'static str> {
        crate::ipa::phoneme_to_ipa(&phoneme)
    }

    /// Parses an IPA transcription using greedy longest-match.
    fn parse(&self, transcription: &str) -> Vec<Phoneme> {
        crate::ipa::parse_ipa_word(transcription)
    }

    fn render(&self, phonemes: &[Phoneme]) -> String {
        crate::ipa::phonemes_to_ipa(phonemes)
    }
}

// --- X-SAMPA notation ---

/// X-SAMPA (Extended Speech Assessment Methods Phonetic Alphabet).
///
/// An ASCII-only representation of IPA, commonly used in computational
/// linguistics and speech synthesis systems.
#[derive(Debug, Clone, Copy)]
pub struct XSampa;

impl PhonemeNotation for XSampa {
    fn name(&self) -> &'static str {
        "x-sampa"
    }

    fn str_to_phoneme(&self, symbol: &str) -> Option<Phoneme> {
        xsampa_to_phoneme(symbol)
    }

    fn phoneme_to_str(&self, phoneme: Phoneme) -> Option<&'static str> {
        phoneme_to_xsampa(phoneme)
    }
}

/// Converts an X-SAMPA symbol to a svara Phoneme.
#[must_use]
fn xsampa_to_phoneme(symbol: &str) -> Option<Phoneme> {
    match symbol {
        // Vowels
        "a" => Some(Phoneme::VowelA),
        "e" => Some(Phoneme::VowelE),
        "i" => Some(Phoneme::VowelI),
        "o" => Some(Phoneme::VowelO),
        "u" => Some(Phoneme::VowelU),
        "@" => Some(Phoneme::VowelSchwa),
        "O" => Some(Phoneme::VowelOpenO),
        "{" => Some(Phoneme::VowelAsh),
        "I" => Some(Phoneme::VowelNearI),
        "U" => Some(Phoneme::VowelNearU),
        "A" => Some(Phoneme::VowelOpenA),
        "E" => Some(Phoneme::VowelOpenE),
        "V" => Some(Phoneme::VowelCupV),
        "3" => Some(Phoneme::VowelBird),
        "i:" => Some(Phoneme::VowelLongI),

        // Diphthongs
        "aI" => Some(Phoneme::DiphthongAI),
        "aU" => Some(Phoneme::DiphthongAU),
        "eI" => Some(Phoneme::DiphthongEI),
        "OI" => Some(Phoneme::DiphthongOI),
        "oU" => Some(Phoneme::DiphthongOU),

        // Plosives
        "p" => Some(Phoneme::PlosiveP),
        "b" => Some(Phoneme::PlosiveB),
        "t" => Some(Phoneme::PlosiveT),
        "d" => Some(Phoneme::PlosiveD),
        "k" => Some(Phoneme::PlosiveK),
        "g" => Some(Phoneme::PlosiveG),

        // Fricatives
        "f" => Some(Phoneme::FricativeF),
        "v" => Some(Phoneme::FricativeV),
        "s" => Some(Phoneme::FricativeS),
        "z" => Some(Phoneme::FricativeZ),
        "S" => Some(Phoneme::FricativeSh),
        "Z" => Some(Phoneme::FricativeZh),
        "T" => Some(Phoneme::FricativeTh),
        "D" => Some(Phoneme::FricativeDh),
        "h" => Some(Phoneme::FricativeH),

        // Nasals
        "m" => Some(Phoneme::NasalM),
        "n" => Some(Phoneme::NasalN),
        "N" => Some(Phoneme::NasalNg),

        // Affricates
        "tS" => Some(Phoneme::AffricateCh),
        "dZ" => Some(Phoneme::AffricateJ),

        // Approximants
        "r\\" | "r" => Some(Phoneme::ApproximantR),
        "w" => Some(Phoneme::ApproximantW),
        "j" => Some(Phoneme::ApproximantJ),

        // Lateral
        "l" => Some(Phoneme::LateralL),

        // Other
        "?" => Some(Phoneme::GlottalStop),
        "4" => Some(Phoneme::TapFlap),

        _ => None,
    }
}

/// Converts a svara Phoneme to its X-SAMPA symbol.
#[must_use]
fn phoneme_to_xsampa(phoneme: Phoneme) -> Option<&'static str> {
    match phoneme {
        // Vowels
        Phoneme::VowelA => Some("a"),
        Phoneme::VowelE => Some("e"),
        Phoneme::VowelI => Some("i"),
        Phoneme::VowelO => Some("o"),
        Phoneme::VowelU => Some("u"),
        Phoneme::VowelSchwa => Some("@"),
        Phoneme::VowelOpenO => Some("O"),
        Phoneme::VowelAsh => Some("{"),
        Phoneme::VowelNearI => Some("I"),
        Phoneme::VowelNearU => Some("U"),
        Phoneme::VowelOpenA => Some("A"),
        Phoneme::VowelOpenE => Some("E"),
        Phoneme::VowelCupV => Some("V"),
        Phoneme::VowelBird => Some("3"),
        Phoneme::VowelLongI => Some("i:"),

        // Diphthongs
        Phoneme::DiphthongAI => Some("aI"),
        Phoneme::DiphthongAU => Some("aU"),
        Phoneme::DiphthongEI => Some("eI"),
        Phoneme::DiphthongOI => Some("OI"),
        Phoneme::DiphthongOU => Some("oU"),

        // Plosives
        Phoneme::PlosiveP => Some("p"),
        Phoneme::PlosiveB => Some("b"),
        Phoneme::PlosiveT => Some("t"),
        Phoneme::PlosiveD => Some("d"),
        Phoneme::PlosiveK => Some("k"),
        Phoneme::PlosiveG => Some("g"),

        // Fricatives
        Phoneme::FricativeF => Some("f"),
        Phoneme::FricativeV => Some("v"),
        Phoneme::FricativeS => Some("s"),
        Phoneme::FricativeZ => Some("z"),
        Phoneme::FricativeSh => Some("S"),
        Phoneme::FricativeZh => Some("Z"),
        Phoneme::FricativeTh => Some("T"),
        Phoneme::FricativeDh => Some("D"),
        Phoneme::FricativeH => Some("h"),

        // Nasals
        Phoneme::NasalM => Some("m"),
        Phoneme::NasalN => Some("n"),
        Phoneme::NasalNg => Some("N"),

        // Affricates
        Phoneme::AffricateCh => Some("tS"),
        Phoneme::AffricateJ => Some("dZ"),

        // Approximants
        Phoneme::ApproximantR => Some("r\\"),
        Phoneme::ApproximantW => Some("w"),
        Phoneme::ApproximantJ => Some("j"),

        // Lateral
        Phoneme::LateralL => Some("l"),

        // Other
        Phoneme::GlottalStop => Some("?"),
        Phoneme::TapFlap => Some("4"),

        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arpabet_roundtrip_consonants() {
        let notation = Arpabet;
        // Consonants roundtrip cleanly (no stress digits).
        for symbol in ["P", "B", "T", "D", "K", "F", "S", "M", "N"] {
            let phoneme = notation.str_to_phoneme(symbol).unwrap();
            let back = notation.phoneme_to_str(phoneme).unwrap();
            assert_eq!(symbol, back, "ARPABET roundtrip failed for {symbol}");
        }
    }

    #[test]
    fn test_arpabet_vowel_mapping() {
        let notation = Arpabet;
        // Vowels include stress digits in output (e.g., AE -> AE1).
        let p = notation.str_to_phoneme("AE").unwrap();
        assert_eq!(p, Phoneme::VowelAsh);
        // phoneme_to_str returns with stress digit.
        assert_eq!(notation.phoneme_to_str(p), Some("AE1"));
    }

    #[test]
    fn test_ipa_roundtrip() {
        let notation = Ipa;
        for symbol in ["p", "b", "t", "k", "æ", "ə", "ʃ"] {
            let phoneme = notation.str_to_phoneme(symbol).unwrap();
            let back = notation.phoneme_to_str(phoneme).unwrap();
            assert_eq!(symbol, back, "IPA roundtrip failed for {symbol}");
        }
    }

    #[test]
    fn test_xsampa_roundtrip() {
        let notation = XSampa;
        for symbol in ["p", "b", "t", "k", "{", "@", "S", "N", "tS", "dZ"] {
            let phoneme = notation.str_to_phoneme(symbol).unwrap();
            let back = notation.phoneme_to_str(phoneme).unwrap();
            assert_eq!(symbol, back, "X-SAMPA roundtrip failed for {symbol}");
        }
    }

    #[test]
    fn test_cross_notation_equivalence() {
        // Same phoneme in all three notations.
        let p = Phoneme::VowelAsh;
        assert_eq!(Arpabet.phoneme_to_str(p), Some("AE1")); // ARPABET includes stress
        assert_eq!(Ipa.phoneme_to_str(p), Some("æ"));
        assert_eq!(XSampa.phoneme_to_str(p), Some("{"));
    }

    #[test]
    fn test_arpabet_parse() {
        let phonemes = Arpabet.parse("K AE T");
        assert_eq!(
            phonemes,
            alloc::vec![Phoneme::PlosiveK, Phoneme::VowelAsh, Phoneme::PlosiveT]
        );
    }

    #[test]
    fn test_ipa_parse() {
        let phonemes = Ipa.parse("kæt");
        assert_eq!(
            phonemes,
            alloc::vec![Phoneme::PlosiveK, Phoneme::VowelAsh, Phoneme::PlosiveT]
        );
    }

    #[test]
    fn test_xsampa_parse() {
        let phonemes = XSampa.parse("k { t");
        assert_eq!(
            phonemes,
            alloc::vec![Phoneme::PlosiveK, Phoneme::VowelAsh, Phoneme::PlosiveT]
        );
    }

    #[test]
    fn test_arpabet_render() {
        let phonemes = [Phoneme::PlosiveK, Phoneme::VowelAsh, Phoneme::PlosiveT];
        assert_eq!(Arpabet.render(&phonemes), "K AE1 T");
    }

    #[test]
    fn test_ipa_render() {
        let phonemes = [Phoneme::PlosiveK, Phoneme::VowelAsh, Phoneme::PlosiveT];
        assert_eq!(Ipa.render(&phonemes), "kæt");
    }

    #[test]
    fn test_xsampa_render() {
        let phonemes = [Phoneme::PlosiveK, Phoneme::VowelAsh, Phoneme::PlosiveT];
        assert_eq!(XSampa.render(&phonemes), "k { t");
    }

    #[test]
    fn test_notation_names() {
        assert_eq!(Arpabet.name(), "arpabet");
        assert_eq!(Ipa.name(), "ipa");
        assert_eq!(XSampa.name(), "x-sampa");
    }

    #[test]
    fn test_unknown_symbol() {
        assert!(Arpabet.str_to_phoneme("ZZZZZ").is_none());
        assert!(Ipa.str_to_phoneme("🎵").is_none());
        assert!(XSampa.str_to_phoneme("!!!").is_none());
    }
}
