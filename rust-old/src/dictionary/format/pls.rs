//! W3C PLS (Pronunciation Lexicon Specification) import/export.
//!
//! Parses and emits PLS XML documents with IPA phoneme transcriptions.
//! Only the `ipa` alphabet is supported.

use alloc::string::String;

use crate::dictionary::PronunciationDict;
use crate::dictionary::entry::{DictEntry, Pronunciation};
use crate::dictionary::format::{xml_escape, xml_unescape};
use crate::error::{Result, ShabdakoshError};
use crate::ipa;

/// Parses a W3C PLS XML string into a [`PronunciationDict`].
///
/// Only the `ipa` alphabet is supported. Other alphabet values produce an error.
///
/// # Errors
///
/// Returns [`ShabdakoshError::DictParseError`] on malformed XML or unsupported alphabet.
pub fn parse_pls(input: &str) -> Result<PronunciationDict> {
    // Validate alphabet
    if let Some(alphabet) = extract_attr(input, "alphabet").filter(|a| *a != "ipa") {
        return Err(ShabdakoshError::DictParseError(alloc::format!(
            "unsupported PLS alphabet: '{alphabet}' (only 'ipa' is supported)"
        )));
    }

    let mut dict = PronunciationDict::new();
    let mut pos = 0;

    while let Some(lexeme_start) = input[pos..].find("<lexeme") {
        let lexeme_start = pos + lexeme_start;
        let Some(lexeme_end) = input[lexeme_start..].find("</lexeme>") else {
            break;
        };
        let lexeme_end = lexeme_start + lexeme_end + "</lexeme>".len();
        let lexeme_xml = &input[lexeme_start..lexeme_end];

        // Extract grapheme
        let grapheme = extract_element_text(lexeme_xml, "grapheme");
        let Some(word) = grapheme else {
            pos = lexeme_end;
            continue;
        };
        let word = xml_unescape(&word);

        // Extract all phoneme elements
        let mut pronunciations = alloc::vec::Vec::new();
        let mut phoneme_pos = 0;
        while let Some(ph) = extract_element_text_from(&lexeme_xml[phoneme_pos..], "phoneme") {
            let phonemes = ipa::parse_ipa_word(&ph);
            if !phonemes.is_empty() {
                pronunciations.push(Pronunciation::new(phonemes));
            }
            // Advance past this phoneme element
            if let Some(end) = lexeme_xml[phoneme_pos..].find("</phoneme>") {
                phoneme_pos += end + "</phoneme>".len();
            } else {
                break;
            }
        }

        if let Some(entry) = DictEntry::from_pronunciations(pronunciations) {
            dict.insert_entry(&word, entry);
        }

        pos = lexeme_end;
    }

    Ok(dict)
}

/// Serializes a [`PronunciationDict`] to W3C PLS XML format.
///
/// Uses the IPA alphabet. Only base entries are exported.
#[must_use]
pub fn to_pls(dict: &PronunciationDict, lang: &str) -> String {
    use core::fmt::Write;

    let mut out = String::new();
    out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    let _ = writeln!(
        out,
        "<lexicon version=\"1.0\" xmlns=\"http://www.w3.org/2005/01/pronunciation-lexicon\" alphabet=\"ipa\" xml:lang=\"{lang}\">"
    );

    let mut words: alloc::vec::Vec<&str> = dict.entries().keys().map(|s| s.as_str()).collect();
    words.sort_unstable();

    for word in words {
        let Some(entry) = dict.entries().get(word) else {
            continue;
        };
        out.push_str("  <lexeme>\n");
        let _ = writeln!(out, "    <grapheme>{}</grapheme>", xml_escape(word));
        for pron in entry.all() {
            let ipa_str = ipa::phonemes_to_ipa(pron.phonemes());
            let _ = writeln!(out, "    <phoneme>{ipa_str}</phoneme>");
        }
        out.push_str("  </lexeme>\n");
    }

    out.push_str("</lexicon>\n");
    out
}

/// Serializes a dictionary to PLS including user overlay entries.
#[must_use]
pub fn to_pls_with_user(dict: &PronunciationDict, lang: &str) -> String {
    use core::fmt::Write;

    let mut out = String::new();
    out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    let _ = writeln!(
        out,
        "<lexicon version=\"1.0\" xmlns=\"http://www.w3.org/2005/01/pronunciation-lexicon\" alphabet=\"ipa\" xml:lang=\"{lang}\">"
    );

    // Collect all effective entries (user overrides base)
    let mut all_words: alloc::collections::BTreeSet<&str> = alloc::collections::BTreeSet::new();
    for word in dict.entries().keys() {
        all_words.insert(word);
    }
    for word in dict.user_entries().keys() {
        all_words.insert(word);
    }

    for word in all_words {
        let Some(entry) = dict.lookup_entry(word) else {
            continue;
        };
        out.push_str("  <lexeme>\n");
        let _ = writeln!(out, "    <grapheme>{}</grapheme>", xml_escape(word));
        for pron in entry.all() {
            let ipa_str = ipa::phonemes_to_ipa(pron.phonemes());
            let _ = writeln!(out, "    <phoneme>{ipa_str}</phoneme>");
        }
        out.push_str("  </lexeme>\n");
    }

    out.push_str("</lexicon>\n");
    out
}

// --- XML helpers ---

/// Extracts an attribute value from the first occurrence of `attr="value"` in the input.
fn extract_attr<'a>(input: &'a str, attr: &str) -> Option<&'a str> {
    let pattern = alloc::format!("{attr}=\"");
    let start = input.find(&pattern)?;
    let value_start = start + pattern.len();
    let value_end = input[value_start..].find('"')? + value_start;
    Some(&input[value_start..value_end])
}

/// Extracts text content from the first `<tag>text</tag>` in the input.
fn extract_element_text(input: &str, tag: &str) -> Option<String> {
    extract_element_text_from(input, tag)
}

/// Extracts text content from the first `<tag>text</tag>` starting from the given position.
fn extract_element_text_from(input: &str, tag: &str) -> Option<String> {
    let open = alloc::format!("<{tag}>");
    let close = alloc::format!("</{tag}>");

    let start = input.find(&open)?;
    let text_start = start + open.len();
    let text_end = input[text_start..].find(&close)? + text_start;
    Some(alloc::string::ToString::to_string(
        input[text_start..text_end].trim(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_pls() {
        let pls = r#"<?xml version="1.0" encoding="UTF-8"?>
<lexicon version="1.0" xmlns="http://www.w3.org/2005/01/pronunciation-lexicon"
         alphabet="ipa" xml:lang="en-US">
  <lexeme>
    <grapheme>hello</grapheme>
    <phoneme>hɛloʊ</phoneme>
  </lexeme>
</lexicon>"#;
        let dict = parse_pls(pls).unwrap();
        assert_eq!(dict.len(), 1);
        assert!(dict.lookup("hello").is_some());
    }

    #[test]
    fn test_parse_pls_multi_phoneme() {
        let pls = r#"<?xml version="1.0"?>
<lexicon version="1.0" xmlns="http://www.w3.org/2005/01/pronunciation-lexicon"
         alphabet="ipa" xml:lang="en-US">
  <lexeme>
    <grapheme>read</grapheme>
    <phoneme>ɹid</phoneme>
    <phoneme>ɹɛd</phoneme>
  </lexeme>
</lexicon>"#;
        let dict = parse_pls(pls).unwrap();
        let all = dict.lookup_all("read").unwrap();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_parse_pls_wrong_alphabet() {
        let pls = r#"<lexicon alphabet="x-sampa" xml:lang="en-US"></lexicon>"#;
        assert!(parse_pls(pls).is_err());
    }

    #[test]
    fn test_pls_roundtrip() {
        let mut dict = PronunciationDict::new();
        dict.insert(
            "cat",
            &[
                svara::phoneme::Phoneme::PlosiveK,
                svara::phoneme::Phoneme::VowelAsh,
                svara::phoneme::Phoneme::PlosiveT,
            ],
        );
        let pls = to_pls(&dict, "en-US");
        let dict2 = parse_pls(&pls).unwrap();
        assert!(dict2.lookup("cat").is_some());
    }

    #[test]
    fn test_pls_xml_escape() {
        let mut dict = PronunciationDict::new();
        dict.insert("a&b", &[svara::phoneme::Phoneme::VowelA]);
        let pls = to_pls(&dict, "en-US");
        assert!(pls.contains("a&amp;b"));
    }
}
