// cargo run --bin order

use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
    fs,
};

use anki_utill::{
    anki::{
        anki_connect::AnkiConnect,
        anki_note::{AnkiNote, ID},
    },
    entry::Kanji,
};

fn main() {
    let kanjilist_save_path = "./result/kanjilist.json";

    // Load kanji data from JSON file
    println!("Loading kanji from {}", kanjilist_save_path);
    let data = fs::read_to_string(kanjilist_save_path).unwrap();
    let kanji: HashMap<char, Kanji> = serde_json::from_str(&data).unwrap();

    // Connect to Anki and fetch note information
    println!("Fetching anki info");
    let anki = AnkiConnect::new("http://127.0.0.1:8765".into(), None).unwrap();
    let notes = anki
        .notes_info(
            &anki
                .find_notes("\"deck:My Deck 4.0\" \"note:JP Card V4\"")
                .unwrap(),
        )
        .unwrap();

    // Get active cards from Anki
    let cards: HashSet<ID> = anki
        .find_cards("\"deck:My Deck 4.0\" \"note:JP Card V4\" is:new -is:suspended -is:buried")
        .unwrap()
        .into_iter()
        .collect();

    // Sort and group notes by JLPT level, kanji complexity, and interleaved kana
    println!("Sorting cards");
    let sorted: Vec<AnkiNote> = sort_jlpt_level(notes)
        .into_iter()
        .rev()
        .map(|notes| sort_by_kanji(notes, &kanji))
        .map(|notes| sort_order(notes, &kanji))
        .flat_map(|(kana, kanji)| flatten_jlpt(kana, kanji))
        .collect();

    // Update Anki cards with new due dates based on sorted order
    println!("Applying sorted list to anki");
    let count = sorted.len();
    for (i, note) in sorted.into_iter().enumerate() {
        if i % (count / 20) == 0 {
            println!(
                "  {:>3}% Notes",
                ((i as f32 / count as f32) * 100.0).round()
            );
        }

        let card: Vec<ID> = note
            .cards
            .unwrap_or_default()
            .into_iter()
            .filter(|card| cards.contains(card))
            .collect();

        for id in card {
            anki.set_specific_value_of_card(id, vec![("due", &((i + 1) as i32).into())])
                .unwrap();
        }
    }
}

/// Flattens and interleaves Kana and Kanji notes based on JLPT level spacing requirements.
/// Interleaves notes from kana and kanji vectors such that each Kana note is repeated as per fill_count.
/// This ensures balanced practice between different script types while maintaining spaced repetition.
///
/// # Arguments
/// * `kana` - Vector of AnkiNotes for Kana (e.g., Hiragana, Katakana)
/// * `kanji` - Vector of vectors containing AnkiNotes grouped by JLPT level
///
/// # Returns
/// A single vector containing interleaved Kana and Kanji notes according to spacing requirements.
fn flatten_jlpt(mut kana: Vec<AnkiNote>, kanji: Vec<Vec<AnkiNote>>) -> Vec<AnkiNote> {
    let mut out = Vec::new();

    // Calculate how many kana notes should be interleaved per kanji group
    let fill_count = kana.len() as f32 / kanji.len() as f32;
    let mut count: f32 = 0.0;

    for note in kanji.into_iter() {
        out.extend_from_slice(&note);

        // Add a kana note when the fill threshold is met
        count += fill_count;
        while count >= 1.0 {
            out.push(kana.remove(0));
            count -= 1.0;
        }
    }

    // Add any remaining kana notes
    out.extend_from_slice(&kana);

    /* println!();
    for note in out.iter() {
        print!("{} ", note.fields["1 Word"])
    }
    println!(); */

    /* println!("{}", kana.len()); */

    out
}

/// Sorts AnkiNotes by JLPT level and Kanji complexity for optimal spaced repetition scheduling.
/// Notes are first grouped by their JLPT level, then within each group sorted by the number of kanji strokes
/// and frequency. This ensures notes with complex or frequently used kanji are practiced earlier.
///
/// # Arguments
/// * `notes` - HashMap where keys represent JLPT levels (as strings) and values are vectors of AnkiNotes
/// * `kanji` - Kanji data structure containing stroke counts for each character, unused in this function
fn sort_order(
    mut notes: HashMap<String, Vec<AnkiNote>>,
    kanji: &HashMap<char, Kanji>,
) -> (Vec<AnkiNote>, Vec<Vec<AnkiNote>>) {
    let kana = notes.remove("").unwrap_or_default();

    // Group notes by JLPT level
    let mut notes: Vec<(String, Vec<AnkiNote>)> =
        notes.into_iter().filter(|(k, _)| !k.is_empty()).collect();

    // Count frequency of each kanji across all notes for prioritization
    let kanji_filter: HashMap<char, u32> =
        notes.iter().fold(HashMap::new(), |mut res, (kan, _)| {
            for kan in kan.chars() {
                if let Some(count) = res.get(&kan) {
                    res.insert(kan, count + 1);
                } else {
                    res.insert(kan, 1);
                }
            }
            res
        });

    // Keep only kanji that appear more than once
    let kanji_filter: HashSet<char> = kanji_filter
        .into_iter()
        .filter(|(_, count)| *count > 1)
        .map(|(kan, _)| kan)
        .collect();

    // Sort notes by complexity and frequency of kanji
    notes.sort_unstable_by_key(|(kan, _)| {
        let count = kan.len();
        let strokes: u32 = kan
            .chars()
            .map(|k| kanji.get(&k).map_or(0, |k| k.strokes.unwrap_or(0) as u32))
            .sum();

        // Extract and sort kanji based on complexity and frequency
        let mut kanji_list: Vec<char> = kan
            .chars()
            .filter(|kan| kanji_filter.contains(kan))
            .collect();

        if kan.is_empty() {
            kanji_list = kan
                .chars()
                .filter(|kan| kanji_filter.contains(kan))
                .collect();
        }

        kanji_list.sort_unstable_by(|a, b| {
            let strokes = kanji
                .get(a)
                .map_or(u8::MAX, |e| e.strokes.unwrap_or(u8::MAX))
                .cmp(
                    &kanji
                        .get(b)
                        .map_or(u8::MAX, |e| e.strokes.unwrap_or(u8::MAX)),
                );

            if strokes != Ordering::Equal {
                return strokes;
            }

            a.cmp(b)
        });

        let kan = kanji_list.last().unwrap_or(&' ');
        let kan_strokes = kanji
            .get(kan)
            .map_or(u8::MAX, |e| e.strokes.unwrap_or(u8::MAX));

        // Create a sortable key combining stroke count and word properties
        let key = format!("{:0>3}{}{:0>3}{:0>4}", kan_strokes, kan, count, strokes);

        key
    });

    /* println!();
    for (kanji, _) in notes.iter() {
        print!("{} ", kanji)
    }
    println!(); */

    let notes = notes.into_iter().map(|(_, notes)| notes).collect();

    (kana, notes)
}

/// Groups AnkiNotes based on their Kanji composition for targeted spaced repetition.
/// Notes are grouped together if they contain the same set of Kanji characters, ignoring order.
///
/// # Arguments
/// * `notes` - Vector of AnkiNotes to be grouped
/// * `kanji` - Mapping from Kanji characters to their properties (unused in this function)
///
/// # Returns
/// A HashMap where keys are sorted strings of Kanji characters and values are vectors of notes sharing those Kanji.
fn sort_by_kanji(
    notes: Vec<AnkiNote>,
    kanji: &HashMap<char, Kanji>,
) -> HashMap<String, Vec<AnkiNote>> {
    let mut out: HashMap<String, Vec<AnkiNote>> = HashMap::new();

    for note in notes.into_iter() {
        // Extract the unique Kanji characters from the note's "1 Word" field
        let key = get_kanji(&note, kanji);

        // Group notes by their Kanji composition
        if let Some(entry) = out.get_mut(&key) {
            entry.push(note);
        } else {
            out.insert(key, vec![note]);
        }
    }

    // Sort the groups and their contents for consistency
    out = out
        .into_iter()
        .map(|(key, notes)| (key, sort_notes(notes)))
        .collect();

    out
}

fn get_kanji(note: &AnkiNote, kanji: &HashMap<char, Kanji>) -> String {
    // Filter only the Kanji characters present in the global Kanji map
    let mut vec: Vec<char> = note.fields["1 Word"]
        .chars()
        .filter(|c| kanji.contains_key(c))
        .collect();

    // Sort the Kanji to ensure consistent grouping regardless of order
    vec.sort_unstable();
    vec.iter().collect()
}

/// Split notes into vectors based on JLPT level (N1-N5)
/// Returns an array where each index corresponds to a JLPT level,
/// with index 0 being non-JLPT, up to index 5 being N1.
fn sort_jlpt_level(notes: Vec<AnkiNote>) -> [Vec<AnkiNote>; 6] {
    // Initialize an array of empty vectors for each JLPT level
    let mut out = [
        Vec::new(), // non-JLPT
        Vec::new(), // N1
        Vec::new(), // N2
        Vec::new(), // N3
        Vec::new(), // N4
        Vec::new(), // N5
    ];

    // Distribute notes into their respective JLPT level vectors
    for note in notes {
        out[get_jlpt_level(&note).unwrap_or(0) as usize].push(note);
    }

    out
}

/// Determine the JLPT level from note tags
/// Returns Some(level) if a JLPT tag is found, None otherwise.
fn get_jlpt_level(note: &AnkiNote) -> Option<u8> {
    // Check each tag for JLPT levels in descending order
    if note.tags.contains(&"JLPT-N1".into()) {
        Some(1)
    } else if note.tags.contains(&"JLPT-N2".into()) {
        Some(2)
    } else if note.tags.contains(&"JLPT-N3".into()) {
        Some(3)
    } else if note.tags.contains(&"JLPT-N4".into()) {
        Some(4)
    } else if note.tags.contains(&"JLPT-N5".into()) {
        Some(5)
    } else {
        None
    }
}

/// Simple sort function that sorts notes alphabetically by their word field
fn sort_notes(mut notes: Vec<AnkiNote>) -> Vec<AnkiNote> {
    // Sort notes by the "1 Word" field in ascending order
    notes.sort_by(|a, b| a.fields["1 Word"].cmp(&b.fields["1 Word"]));
    notes
}
