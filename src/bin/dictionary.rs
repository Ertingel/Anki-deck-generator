// cargo run --bin dictionary

use std::{cmp::Ordering, collections::HashMap, fs, path::Path};

use anki_utill::{
    dict::{
        dict_parser::{convert_data, convert_word_data, parse_directory},
        jitendex::jitendex_word::JitendexWord,
        jmnedict::jmnedict_entry::JmnedictEntry,
    },
    entry::Word,
    japanese::JapaneseStr,
};
use regex::Regex;

fn main() {
    // Define output file paths
    let wordlist_save_path = "./result/wordlist.json";
    let kanjilist_save_path = "./result/kanjilist.json";

    // Parse dictionary entries from jmnedict directory
    let entries = parse_directory::<JmnedictEntry>(Path::new("./input/dictionaries")).unwrap();
    let (kanji, words) = convert_data(&entries);

    // Parse example sentences from jitendex directory
    println!("Parsing examples:");
    let entries = parse_directory::<JitendexWord>(Path::new("./input/examples")).unwrap();
    let exampes = convert_word_data(&kanji, &entries);

    println!("Filtering words...");
    // Filter words based on the filter_words function
    let before_count = words.len();
    let words: HashMap<String, Word> = words
        .into_iter()
        .filter(|(_, word)| filter_words(word))
        .collect();

    let mut words = filter_overlapping(words);

    // Add examples to filtered words if they exist in the examples data
    for (_, word) in words.iter_mut() {
        if let Some(example) = exampes.get(&word.furigana) {
            word.examples = example.examples.clone();
        }
    }

    // Report filtering statistics
    println!(
        "Filtered {}/{} ({:.1}%)\n",
        words.len(),
        before_count,
        ((words.len() as f32 / before_count as f32) * 1000.0).round() / 10.0
    );

    // Save filtered word data to JSON file
    println!("Saving result to {}\n", wordlist_save_path);

    let save_data: Vec<String> = words
        .iter()
        .map(|(k, v)| format!("	\"{}\": {}", k, serde_json::to_string(&v).unwrap()))
        .collect();

    fs::write(
        wordlist_save_path,
        format!("{{\n{}\n}}", save_data.join(",\n")),
    )
    .unwrap();

    // Save kanji data to JSON file
    println!("Saving result to {}\n", kanjilist_save_path);

    let save_data: Vec<String> = kanji
        .iter()
        .map(|(k, v)| format!("	\"{}\": {}", k, serde_json::to_string(&v).unwrap()))
        .collect();

    fs::write(
        kanjilist_save_path,
        format!("{{\n{}\n}}", save_data.join(",\n")),
    )
    .unwrap();
}

/// Filters words based on JLPT tags and compound status.
/// Returns true for:
/// - Words with N1/N2 tags that are compounds (have 'comp' tag)
/// - Words with N5, N4, or N3 tags
fn filter_words(word: &Word) -> bool {
    if word.glossary.is_empty() {
        return false;
    }

    let regex = Regex::new("〇|０|１|２|３|４|５|６|７|８|９").unwrap();
    if regex.is_match(&word.furigana) {
        return false;
    }

    let tags = word.get_all_tags();

    // Check if the word is a compound and has higher level JLPT tag
    if (tags.contains("JLPT-N2") || tags.contains("JLPT-N1")) && tags.contains("comp") {
        return true;
    }

    // Check for lower JLPT levels
    tags.contains("JLPT-N5") || tags.contains("JLPT-N4") || tags.contains("JLPT-N3")
}

fn filter_overlapping(words: HashMap<String, Word>) -> HashMap<String, Word> {
    let mut out: HashMap<String, Word> = HashMap::new();

    for (key, value1) in words {
        let key = key.to_kanji();

        if let Some(value2) = out.get(&key) {
            let jlpt = get_jlpt_level(&value1)
                .unwrap_or_default()
                .cmp(&get_jlpt_level(value2).unwrap_or_default());

            let news = get_newsnk(value2)
                .unwrap_or(u8::MAX)
                .cmp(&get_newsnk(&value1).unwrap_or(u8::MAX));

            let example = value1.examples.len().cmp(&value2.examples.len());

            let len = value1.glossary.len().cmp(&value2.glossary.len());

            let freq1 = value1
                .glossary
                .iter()
                .map(|gloss| gloss.order)
                .min()
                .unwrap_or(i32::MAX);

            let freq2 = value2
                .glossary
                .iter()
                .map(|gloss| gloss.order)
                .min()
                .unwrap_or(i32::MAX);

            let freq = freq1.cmp(&freq2);

            match (jlpt, news, example, len, freq) {
                (Ordering::Greater, _, _, _, _)
                | (Ordering::Equal, Ordering::Greater, _, _, _)
                | (Ordering::Equal, Ordering::Equal, Ordering::Greater, _, _)
                | (Ordering::Equal, Ordering::Equal, Ordering::Equal, Ordering::Greater, _)
                | (
                    Ordering::Equal,
                    Ordering::Equal,
                    Ordering::Equal,
                    Ordering::Equal,
                    Ordering::Greater,
                ) => {
                    out.remove_entry(&key);
                    out.insert(key, value1);
                }
                _ => {}
            }
        } else {
            out.insert(key, value1);
        }
    }

    out.into_values()
        .map(|value| (value.furigana.clone(), value))
        .collect()
}

fn get_newsnk(note: &Word) -> Option<u8> {
    let tags = note.get_all_tags();

    let regex = Regex::new(r"^news(\d+)k$").unwrap();
    tags.iter()
        .filter_map(|tag| {
            let mat = regex.captures(tag)?;
            mat.get(1)?.as_str().parse::<u8>().ok()
        })
        .min()
}

/// Determine the JLPT level from note tags
/// Returns Some(level) if a JLPT tag is found, None otherwise.
fn get_jlpt_level(note: &Word) -> Option<u8> {
    // Check each tag for JLPT levels in descending order
    let tags = note.get_all_tags();
    if tags.contains(&"JLPT-N1") {
        Some(1)
    } else if tags.contains(&"JLPT-N2") {
        Some(2)
    } else if tags.contains(&"JLPT-N3") {
        Some(3)
    } else if tags.contains(&"JLPT-N4") {
        Some(4)
    } else if tags.contains(&"JLPT-N5") {
        Some(5)
    } else {
        None
    }
}
