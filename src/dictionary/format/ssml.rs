//! SSML `<phoneme>` tag parsing and generation.
//!
//! Handles the SSML phoneme element: `<phoneme alphabet="ipa" ph="IPA">word</phoneme>`.

use alloc::string::String;
use alloc::vec::Vec;

use svara::phoneme::Phoneme;

use crate::dictionary::format::{xml_escape, xml_unescape};
use crate::error::{Result, ShabdakoshError};
use crate::ipa;

/// Extracts the word and phonemes from an SSML `<phoneme>` tag.
///
/// Expected format: `<phoneme alphabet="ipa" ph="IPA">word</phoneme>`
///
/// # Errors
///
/// Returns [`ShabdakoshError::DictParseError`] if the tag is malformed or
/// the alphabet is not `ipa`.
pub fn parse_ssml_phoneme(tag: &str) -> Result<(String, Vec<Phoneme>)> {
    let tag = tag.trim();

    if !tag.starts_with("<phoneme") {
        return Err(ShabdakoshError::DictParseError(
            "not a <phoneme> tag".into(),
        ));
    }

    // Extract alphabet attribute if present
    if let Some(start) = tag.find("alphabet=\"") {
        let val_start = start + "alphabet=\"".len();
        if let Some(val_end) = tag[val_start..].find('"') {
            let alphabet = &tag[val_start..val_start + val_end];
            if alphabet != "ipa" {
                return Err(ShabdakoshError::DictParseError(alloc::format!(
                    "unsupported SSML alphabet: '{alphabet}'"
                )));
            }
        }
    }

    // Extract ph attribute
    let ph_start = tag
        .find("ph=\"")
        .ok_or_else(|| ShabdakoshError::DictParseError("missing ph attribute".into()))?;
    let val_start = ph_start + "ph=\"".len();
    let val_end = tag[val_start..]
        .find('"')
        .ok_or_else(|| ShabdakoshError::DictParseError("unclosed ph attribute".into()))?;
    let ipa_str = &tag[val_start..val_start + val_end];

    let phonemes = ipa::parse_ipa_word(ipa_str);
    if phonemes.is_empty() {
        return Err(ShabdakoshError::DictParseError(alloc::format!(
            "no phonemes parsed from ph=\"{ipa_str}\""
        )));
    }

    // Extract text content (the word) between > and </phoneme>
    let content_start = tag
        .find('>')
        .ok_or_else(|| ShabdakoshError::DictParseError("malformed tag".into()))?
        + 1;
    let content_end = tag
        .find("</phoneme>")
        .ok_or_else(|| ShabdakoshError::DictParseError("missing </phoneme> close tag".into()))?;
    let word = xml_unescape(tag[content_start..content_end].trim());

    Ok((word, phonemes))
}

/// Emits an SSML `<phoneme>` tag for the given word and phonemes.
///
/// Uses the IPA alphabet.
#[must_use]
pub fn to_ssml_phoneme(word: &str, phonemes: &[Phoneme]) -> String {
    let ipa_str = ipa::phonemes_to_ipa(phonemes);
    alloc::format!(
        "<phoneme alphabet=\"ipa\" ph=\"{ipa_str}\">{}</phoneme>",
        xml_escape(word)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ssml_phoneme() {
        let tag = r#"<phoneme alphabet="ipa" ph="hɛloʊ">hello</phoneme>"#;
        let (word, phonemes) = parse_ssml_phoneme(tag).unwrap();
        assert_eq!(word, "hello");
        assert_eq!(phonemes.len(), 4);
    }

    #[test]
    fn test_roundtrip() {
        let phonemes = alloc::vec![
            Phoneme::PlosiveK,
            Phoneme::VowelAsh,
            Phoneme::PlosiveT,
        ];
        let tag = to_ssml_phoneme("cat", &phonemes);
        let (word, parsed) = parse_ssml_phoneme(&tag).unwrap();
        assert_eq!(word, "cat");
        assert_eq!(parsed, phonemes);
    }

    #[test]
    fn test_parse_missing_ph() {
        let tag = r#"<phoneme alphabet="ipa">hello</phoneme>"#;
        assert!(parse_ssml_phoneme(tag).is_err());
    }

    #[test]
    fn test_parse_wrong_alphabet() {
        let tag = r#"<phoneme alphabet="x-sampa" ph="hEloU">hello</phoneme>"#;
        assert!(parse_ssml_phoneme(tag).is_err());
    }

    #[test]
    fn test_xml_escape_in_output() {
        let tag = to_ssml_phoneme("a&b", &[Phoneme::VowelA]);
        assert!(tag.contains("a&amp;b"));
    }
}
