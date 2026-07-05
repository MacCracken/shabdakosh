//! IPA (International Phonetic Alphabet) to svara phoneme mapping.
//!
//! Provides bidirectional conversion between IPA Unicode strings and
//! svara [`Phoneme`] variants, plus a greedy parser for full IPA transcriptions.

use alloc::vec::Vec;
use svara::phoneme::Phoneme;

/// Converts a single IPA symbol or multi-character sequence to a svara [`Phoneme`].
///
/// Returns `None` if the symbol is not recognized.
#[must_use]
pub fn ipa_to_phoneme(ipa: &str) -> Option<Phoneme> {
    match ipa {
        // Vowels
        "a" => Some(Phoneme::VowelA),
        "e" => Some(Phoneme::VowelE),
        "i" => Some(Phoneme::VowelI),
        "o" => Some(Phoneme::VowelO),
        "u" => Some(Phoneme::VowelU),
        "ə" => Some(Phoneme::VowelSchwa),
        "ɔ" => Some(Phoneme::VowelOpenO),
        "æ" => Some(Phoneme::VowelAsh),
        "ɪ" => Some(Phoneme::VowelNearI),
        "ʊ" => Some(Phoneme::VowelNearU),
        "ɑ" => Some(Phoneme::VowelOpenA),
        "ɛ" => Some(Phoneme::VowelOpenE),
        "ʌ" => Some(Phoneme::VowelCupV),
        "ɜ" | "ɝ" => Some(Phoneme::VowelBird),
        "iː" => Some(Phoneme::VowelLongI),

        // Diphthongs
        "aɪ" => Some(Phoneme::DiphthongAI),
        "aʊ" => Some(Phoneme::DiphthongAU),
        "ɔɪ" => Some(Phoneme::DiphthongOI),
        "eɪ" => Some(Phoneme::DiphthongEI),
        "oʊ" => Some(Phoneme::DiphthongOU),

        // Plosives
        "p" => Some(Phoneme::PlosiveP),
        "b" => Some(Phoneme::PlosiveB),
        "t" => Some(Phoneme::PlosiveT),
        "d" => Some(Phoneme::PlosiveD),
        "k" => Some(Phoneme::PlosiveK),
        "ɡ" | "g" => Some(Phoneme::PlosiveG),

        // Fricatives
        "f" => Some(Phoneme::FricativeF),
        "v" => Some(Phoneme::FricativeV),
        "s" => Some(Phoneme::FricativeS),
        "z" => Some(Phoneme::FricativeZ),
        "ʃ" => Some(Phoneme::FricativeSh),
        "ʒ" => Some(Phoneme::FricativeZh),
        "θ" => Some(Phoneme::FricativeTh),
        "ð" => Some(Phoneme::FricativeDh),
        "h" => Some(Phoneme::FricativeH),

        // Nasals
        "m" => Some(Phoneme::NasalM),
        "n" => Some(Phoneme::NasalN),
        "ŋ" => Some(Phoneme::NasalNg),

        // Affricates
        "tʃ" | "t͡ʃ" => Some(Phoneme::AffricateCh),
        "dʒ" | "d͡ʒ" => Some(Phoneme::AffricateJ),

        // Other
        "ʔ" => Some(Phoneme::GlottalStop),
        "ɾ" => Some(Phoneme::TapFlap),

        // Approximants
        "l" => Some(Phoneme::LateralL),
        "ɹ" | "r" => Some(Phoneme::ApproximantR),
        "w" => Some(Phoneme::ApproximantW),
        "j" => Some(Phoneme::ApproximantJ),

        _ => None,
    }
}

/// Converts a svara [`Phoneme`] to its IPA Unicode representation.
///
/// Returns `None` for phonemes that have no IPA equivalent (e.g., `Silence`).
#[must_use]
pub fn phoneme_to_ipa(phoneme: &Phoneme) -> Option<&'static str> {
    match phoneme {
        // Vowels
        Phoneme::VowelA => Some("a"),
        Phoneme::VowelE => Some("e"),
        Phoneme::VowelI => Some("i"),
        Phoneme::VowelO => Some("o"),
        Phoneme::VowelU => Some("u"),
        Phoneme::VowelSchwa => Some("ə"),
        Phoneme::VowelOpenO => Some("ɔ"),
        Phoneme::VowelAsh => Some("æ"),
        Phoneme::VowelNearI => Some("ɪ"),
        Phoneme::VowelNearU => Some("ʊ"),
        Phoneme::VowelOpenA => Some("ɑ"),
        Phoneme::VowelOpenE => Some("ɛ"),
        Phoneme::VowelCupV => Some("ʌ"),
        Phoneme::VowelBird => Some("ɜ"),
        Phoneme::VowelLongI => Some("iː"),

        // Diphthongs
        Phoneme::DiphthongAI => Some("aɪ"),
        Phoneme::DiphthongAU => Some("aʊ"),
        Phoneme::DiphthongOI => Some("ɔɪ"),
        Phoneme::DiphthongEI => Some("eɪ"),
        Phoneme::DiphthongOU => Some("oʊ"),

        // Plosives
        Phoneme::PlosiveP => Some("p"),
        Phoneme::PlosiveB => Some("b"),
        Phoneme::PlosiveT => Some("t"),
        Phoneme::PlosiveD => Some("d"),
        Phoneme::PlosiveK => Some("k"),
        Phoneme::PlosiveG => Some("ɡ"),

        // Fricatives
        Phoneme::FricativeF => Some("f"),
        Phoneme::FricativeV => Some("v"),
        Phoneme::FricativeS => Some("s"),
        Phoneme::FricativeZ => Some("z"),
        Phoneme::FricativeSh => Some("ʃ"),
        Phoneme::FricativeZh => Some("ʒ"),
        Phoneme::FricativeTh => Some("θ"),
        Phoneme::FricativeDh => Some("ð"),
        Phoneme::FricativeH => Some("h"),

        // Nasals
        Phoneme::NasalM => Some("m"),
        Phoneme::NasalN => Some("n"),
        Phoneme::NasalNg => Some("ŋ"),

        // Affricates
        Phoneme::AffricateCh => Some("tʃ"),
        Phoneme::AffricateJ => Some("dʒ"),

        // Other
        Phoneme::GlottalStop => Some("ʔ"),
        Phoneme::TapFlap => Some("ɾ"),

        // Approximants
        Phoneme::LateralL => Some("l"),
        Phoneme::ApproximantR => Some("ɹ"),
        Phoneme::ApproximantW => Some("w"),
        Phoneme::ApproximantJ => Some("j"),

        _ => None,
    }
}

/// Parses a full IPA transcription string into a sequence of phonemes.
///
/// Uses greedy longest-match parsing. Strips IPA stress markers (ˈ ˌ),
/// syllable boundaries (.), and enclosing slashes or brackets.
///
/// # Examples
///
/// ```
/// use shabdakosh::ipa::parse_ipa_word;
/// use svara::phoneme::Phoneme;
///
/// let phonemes = parse_ipa_word("hɛˈloʊ");
/// assert!(!phonemes.is_empty());
/// ```
#[must_use]
pub fn parse_ipa_word(ipa: &str) -> Vec<Phoneme> {
    let mut phonemes = Vec::new();
    let chars: Vec<char> = ipa.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let ch = chars[i];

        // Skip IPA markers: stress (ˈ ˌ), syllable boundary (.), slashes, brackets, spaces
        if matches!(ch, 'ˈ' | 'ˌ' | '.' | '/' | '[' | ']' | ' ' | 'ː') {
            // Handle long vowel marker separately — it's part of multi-char sequences
            // but standalone ː after a vowel is handled below
            i += 1;
            continue;
        }

        // Try 3-char match first (e.g., "t͡ʃ" = 3 chars: t, ͡, ʃ)
        if i + 2 < chars.len() {
            let three: alloc::string::String = chars[i..i + 3].iter().collect();
            if let Some(phoneme) = ipa_to_phoneme(&three) {
                phonemes.push(phoneme);
                i += 3;
                continue;
            }
        }

        // Try 2-char match (diphthongs, affricates, long vowels)
        if i + 1 < chars.len() {
            let two: alloc::string::String = chars[i..i + 2].iter().collect();
            if let Some(phoneme) = ipa_to_phoneme(&two) {
                phonemes.push(phoneme);
                i += 2;
                continue;
            }
        }

        // Try single char
        let one: alloc::string::String = chars[i..i + 1].iter().collect();
        if let Some(phoneme) = ipa_to_phoneme(&one) {
            phonemes.push(phoneme);
        }
        // Skip unrecognized characters silently

        i += 1;
    }

    phonemes
}

/// Converts a phoneme sequence to an IPA string.
#[must_use]
pub fn phonemes_to_ipa(phonemes: &[Phoneme]) -> alloc::string::String {
    let mut ipa = alloc::string::String::new();
    for phoneme in phonemes {
        if let Some(s) = phoneme_to_ipa(phoneme) {
            ipa.push_str(s);
        }
    }
    ipa
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vowel_mappings() {
        assert_eq!(ipa_to_phoneme("a"), Some(Phoneme::VowelA));
        assert_eq!(ipa_to_phoneme("e"), Some(Phoneme::VowelE));
        assert_eq!(ipa_to_phoneme("i"), Some(Phoneme::VowelI));
        assert_eq!(ipa_to_phoneme("o"), Some(Phoneme::VowelO));
        assert_eq!(ipa_to_phoneme("u"), Some(Phoneme::VowelU));
        assert_eq!(ipa_to_phoneme("ə"), Some(Phoneme::VowelSchwa));
        assert_eq!(ipa_to_phoneme("ɔ"), Some(Phoneme::VowelOpenO));
        assert_eq!(ipa_to_phoneme("æ"), Some(Phoneme::VowelAsh));
        assert_eq!(ipa_to_phoneme("ɪ"), Some(Phoneme::VowelNearI));
        assert_eq!(ipa_to_phoneme("ʊ"), Some(Phoneme::VowelNearU));
        assert_eq!(ipa_to_phoneme("ɑ"), Some(Phoneme::VowelOpenA));
        assert_eq!(ipa_to_phoneme("ɛ"), Some(Phoneme::VowelOpenE));
        assert_eq!(ipa_to_phoneme("ʌ"), Some(Phoneme::VowelCupV));
        assert_eq!(ipa_to_phoneme("ɜ"), Some(Phoneme::VowelBird));
        assert_eq!(ipa_to_phoneme("iː"), Some(Phoneme::VowelLongI));
    }

    #[test]
    fn test_diphthong_mappings() {
        assert_eq!(ipa_to_phoneme("aɪ"), Some(Phoneme::DiphthongAI));
        assert_eq!(ipa_to_phoneme("aʊ"), Some(Phoneme::DiphthongAU));
        assert_eq!(ipa_to_phoneme("ɔɪ"), Some(Phoneme::DiphthongOI));
        assert_eq!(ipa_to_phoneme("eɪ"), Some(Phoneme::DiphthongEI));
        assert_eq!(ipa_to_phoneme("oʊ"), Some(Phoneme::DiphthongOU));
    }

    #[test]
    fn test_consonant_mappings() {
        assert_eq!(ipa_to_phoneme("p"), Some(Phoneme::PlosiveP));
        assert_eq!(ipa_to_phoneme("b"), Some(Phoneme::PlosiveB));
        assert_eq!(ipa_to_phoneme("t"), Some(Phoneme::PlosiveT));
        assert_eq!(ipa_to_phoneme("d"), Some(Phoneme::PlosiveD));
        assert_eq!(ipa_to_phoneme("k"), Some(Phoneme::PlosiveK));
        assert_eq!(ipa_to_phoneme("ɡ"), Some(Phoneme::PlosiveG));
        assert_eq!(ipa_to_phoneme("f"), Some(Phoneme::FricativeF));
        assert_eq!(ipa_to_phoneme("v"), Some(Phoneme::FricativeV));
        assert_eq!(ipa_to_phoneme("s"), Some(Phoneme::FricativeS));
        assert_eq!(ipa_to_phoneme("z"), Some(Phoneme::FricativeZ));
        assert_eq!(ipa_to_phoneme("ʃ"), Some(Phoneme::FricativeSh));
        assert_eq!(ipa_to_phoneme("ʒ"), Some(Phoneme::FricativeZh));
        assert_eq!(ipa_to_phoneme("θ"), Some(Phoneme::FricativeTh));
        assert_eq!(ipa_to_phoneme("ð"), Some(Phoneme::FricativeDh));
        assert_eq!(ipa_to_phoneme("h"), Some(Phoneme::FricativeH));
        assert_eq!(ipa_to_phoneme("m"), Some(Phoneme::NasalM));
        assert_eq!(ipa_to_phoneme("n"), Some(Phoneme::NasalN));
        assert_eq!(ipa_to_phoneme("ŋ"), Some(Phoneme::NasalNg));
        assert_eq!(ipa_to_phoneme("tʃ"), Some(Phoneme::AffricateCh));
        assert_eq!(ipa_to_phoneme("dʒ"), Some(Phoneme::AffricateJ));
        assert_eq!(ipa_to_phoneme("ʔ"), Some(Phoneme::GlottalStop));
        assert_eq!(ipa_to_phoneme("ɾ"), Some(Phoneme::TapFlap));
        assert_eq!(ipa_to_phoneme("l"), Some(Phoneme::LateralL));
        assert_eq!(ipa_to_phoneme("ɹ"), Some(Phoneme::ApproximantR));
        assert_eq!(ipa_to_phoneme("w"), Some(Phoneme::ApproximantW));
        assert_eq!(ipa_to_phoneme("j"), Some(Phoneme::ApproximantJ));
    }

    #[test]
    fn test_unknown_symbol() {
        assert_eq!(ipa_to_phoneme("X"), None);
        assert_eq!(ipa_to_phoneme(""), None);
    }

    #[test]
    fn test_parse_hello() {
        let phonemes = parse_ipa_word("hɛˈloʊ");
        assert_eq!(phonemes.len(), 4);
        assert_eq!(phonemes[0], Phoneme::FricativeH);
        assert_eq!(phonemes[1], Phoneme::VowelOpenE);
        assert_eq!(phonemes[2], Phoneme::LateralL);
        assert_eq!(phonemes[3], Phoneme::DiphthongOU);
    }

    #[test]
    fn test_parse_strips_slashes() {
        let phonemes = parse_ipa_word("/hɛloʊ/");
        assert_eq!(phonemes.len(), 4);
    }

    #[test]
    fn test_parse_affricate() {
        let phonemes = parse_ipa_word("tʃɪp");
        assert_eq!(phonemes[0], Phoneme::AffricateCh);
        assert_eq!(phonemes[1], Phoneme::VowelNearI);
        assert_eq!(phonemes[2], Phoneme::PlosiveP);
    }

    #[test]
    fn test_roundtrip() {
        let original = alloc::vec![
            Phoneme::FricativeH,
            Phoneme::VowelOpenE,
            Phoneme::LateralL,
            Phoneme::DiphthongOU,
        ];
        let ipa = phonemes_to_ipa(&original);
        let roundtripped = parse_ipa_word(&ipa);
        assert_eq!(original, roundtripped);
    }

    #[test]
    fn test_serde_roundtrip_all_mapped() {
        // Verify every mapped phoneme survives IPA roundtrip
        let phonemes = [
            Phoneme::VowelA,
            Phoneme::VowelE,
            Phoneme::VowelI,
            Phoneme::VowelO,
            Phoneme::VowelU,
            Phoneme::VowelSchwa,
            Phoneme::VowelOpenO,
            Phoneme::VowelAsh,
            Phoneme::VowelNearI,
            Phoneme::VowelNearU,
            Phoneme::VowelOpenA,
            Phoneme::VowelOpenE,
            Phoneme::VowelCupV,
            Phoneme::VowelBird,
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
            Phoneme::AffricateCh,
            Phoneme::AffricateJ,
            Phoneme::GlottalStop,
            Phoneme::TapFlap,
            Phoneme::LateralL,
            Phoneme::ApproximantR,
            Phoneme::ApproximantW,
            Phoneme::ApproximantJ,
        ];
        for p in phonemes {
            let ipa = phoneme_to_ipa(&p).unwrap();
            let back = ipa_to_phoneme(ipa).unwrap();
            assert_eq!(p, back, "roundtrip failed for {ipa}");
        }
    }
}
