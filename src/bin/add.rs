// cargo run --bin add

use std::{
    collections::{HashMap, HashSet},
    fs,
};

use anki_utill::{
    anki::{anki_connect::AnkiConnect, anki_note::AnkiNote},
    entry::{Glossary, Word},
};
use regex::Regex;

/// Main function that loads word data from a JSON file and writes it to Anki notes.
/// Loads words from './result/wordlist.json' and processes them into Anki cards.
fn main() {
    let wordlist_save_path = "./result/wordlist.json";

    println!("Loading words from {}", wordlist_save_path);
    let data = fs::read_to_string(wordlist_save_path).unwrap();

    let words = serde_json::from_str(&data).unwrap();

    write_words(&words);
}

/// Handles writing of words to Anki by first updating existing notes then adding new ones.
/// Updates are done before additions to ensure any necessary modifications are made first.
fn write_words(words: &HashMap<String, Word>) {
    let anki = AnkiConnect::new("http://127.0.0.1:8765".into(), None).unwrap();

    println!("\nGetting Notes info.");
    let notes = anki
        .notes_info(
            &anki
                .find_notes("\"deck:My Deck 4.0\" \"note:JP Card V4\"")
                .unwrap(),
        )
        .unwrap();

    update_words(words, &notes, &anki);
    add_words(words, &notes, &anki);
}

/// Updates existing Anki notes with new word data while managing note states and tags.
/// For each note:
/// - Extracts the word from the first field
/// - Checks if word exists in provided `words` map
/// - Updates fields (word, meaning, examples) if needed
/// - Manages tags by removing old ones and adding new ones
/// - Suspends notes that don't match any word
fn update_words(words: &HashMap<String, Word>, notes: &[AnkiNote], anki: &AnkiConnect) {
    println!("Updating Notes:");
    let re = Regex::new(r"] ").unwrap();

    for (count, note) in notes.iter().enumerate() {
        // Progress tracking every 5% of total notes
        if count % (notes.len() / 20) == 0 {
            println!(
                "  {:>3}% Notes",
                ((count as f32 / notes.len() as f32) * 100.0).round()
            );
        }

        // Extract word from first field
        let note_id = note.noteId.unwrap();
        let note_cards = &note.cards.clone().unwrap();
        let word = re
            .replace_all(note.fields.get("1 Word").unwrap(), "]")
            .to_string();

        if let Some(word_data) = words.get(&word) {
            // Prepare fields to update
            let mut fields: HashMap<String, String> = HashMap::new();

            // Update word field if changed
            if note.fields["1 Word"] != word_data.furigana {
                fields.insert("1 Word".to_owned(), word_data.furigana.clone());
            }

            // Update meaning field if changed
            let meaning = get_meaning(word_data);
            if note.fields["2 Meaning"] != meaning {
                fields.insert("2 Meaning".to_owned(), meaning);
            }

            // Update examples field if empty
            let examples = get_examples(word_data);
            if note.fields["4 Sentences"].is_empty() {
                fields.insert("4 Sentences".to_owned(), examples);
            }

            // Update note fields in Anki
            anki.update_note_fields(note_id, &fields);

            // Manage tags: remove old ones and add new ones
            let word_tags = word_data.get_all_tags();

            note.tags
                .iter()
                .filter(|tag| !word_tags.contains(tag.as_str()))
                .for_each(|tag| anki.remove_tags(&[note_id], tag));

            word_tags
                .iter()
                .filter(|tag| !note.tags.contains(&(**tag).to_owned()))
                .for_each(|tag| anki.add_tags(&[note_id], tag));

            // Unsuspend note if updated
            let _ = anki.unsuspend(note_cards);
        } else {
            // No matching word found, suspend the note
            anki.suspend(note_cards).unwrap();
        }
    }
}

/// Adds new Anki notes for words not already present in the collection.
/// Skips adding if:
/// - The word already exists in Anki
/// - The note was recently created
fn add_words(words: &HashMap<String, Word>, notes: &[AnkiNote], anki: &AnkiConnect) {
    println!("Adding Notes:");

    // Extract existing words from Anki notes
    let re = Regex::new(r"] ").unwrap();
    let notes: HashSet<String> = notes
        .iter()
        .map(|note| {
            re.replace_all(note.fields.get("1 Word").unwrap(), "]")
                .to_string()
        })
        .collect();

    for (count, word) in words.values().enumerate() {
        // Progress tracking every 5% of total note
        if count % (notes.len() / 20) == 0 {
            println!(
                "  {:>3}% Notes",
                ((count as f32 / notes.len() as f32) * 100.0).round()
            );
        }

        // Skip if word already exists in Anki or was recently added
        if notes.contains(&word.furigana) {
            continue;
        }

        // Prepare fields for new note
        let mut fields: HashMap<String, String> = HashMap::new();

        fields.insert("1 Word".to_owned(), word.furigana.clone());
        fields.insert("2 Meaning".to_owned(), get_meaning(word));
        fields.insert("4 Sentences".to_owned(), get_examples(word));

        // Create new note
        let mut note = AnkiNote {
            modelName: "JP Card V4".to_owned(),
            deckName: "My Deck 4.0".to_owned().into(),
            tags: word
                .get_all_tags()
                .iter()
                .map(|tag| tag.to_string())
                .collect(),
            fields,

            ..AnkiNote::default()
        };

        match anki.add_note(&mut note) {
            Ok(_) => {}
            Err(res) => println!("{}", res),
        }
    }
}

/// Filters out glossary entries that have the "forms" tag.
/// Used to exclude certain grammatical forms from processing.
fn filter_glossary(glossary: &Glossary) -> bool {
    !glossary.tags.contains("forms")
}

/// Constructs meaning field for Anki notes by:
/// - Formatting glossary entries with their tags
/// - Adding a separator between multiple entries
/// - Highlighting tags in square brackets
fn get_meaning(word: &Word) -> String {
    let mut output = "".to_owned();
    let mut previus_tags: HashSet<String> = HashSet::new();

    for (i, glossary) in word
        .glossary
        .iter()
        .filter(|gloss| filter_glossary(gloss))
        .enumerate()
    {
        if i != 0 {
            output += "<br>";
        }

        let meaning = glossary.meaning.join(" | ");

        // Add tags if they are new or not empty
        if glossary.tags.is_empty() || glossary.tags.iter().all(|k| previus_tags.contains(k)) {
            output += &meaning;
            continue;
        }

        let mut tags: Vec<&str> = glossary.tags.iter().map(|t| t.as_str()).collect();
        tags.sort_unstable();

        output += &format!("[ {} ] {}", tags.join(" "), meaning);

        previus_tags = glossary.tags.clone();
    }

    output
}

/// Constructs example sentences field for Anki notes by:
/// - Formatting Japanese-English example pairs
/// - Separating examples with line breaks
fn get_examples(word: &Word) -> String {
    word.examples
        .iter()
        .filter_map(
            |example| match (!example.japanese.is_empty(), !example.english.is_empty()) {
                (true, true) => Some(format!("{}<br>{}", example.japanese, example.english)),
                (true, false) => Some(example.japanese.clone()),
                _ => None,
            },
        )
        .reduce(|a, b| a + "<br><br>" + &b)
        .unwrap_or("".to_owned())
}
