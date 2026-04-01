//! Basic usage of shabdakosh — pronunciation dictionary lookup.
//!
//! Run with: `cargo run --example basic --features full`

fn main() {
    // --- English dictionary ---
    let dict = shabdakosh::PronunciationDict::english();
    println!("English dictionary: {} entries", dict.len());
    println!("Language: {:?}", dict.language());

    // Simple lookup
    if let Some(phonemes) = dict.lookup("hello") {
        let ipa = shabdakosh::ipa::phonemes_to_ipa(phonemes);
        println!("hello -> /{ipa}/");
    }

    // Variant pronunciations (heteronyms)
    if let Some(entry) = dict.lookup_entry("read") {
        println!("read has {} pronunciations:", entry.len());
        for p in entry.all() {
            let ipa = shabdakosh::ipa::phonemes_to_ipa(p.phonemes());
            println!("  /{ipa}/ (freq: {:?})", p.frequency());
        }
    }

    // --- User overlay ---
    let mut dict = shabdakosh::PronunciationDict::english_minimal();
    dict.insert_user(
        "agnos",
        &[
            svara::phoneme::Phoneme::VowelAsh,
            svara::phoneme::Phoneme::PlosiveG,
            svara::phoneme::Phoneme::NasalN,
            svara::phoneme::Phoneme::VowelO,
            svara::phoneme::Phoneme::FricativeS,
        ],
    );
    if let Some(phonemes) = dict.lookup("agnos") {
        let ipa = shabdakosh::ipa::phonemes_to_ipa(phonemes);
        println!("agnos -> /{ipa}/ (user overlay)");
    }

    // --- Import/export ---
    let cmudict_text = "cat  K AE1 T\ndog  D AO1 G\n";
    let imported = shabdakosh::dictionary::format::parse_cmudict(cmudict_text).unwrap();
    println!("Imported {} entries from CMUdict text", imported.len());

    let ipa_text = shabdakosh::dictionary::format::to_ipa(&imported);
    println!("IPA export:\n{ipa_text}");

    // --- Varna integration ---
    #[cfg(feature = "varna")]
    {
        // Script detection
        let scripts = [("hello", "Latin"), ("नमस्ते", "Devanagari"), ("مرحبا", "Arabic")];
        for (word, expected) in scripts {
            if let Some((_, name)) = shabdakosh::detect_script_name(word) {
                println!("{word} -> {name} script");
                assert_eq!(name, expected);
            }
        }

        // Lexicon ingestion
        let lexicon = varna::lexicon::swadesh::by_code("es").unwrap();
        let es_dict = shabdakosh::PronunciationDict::from_lexicon(&lexicon);
        println!(
            "Spanish seed dictionary: {} entries (language: {:?})",
            es_dict.len(),
            es_dict.language()
        );

        // Validation
        let mut test_dict = shabdakosh::PronunciationDict::new().with_language("en");
        test_dict.insert(
            "pat",
            &[
                svara::phoneme::Phoneme::PlosiveP,
                svara::phoneme::Phoneme::VowelAsh,
                svara::phoneme::Phoneme::PlosiveT,
            ],
        );
        if let Some(report) = test_dict.validate() {
            println!("Validation: {}", if report.is_valid() { "PASS" } else { "FAIL" });
        }
    }
}
