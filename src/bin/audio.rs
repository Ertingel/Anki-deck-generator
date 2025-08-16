// cargo run --bin audio

use std::io::Write;
use std::{io, thread, time};

use anki_utill::{
    anki::{anki_connect::AnkiConnect, anki_note::AnkiNote},
    japanese::JapaneseStr,
};
use regex::Regex;

fn main() {
    println!("Fetching anki info");
    let anki = AnkiConnect::new("http://127.0.0.1:8765".into(), None).unwrap();
    let notes = anki
        .notes_info(
            &anki
                .find_notes("\"deck:My Deck 4.0\" \"note:JP Card V4\" \"3 Audio:\"")
                .unwrap(),
        )
        .unwrap();

    /* for note in notes.iter().take(10) {
        add_audio(&anki, note);
        thread::sleep(time::Duration::from_secs(1));
    } */

    println!("Adding audio to {} notes. ", notes.len());
    for (i, chunk) in notes.chunks(10).enumerate() {
        for (j, note) in chunk.iter().enumerate() {
            // Progress tracking every 5% of total notes
            if (i * 10 + j) % (notes.len() / 20) == 0 {
                print!(
                    "\n{:>3}% Notes ",
                    (((i * 10 + j) as f32 / notes.len() as f32) * 100.0).round()
                );
                io::stdout().flush().unwrap();
            }

            add_audio(&anki, note);
            print!("+");
            io::stdout().flush().unwrap();
            thread::sleep(time::Duration::from_secs(2));
        }

        for _ in 0..20 {
            print!("-");
            thread::sleep(time::Duration::from_secs(2));
            io::stdout().flush().unwrap();
        }
    }

    println!();
}

fn add_audio(anki: &AnkiConnect, note: &AnkiNote) {
    //https://assets.languagepod101.com/dictionary/japanese/audiomp3.php?kanji=猫&kana=ねこ,
    let word = &note.fields["1 Word"];
    let regex = Regex::new(r"\s").unwrap();
    let word = regex.replace_all(word, "").to_string();

    let url = format!(
        "https://assets.languagepod101.com/dictionary/japanese/audiomp3.php?kanji={}&kana={}",
        word.to_kanji(),
        word.to_kana()
    );

    println!("{}", url);

    let filename = word;
    let regex = Regex::new(r"\[").unwrap();
    let filename = regex.replace_all(&filename, "「").to_string();
    let regex = Regex::new(r"\]").unwrap();
    let filename = regex.replace_all(&filename, "」").to_string();
    let filename = format!("JapanesePod101_{}.mp3", filename);

    /* println!(
        "anki.add_note_audio(\n  {:?},\n  {:?},\n  {:?},\n  {:?},\n  None\n);\n",
        note.noteId.unwrap(),
        &url,
        vec!["3 Audio"],
        Some(&filename),
    ); */

    anki.add_note_audio(
        note.noteId.unwrap(),
        &url,
        &filename,
        &["3 Audio"],
        Some("7e2c2f954ef6051373ba916f000168dc"),
    );
}
