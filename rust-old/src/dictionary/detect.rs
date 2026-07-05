//! Script and language detection using varna's writing system metadata.
//!
//! Given a word, detect which writing system (script) it uses by checking
//! Unicode code points against varna's script ranges, then suggest candidate
//! languages that use that script.

use alloc::{string::String, vec::Vec};

/// Detects the ISO 15924 script code for a word based on its Unicode code points.
///
/// Examines the first non-ASCII alphabetic character to determine the script.
/// Returns `None` if no registered script matches.
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "varna")]
/// # {
/// use shabdakosh::dictionary::detect::detect_script;
///
/// assert_eq!(detect_script("hello"), Some("Latn"));
/// assert_eq!(detect_script("नमस्ते"), Some("Deva"));
/// # }
/// ```
#[must_use]
pub fn detect_script(word: &str) -> Option<&'static str> {
    // Check each character against registered scripts.
    // Use majority voting: count which script has the most matching codepoints.
    let script_codes = varna::script::all_codes();
    let mut best_code: Option<&'static str> = None;
    let mut best_count: usize = 0;

    for &code in script_codes {
        // Skip historical scripts for detection purposes.
        if code == "Xsux" || code == "Egyp" {
            continue;
        }
        let Some(script) = varna::script::by_code(code) else {
            continue;
        };
        let count = word
            .chars()
            .filter(|ch| script.contains_codepoint(*ch as u32))
            .count();
        if count > best_count {
            best_count = count;
            best_code = Some(code);
        }
    }

    if best_count > 0 { best_code } else { None }
}

/// Suggests candidate ISO 639 language codes for a word based on its script.
///
/// First detects the script, then returns all languages registered in varna
/// that use that script. Results are sorted alphabetically.
///
/// Returns an empty list if the script cannot be detected.
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "varna")]
/// # {
/// use shabdakosh::dictionary::detect::detect_language_hint;
///
/// let hints = detect_language_hint("hello");
/// assert!(hints.contains(&"en"));
/// # }
/// ```
#[must_use]
pub fn detect_language_hint(word: &str) -> Vec<&'static str> {
    let Some(script_code) = detect_script(word) else {
        return Vec::new();
    };

    let mut languages: Vec<&'static str> = varna::registry::REGISTERED
        .iter()
        .filter(|info| info.script_codes.contains(&script_code))
        .map(|info| info.code)
        .collect();

    languages.sort_unstable();
    languages
}

/// Detects the script and returns both the script code and a human-readable name.
///
/// Returns `None` if no script is detected.
#[must_use]
pub fn detect_script_name(word: &str) -> Option<(String, String)> {
    let code = detect_script(word)?;
    let script = varna::script::by_code(code)?;
    Some((
        alloc::string::ToString::to_string(code),
        alloc::string::ToString::to_string(script.name.as_ref()),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_latin() {
        assert_eq!(detect_script("hello"), Some("Latn"));
        assert_eq!(detect_script("world"), Some("Latn"));
    }

    #[test]
    fn test_detect_devanagari() {
        assert_eq!(detect_script("नमस्ते"), Some("Deva"));
    }

    #[test]
    fn test_detect_arabic() {
        assert_eq!(detect_script("مرحبا"), Some("Arab"));
    }

    #[test]
    fn test_detect_cyrillic() {
        assert_eq!(detect_script("привет"), Some("Cyrl"));
    }

    #[test]
    fn test_detect_cjk() {
        assert_eq!(detect_script("你好"), Some("Hani"));
    }

    #[test]
    fn test_detect_greek() {
        assert_eq!(detect_script("γεια"), Some("Grek"));
    }

    #[test]
    fn test_detect_empty() {
        assert_eq!(detect_script(""), None);
    }

    #[test]
    fn test_detect_digits_only() {
        assert_eq!(detect_script("12345"), None);
    }

    #[test]
    fn test_language_hint_latin() {
        let hints = detect_language_hint("hello");
        assert!(hints.contains(&"en"));
        assert!(hints.contains(&"es"));
        assert!(hints.contains(&"fr"));
        assert!(hints.contains(&"de"));
    }

    #[test]
    fn test_language_hint_devanagari() {
        let hints = detect_language_hint("नमस्ते");
        assert!(hints.contains(&"hi"));
        assert!(hints.contains(&"sa"));
    }

    #[test]
    fn test_language_hint_empty() {
        let hints = detect_language_hint("");
        assert!(hints.is_empty());
    }

    #[test]
    fn test_detect_script_name() {
        let (code, name) = detect_script_name("hello").unwrap();
        assert_eq!(code, "Latn");
        assert_eq!(name, "Latin");
    }

    #[test]
    fn test_detect_script_name_devanagari() {
        let (code, name) = detect_script_name("नमस्ते").unwrap();
        assert_eq!(code, "Deva");
        assert_eq!(name, "Devanagari");
    }
}
