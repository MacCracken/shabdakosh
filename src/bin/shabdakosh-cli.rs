//! Dictionary builder CLI for shabdakosh.
//!
//! Subcommands:
//!   import   — Import from CMUdict/IPA/PLS/JSON/binary format
//!   export   — Export to CMUdict/IPA/JSON/PLS/binary format
//!   merge    — Merge two dictionaries
//!   validate — Validate entries against the English dictionary
//!   diff     — Compare two dictionaries
//!   coverage — Analyze text corpus coverage
//!   info     — Show dictionary statistics
//!
//! Usage: shabdakosh-cli <command> [args...]

use std::path::Path;
use std::process;

use shabdakosh::PronunciationDict;
use shabdakosh::dictionary::format;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }

    let result = match args[1].as_str() {
        "import" => cmd_import(&args[2..]),
        "export" => cmd_export(&args[2..]),
        "merge" => cmd_merge(&args[2..]),
        "validate" => cmd_validate(&args[2..]),
        "diff" => cmd_diff(&args[2..]),
        "coverage" => cmd_coverage(&args[2..]),
        "info" => cmd_info(&args[2..]),
        "help" | "--help" | "-h" => {
            print_usage();
            Ok(())
        }
        other => {
            eprintln!("unknown command: {other}");
            print_usage();
            process::exit(1);
        }
    };

    if let Err(e) = result {
        eprintln!("error: {e}");
        process::exit(1);
    }
}

fn print_usage() {
    eprintln!(
        "\
shabdakosh-cli — Dictionary builder tool

Usage: shabdakosh-cli <command> [args...]

Commands:
  import  <format> <input>           Import a dictionary file
  export  <format> <input> <output>  Export to a different format
  merge   <file1> <file2> <output>   Merge two dictionaries
  diff    <file1> <file2>            Show differences between dictionaries
  coverage <dict> <text-file>        Analyze text corpus coverage
  info    [file]                     Show dictionary info (default: built-in English)

Formats: cmudict, ipa, json, pls, binary

Examples:
  shabdakosh-cli info
  shabdakosh-cli import cmudict my_dict.txt
  shabdakosh-cli export binary my_dict.txt my_dict.bin
  shabdakosh-cli coverage my_dict.txt corpus.txt
  shabdakosh-cli diff dict1.txt dict2.txt"
    );
}

fn detect_format(path: &str) -> &str {
    match Path::new(path).extension().and_then(|e| e.to_str()) {
        Some("json") => "json",
        Some("pls" | "xml") => "pls",
        Some("bin" | "binary") => "binary",
        Some("ipa") => "ipa",
        _ => "cmudict",
    }
}

fn load_dict(format: &str, path: &str) -> Result<PronunciationDict, String> {
    let data = std::fs::read_to_string(path).map_err(|e| format!("failed to read {path}: {e}"))?;

    match format {
        "cmudict" => format::parse_cmudict(&data).map_err(|e| e.to_string()),
        "ipa" => format::parse_ipa(&data).map_err(|e| e.to_string()),
        #[cfg(feature = "json")]
        "json" => format::from_json(&data).map_err(|e| e.to_string()),
        "pls" => format::pls::parse_pls(&data).map_err(|e| e.to_string()),
        #[cfg(feature = "binary")]
        "binary" => {
            let bytes = std::fs::read(path).map_err(|e| format!("failed to read {path}: {e}"))?;
            format::binary::from_binary(&bytes).map_err(|e| e.to_string())
        }
        other => Err(format!("unsupported format: {other}")),
    }
}

fn save_dict(dict: &PronunciationDict, format: &str, path: &str) -> Result<(), String> {
    match format {
        "cmudict" => {
            let data = format::to_cmudict(dict);
            std::fs::write(path, data).map_err(|e| format!("failed to write {path}: {e}"))
        }
        "ipa" => {
            let data = format::to_ipa(dict);
            std::fs::write(path, data).map_err(|e| format!("failed to write {path}: {e}"))
        }
        #[cfg(feature = "json")]
        "json" => {
            let data = format::to_json(dict).map_err(|e| e.to_string())?;
            std::fs::write(path, data).map_err(|e| format!("failed to write {path}: {e}"))
        }
        "pls" => {
            let lang = dict.language().unwrap_or("en-US");
            let data = format::pls::to_pls(dict, lang);
            std::fs::write(path, data).map_err(|e| format!("failed to write {path}: {e}"))
        }
        #[cfg(feature = "binary")]
        "binary" => {
            format::binary::save_binary_file(dict, Path::new(path)).map_err(|e| e.to_string())
        }
        other => Err(format!("unsupported output format: {other}")),
    }
}

fn cmd_import(args: &[String]) -> Result<(), String> {
    if args.len() < 2 {
        return Err("usage: import <format> <input>".into());
    }
    let format = &args[0];
    let input = &args[1];

    let dict = load_dict(format, input)?;
    println!(
        "imported {} entries from {input} ({format} format)",
        dict.len()
    );
    if let Some(lang) = dict.language() {
        println!("language: {lang}");
    }
    Ok(())
}

fn cmd_export(args: &[String]) -> Result<(), String> {
    if args.len() < 3 {
        return Err("usage: export <output-format> <input> <output>".into());
    }
    let out_format = &args[0];
    let input = &args[1];
    let output = &args[2];

    let in_format = detect_format(input);
    let dict = load_dict(in_format, input)?;
    save_dict(&dict, out_format, output)?;
    println!(
        "exported {} entries to {output} ({out_format} format)",
        dict.len()
    );
    Ok(())
}

fn cmd_validate(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return Err("usage: validate <dict-file>".into());
    }
    let fmt = detect_format(&args[0]);
    let dict = load_dict(fmt, &args[0])?;

    // Check for empty entries or entries with no phonemes.
    let mut issues = 0_usize;
    for (word, entry) in dict.entries() {
        if entry.all().is_empty() {
            println!("  empty entry: {word}");
            issues += 1;
        }
        for pron in entry.all() {
            if pron.phonemes().is_empty() {
                println!("  empty phonemes: {word}");
                issues += 1;
            }
        }
    }

    if issues == 0 {
        println!("validated {} entries: no issues found", dict.len());
    } else {
        println!("validated {} entries: {issues} issue(s) found", dict.len());
    }
    Ok(())
}

fn cmd_merge(args: &[String]) -> Result<(), String> {
    if args.len() < 3 {
        return Err("usage: merge <file1> <file2> <output>".into());
    }
    let f1 = detect_format(&args[0]);
    let f2 = detect_format(&args[1]);
    let out_format = detect_format(&args[2]);

    let mut dict1 = load_dict(f1, &args[0])?;
    let dict2 = load_dict(f2, &args[1])?;

    let before = dict1.len();
    dict1.merge(&dict2);
    println!(
        "merged: {} + {} => {} entries",
        before,
        dict2.len(),
        dict1.len()
    );

    save_dict(&dict1, out_format, &args[2])?;
    println!("saved to {}", args[2]);
    Ok(())
}

fn cmd_diff(args: &[String]) -> Result<(), String> {
    if args.len() < 2 {
        return Err("usage: diff <file1> <file2>".into());
    }
    let f1 = detect_format(&args[0]);
    let f2 = detect_format(&args[1]);

    let dict1 = load_dict(f1, &args[0])?;
    let dict2 = load_dict(f2, &args[1])?;

    let d = shabdakosh::dictionary::diff(&dict1, &dict2);

    if d.is_empty() {
        println!("dictionaries are identical");
    } else {
        if !d.added.is_empty() {
            println!("added ({}):", d.added.len());
            for word in &d.added {
                println!("  + {word}");
            }
        }
        if !d.removed.is_empty() {
            println!("removed ({}):", d.removed.len());
            for word in &d.removed {
                println!("  - {word}");
            }
        }
        if !d.changed.is_empty() {
            println!("changed ({}):", d.changed.len());
            for word in &d.changed {
                println!("  ~ {word}");
            }
        }
        println!("total differences: {}", d.len());
    }
    Ok(())
}

fn cmd_coverage(args: &[String]) -> Result<(), String> {
    if args.len() < 2 {
        return Err("usage: coverage <dict-file> <text-file>".into());
    }
    let dict_format = detect_format(&args[0]);
    let dict = load_dict(dict_format, &args[0])?;
    let text =
        std::fs::read_to_string(&args[1]).map_err(|e| format!("failed to read text: {e}"))?;

    let report = dict.coverage(&text);
    println!("tokens:    {}", report.total_tokens);
    println!("covered:   {}", report.covered_tokens);
    println!("coverage:  {:.1}%", report.coverage_pct());
    if !report.uncovered_words.is_empty() {
        println!("uncovered ({}):", report.uncovered_count());
        for word in &report.uncovered_words {
            println!("  {word}");
        }
    }
    Ok(())
}

fn cmd_info(args: &[String]) -> Result<(), String> {
    let dict = if args.is_empty() {
        println!("built-in English dictionary:");
        PronunciationDict::english()
    } else {
        let format = detect_format(&args[0]);
        let d = load_dict(format, &args[0])?;
        println!("{}:", args[0]);
        d
    };

    println!("  entries:      {}", dict.len());
    println!("  user entries: {}", dict.user_len());
    if let Some(lang) = dict.language() {
        println!("  language:     {lang}");
    }

    // Count entries with multiple pronunciations.
    let heteronyms: usize = dict.entries().values().filter(|e| e.len() > 1).count();
    println!("  heteronyms:   {heteronyms}");

    Ok(())
}
