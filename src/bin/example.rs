// cargo run --bin example

use std::collections::HashMap;
use std::io::Write;
use std::{collections::HashSet, io, time};

use anki_utill::tatoeba::tatoeba_search::{TatoebaOrigin, TatoebaSort};
use anki_utill::{
    anki::{anki_connect::AnkiConnect, anki_note::AnkiNote},
    japanese::JapaneseStr,
    tatoeba::tatoeba_search::TatoebaSearch,
};
use regex::{Captures, Regex};

/// Entry point of the example program.
///
/// Builds five `TatoebaSearch` configurations, connects to an
/// Anki‑deck and iterates over all notes that match a given query.
/// For each note it calls [`process_note`] to enrich the “4 Sentences”
/// field with up to *count* new examples.  
/// Progress is reported every ~2 % of the total notes.
fn main() {
    let mut search1 = TatoebaSearch::new("jpn", "eng");
    search1.word_count = (Some(3), Some(30));
    search1.is_orphan = Some(false);
    search1.is_unapproved = Some(false);
    search1.is_native = Some(true);
    search1.origin = Some(TatoebaOrigin::Original);

    search1.trans_is_direct = Some(true);
    search1.trans_is_orphan = Some(false);
    search1.trans_is_unapproved = Some(false);
    search1.trans_count = Some(true);

    search1.sort = Some(TatoebaSort::Shortest);
    search1.limit = Some(25);

    let mut search2 = search1.clone();
    search2.origin = None;

    let mut search3 = search1.clone();
    search3.is_orphan = None;
    search3.is_unapproved = None;
    search3.trans_is_orphan = None;
    search3.trans_is_unapproved = None;

    let mut search4 = search3.clone();
    search4.origin = None;

    let mut search5 = search4.clone();
    search5.is_native = None;

    let search = vec![search1, search2, search3, search4, search5];
    /* let search = vec![search5]; */

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

    /* let note = notes
        .iter()
        .filter(|note| note.fields["4 Sentences"].is_empty())
        .nth(1)
        .unwrap();
    process_note(&anki, &search, note, 15); */
    /* process_note(&anki, &search, &notes[0], 15); */

    println!("Adding examples to {} notes. ", notes.len());
    for (i, note) in notes.iter().enumerate() {
        // Progress tracking every 2% of total notes
        if i % (notes.len() / 50) == 0 {
            print!(
                "\n{:>3}% Notes ",
                ((i as f32 / notes.len() as f32) * 100.0).round()
            );
            io::stdout().flush().unwrap();
        }

        print!("|");

        process_note(&anki, &search, note, 15);
    }

    println!();
}

/// Augments a single Anki note with additional example sentences.
///
/// The function first parses any examples that are already present in the
/// “4 Sentences” field to avoid duplicates.  It then iterates over each
/// `TatoebaSearch` configuration until either *count* examples have been
/// collected or all searches are exhausted.  Each candidate example is
/// filtered for duplicate content, language correctness and proper
/// conjugation highlighting before being appended.
///
/// # Arguments
///
/// * `anki` – Connection used to update the note.
/// * `search` – Slice of `TatoebaSearch` objects that provide query settings.
/// * `note` – The Anki note to be processed.
/// * `count` – Maximum number of examples to keep in the field.
fn process_note(anki: &AnkiConnect, search: &[TatoebaSearch], note: &AnkiNote, count: usize) {
    // Parse already stored examples so we can avoid duplicates.
    let mut examples = parse_examples(note);
    /* let mut examples: Vec<(String, String)> = Vec::new(); */
    let mut filter: HashSet<String> = examples.iter().map(|(jp, _)| get_filter_key(jp)).collect();

    for search in search {
        if examples.len() >= count {
            break;
        }

        // Indicate progress to the user.
        print!("+");
        io::stdout().flush().unwrap();

        // Retrieve sentences matching the target word.
        for example in search.search_iter(
            &note.fields["1 Word"],
            Some(time::Duration::from_millis(333)),
        ) {
            /* --- Build transcription candidate -------------------------------- */
            let mut transcriptions: Vec<String> = example
                .transcriptions
                .into_iter()
                .filter_map(|e| format_tatoeba_response(note, &e.text))
                .collect();

            if transcriptions.is_empty() {
                continue;
            }

            // Prefer the longest transcription (most complete).
            transcriptions.sort_unstable_by_key(|e| std::cmp::Reverse(e.len()));
            let transcription = transcriptions.remove(0);

            /* --- Build translation candidate --------------------------------- */
            let mut translations: Vec<String> = example
                .translations
                .into_iter()
                .flatten()
                .map(|e| e.text)
                .collect();

            if translations.is_empty() {
                continue;
            }

            translations.sort_unstable_by_key(|e| std::cmp::Reverse(e.len()));
            let translation = translations.remove(0);

            /* --- Avoid duplicate or overly similar examples ----------------- */
            let key = get_filter_key(&transcription);

            if filter.contains(&key) {
                continue;
            } else {
                filter.insert(key);
            }

            examples.push((transcription, translation));

            print!("-");
            io::stdout().flush().unwrap();

            if examples.len() >= count {
                break;
            }
        }
    }

    /* ----------------------------------------------------------------------- */
    // Inform the user how many examples were added when we fell short.
    if examples.len() < count {
        print!("{}", examples.len());
    }

    /* println!("\nWord: ");
    for (jp, en) in &examples {
        println!("\n{jp}\n{en}");
    } */

    /* ----------------------------------------------------------------------- */
    // Join all examples into a single string and write back to Anki.
    let examples = examples
        .iter()
        .map(|(jp, en)| format!("{jp}<br>{en}"))
        .reduce(|a, b| a + "<br><br>" + &b)
        .unwrap_or("".to_owned());

    let mut fields: HashMap<String, String> = HashMap::new();
    fields.insert("4 Sentences".to_owned(), examples);
    anki.update_note_fields(note.noteId.unwrap(), &fields);
}

/// Creates a “filter key” used to de‑duplicate example sentences.
///
/// The key is produced by:
/// 1. Removing all whitespace.
/// 2. Stripping any HTML tags.
/// 3. Keeping only kanji characters (the rest are discarded).
///
/// This keeps the comparison lightweight while still distinguishing
/// distinct examples that differ in content or form.
fn get_filter_key(str: &str) -> String {
    let regex = Regex::new(r"\s").unwrap();
    let str = strip_html(str).to_kanji();
    regex.replace_all(&str, "").to_string()
}

/// Parses the “4 Sentences” field of a note into `(jp, en)` pairs.
///
/// The parsing logic follows the format produced by this program:
/// * `&nbsp;` → space
/// * `<br>`   → newline
/// * Lines are split into Japanese and English parts using a regex that
///   captures optional English translation.
///
/// Returns an empty vector if the field contains no examples.
fn parse_examples(note: &AnkiNote) -> Vec<(String, String)> {
    let str = &note.fields["4 Sentences"];

    // Replace HTML‑specific entities with plain text.
    let regex = Regex::new(r"&nbsp;").unwrap();
    let str = regex.replace_all(str, " ").to_string();

    let regex = Regex::new(r"<br>").unwrap();
    let str = regex.replace_all(&str, "\n").to_string();

    // Capture a Japanese line and an optional English translation.
    let regex =
        Regex::new(r"(?:^|\n)[ 	]*([^\n| 	][^\n|]*?)[ 	]*(?:\n[ 	]*([^\n]+?)[ 	]*)?(?:\n|$)")
            .unwrap();
    let matches = regex.captures_iter(&str);

    let out = matches
        .filter_map(|mat| {
            let jp = mat[1].to_owned();
            let en = mat.get(2)?.as_str().to_owned();

            // Highlight the target word in the Japanese example.
            let jp = rehighlight_word(note, &jp);

            jp.map(|jp| (jp, en))
        })
        .collect();

    out
}

/// Formats a raw Tatoeba transcription into the same style as used by this program.
///
/// * Removes alphabetic characters (English words are ignored).
/// * Replaces `[kanji|kana]` syntax with “kanji [kana]” highlighting.
/// * Finally highlights the target word within the note.
fn format_tatoeba_response(note: &AnkiNote, str: &str) -> Option<String> {
    // Ignore transcriptions that contain English letters.
    let regex = Regex::new(r"[A-Za-z]").unwrap();
    if regex.is_match(str) {
        return None;
    }

    // Replace `[kanji|kana]` syntax with highlighted form.
    let regex = Regex::new(r"\[((?:[^\[\]\|]+\|?)+)\]").unwrap();

    let str = regex
        .replace_all(str, |caps: &Captures| -> String {
            let caps = &caps[1];
            let mut caps: Vec<&str> = caps.split('|').collect();

            let kanji = caps.remove(0);
            let kana = caps;

            let out: String = kanji
                .chars()
                .zip(kana.iter().chain(std::iter::repeat(&"")))
                .map(|(kanji, kana)| {
                    if kana.is_empty() {
                        format!("{kanji}")
                    } else {
                        format!(" {kanji}[{kana}]")
                    }
                })
                .collect();

            out
        })
        .to_string();

    let regex = Regex::new("] +").unwrap();
    let str = regex.replace_all(&str, "]").to_string().trim().to_owned();

    highlight_word(note, &str)
}

fn rehighlight_word(note: &AnkiNote, str: &str) -> Option<String> {
    highlight_word(note, &strip_html(str))
}

fn highlight_word(note: &AnkiNote, str: &str) -> Option<String> {
    let before_len = str.len();
    let regex = Regex::new(&get_find_regex(note)).unwrap();
    let str = regex.replace_all(str, r"<b>$0</b>").to_string();

    if str.len() == before_len {
        None
    } else {
        Some(str)
    }
}

/// Removes any HTML tags from a string.
///
/// Used to strip formatting before further processing.
fn strip_html(str: &str) -> String {
    let regex = Regex::new(r"<[^<>]*>").unwrap();
    regex.replace_all(str, "").to_string()
}

/// Returns the regular expression used to highlight the target word in a note.
///
/// The pattern depends on the conjugation type of the target verb/adjective.
/// It attempts to match the base form and common inflected forms so that
/// the word is bolded wherever it appears in an example sentence.
fn get_find_regex(note: &AnkiNote) -> String {
    let word = &note.fields["1 Word"];
    // Escape literal brackets to avoid regex syntax errors.
    let regex = Regex::new(r"[\[\]]").unwrap();
    let word = regex.replace_all(word, "\\$0").to_string();

    let end = word.chars().last().unwrap();
    let stem = &word[..(word.len() - end.len_utf8())];

    match get_conjugation_type(note) {
        ConjugationType::None => format!(" ?{word}"),
        ConjugationType::IAdjective => format!("{stem}(?:くありませんでした|くないでしょう|くないだろう|くありません|くなかった|いでしょう|かったです|くなければ|いだろう|くない|いです|かった|ければ|い)"),
        ConjugationType::IxAdjective => "(?: ?良[よ]|良|よ)くありませんでした|(?: ?良[よ]|良|よ)くありません|(?: ?良[よ]|良|よ)くなかった|(?: ?良[よ]|良|よ)かったです|(?: ?良[よ]|良|よ)ければ|(?: ?良[よ]|良|よ)かった|(?: ?良[よ]|良|よ)くない|(?: ?良[よ]|良|よ)くて|いいです|いい".to_owned(),
        ConjugationType::NaAdjective => format!("{word}(?:ではありませんでした|ではありません|ではなかった|ではない|だった|でした|であれ|です|なれ|だろ|では|なら|なり|なる|で|だ|に|な|)"),
        ConjugationType::Ichidan => format!("{stem}(?:ていませんでした|なかったでしょう|ませんでしたら|なかっただろう|ないでください|ないでしょう|ませんでした|ないだろう|てください|させません|たでしょう|ていました|ていません|なかったら|られません|られません|るでしょう|られます|られない|させない|ています|るだろう|ただろう|なかった|ましょう|なければ|られます|られない|させます|ましたら|ている|ていた|ません|ました|させる|たろう|られる|られる|れば|ない|たら|よう|ます|るな|る|た|ろ)"),
        ConjugationType::Godan => {
            let end = match end {
                'う' => "らなかったでしょう|っていませんでした|らなかっただろう|りませんでしたら|らないでください|りませんでした|らないでしょう|っていました|っていません|らなかったら|ったでしょう|らないだろう|ってください|らなければ|らなかった|られません|らせません|るでしょう|っています|っただろう|りましたら|りましょう|っていた|っている|らせない|らせます|られない|りました|りません|れません|るだろう|られます|られない|られます|らせる|れます|られる|ったら|らない|ります|れない|るな|れば|ろう|った|れる|れ|る",
                'く' => "いていませんでした|かなかったでしょう|きませんでしたら|かなかっただろう|かないでください|かないでしょう|きませんでした|いたでしょう|いていました|いてください|かないだろう|いていません|かなかったら|くでしょう|いています|かなければ|かなかった|かれません|いただろう|かせません|きましょう|きましたら|けません|いている|かせない|いていた|きました|くだろう|きません|かれます|かせます|かれない|いたら|けます|けない|かせる|かれる|きます|かない|いた|こう|ける|くな|けば|く|け",
                'す' => "していませんでした|さなかったでしょう|しませんでしたら|さなかっただろう|さないでください|しませんでした|さないでしょう|してください|したでしょう|していません|さないだろう|さなかったら|していました|しただろう|すでしょう|さなかった|しています|されません|しましょう|さなければ|させません|しましたら|せません|されます|されない|していた|すだろう|しました|しません|している|させます|さない|せます|させる|さない|される|します|したら|せない|せば|そう|すな|した|せる|せ|す",
                'つ' => "っていませんでした|たなかったでしょう|たないでください|ちませんでしたら|たなかっただろう|たないでしょう|ちませんでした|たなかったら|っていません|ったでしょう|ってください|っていました|たないだろう|っただろう|ちましたら|たれません|たなければ|たせません|たなかった|ちましょう|つでしょう|っています|たれます|ちました|たれない|てません|たせます|つだろう|っている|ちません|たせない|っていた|てます|ちます|たせる|たれる|たない|てない|ったら|つな|てば|った|てる|とう|て|つ",
                'ぬ' => "んでいませんでした|ななかったでしょう|にませんでしたら|ななかっただろう|なないでください|なないでしょう|にませんでした|んでいません|ななかったら|んでください|なないだろう|んでいました|んだでしょう|なれません|にましたら|んでいます|ななければ|なせません|ななかった|にましょう|んだだろう|ぬでしょう|にました|んでいる|なれます|なれない|ねません|なせます|んでいた|なせない|にません|ぬだろう|ねます|なない|なれる|ねない|んだら|にます|なせる|ねば|んだ|ぬな|のう|ねる|ぬ|ね",
                'む' => "んでいませんでした|まなかったでしょう|みませんでしたら|まなかっただろう|まないでください|まないでしょう|みませんでした|んでいません|まなかったら|んでください|まないだろう|んでいました|んだでしょう|まれません|みましたら|んでいます|まなければ|ませません|まなかった|みましょう|んだだろう|むでしょう|みました|んでいる|まれます|まれない|めません|ませます|んでいた|ませない|みません|むだろう|めます|まない|まれる|めない|んだら|みます|ませる|めば|んだ|むな|もう|める|む|め",
                'る' => "っていませんでした|らなかったでしょう|りませんでしたら|らなかっただろう|らないでください|らないでしょう|りませんでした|ったでしょう|っていました|ってください|らないだろう|っていません|らなかったら|るでしょう|っています|らなければ|らなかった|られません|っただろう|らせません|りましょう|りましたら|れません|っている|らせない|っていた|りました|るだろう|りません|られます|らせます|られない|ったら|れます|れない|らせる|られる|ります|らない|った|ろう|れる|るな|れば|る|れ",

                'ぐ' => "いでいませんでした|がなかったでしょう|ぎませんでしたら|がなかっただろう|がないでください|がないでしょう|ぎませんでした|いだでしょう|いでいました|いでください|がないだろう|いでいません|がなかったら|ぐでしょう|いでいます|がなかった|がなければ|がせません|がれません|ぎましょう|いだだろう|ぎましたら|がせます|げません|いでいる|いでいた|ぎました|ぐだろう|ぎません|がれます|がれない|げます|いだら|がない|がせる|がれる|げない|ぎます|がない|いだ|げる|ぐな|げば|ぐ|ご|げ",
                'づ' => "っていませんでした|たなかったでしょう|たないでください|ちませんでしたら|たなかっただろう|たないでしょう|ちませんでした|たなかったら|っていません|ったでしょう|ってください|っていました|たないだろう|っただろう|ちましたら|たれません|たなければ|たせません|たなかった|ちましょう|つでしょう|っています|たれます|ちました|たれない|てません|たせます|つだろう|っている|ちません|たせない|っていた|てます|ちます|たせる|たれる|たない|てない|ったら|つな|てば|った|てる|とう|て|つc",
                'ぶ' => "んでいませんでした|ばなかったでしょう|びませんでしたら|ばなかっただろう|ばないでください|ばないでしょう|びませんでした|んでいません|ばなかったら|んでください|ばないだろう|んでいました|んだでしょう|ばれません|びましたら|んでいます|ばなければ|ばせません|ばなかった|びましょう|ぶでしょう|んだだろう|びました|んでいる|ばれない|ばれます|んでいた|べません|ばせます|びません|ぶだろう|ばない|べます|ばれる|べない|んだら|ばせる|びます|ばない|べば|んだ|ぶな|べる|ぶ|ぼ|べ",

                'ふ' | 'ず' | 'ぷ' => panic!("There is no godan verb ending with '{end}'! ({word})"),
                _ => panic!("Unknown godan verb \"{word}\" ending '{end}'!"),
            };

            format!(" ?{stem}(?:{end})")
        }
        ConjugationType::Aru => "(?: ?有[あ]|有|あ)|(?: ?有[あ]|有|あ)りませんでした|(?: ?有[あ]|有|あ)ってください|ないでください|(?: ?有[あ]|有|あ)らせません|(?: ?有[あ]|有|あ)られません|(?: ?有[あ]|有|あ)りましょう|(?: ?有[あ]|有|あ)りました|(?: ?有[あ]|有|あ)られない|(?: ?有[あ]|有|あ)られます|(?: ?有[あ]|有|あ)らせない|(?: ?有[あ]|有|あ)らせます|(?: ?有[あ]|有|あ)りません|なかったら|(?: ?有[あ]|有|あ)れません|なかった|(?: ?有[あ]|有|あ)れない|(?: ?有[あ]|有|あ)らせる|(?: ?有[あ]|有|あ)られる|(?: ?有[あ]|有|あ)れます|(?: ?有[あ]|有|あ)ります|なければ|(?: ?有[あ]|有|あ)ったら|(?: ?有[あ]|有|あ)って|なくて|(?: ?有[あ]|有|あ)ろう|(?: ?有[あ]|有|あ)るな|(?: ?有[あ]|有|あ)れば|(?: ?有[あ]|有|あ)った|(?: ?有[あ]|有|あ)れる|(?: ?有[あ]|有|あ)れ|(?: ?有[あ]|有|あ)る|ない".to_owned(),
        ConjugationType::Kuru => "(?: ?来[く]|来|く)なかったでしょう|(?: ?来[く]|来|く)なかっただろう|(?: ?来[く]|来|く)ませんでしたら|(?: ?来[く]|来|く)ないでください|(?: ?来[く]|来|く)ないでしょう|(?: ?来[く]|来|く)ませんでした|(?: ?来[く]|来|く)させません|(?: ?来[く]|来|く)るでしょう|(?: ?来[く]|来|く)ませんなら|(?: ?来[く]|来|く)てください|(?: ?来[く]|来|く)なかったら|(?: ?来[く]|来|く)たでしょう|(?: ?来[く]|来|く)られません|(?: ?来[く]|来|く)ないだろう|(?: ?来[く]|来|く)させない|(?: ?来[く]|来|く)なかった|(?: ?来[く]|来|く)させます|(?: ?来[く]|来|く)なければ|(?: ?来[く]|来|く)られない|(?: ?来[く]|来|く)られます|(?: ?来[く]|来|く)ますれば|(?: ?来[く]|来|く)ましたら|(?: ?来[く]|来|く)られる|(?: ?来[く]|来|く)させる|(?: ?来[く]|来|く)ました|(?: ?来[く]|来|く)られる|(?: ?来[く]|来|く)ません|きませば|(?: ?来[く]|来|く)れば|(?: ?来[く]|来|く)ない|(?: ?来[く]|来|く)るな|(?: ?来[く]|来|く)よう|(?: ?来[く]|来|く)たら|(?: ?来[く]|来|く)ます|(?: ?来[く]|来|く)い|(?: ?来[く]|来|く)る|(?: ?来[く]|来|く)た".to_owned(),
        ConjugationType::Suru => "していませんでした|しなかっただろう|しないでください|しなかたでしょう|しませんでしたら|しませんでした|しないでしょう|[為す]るでしょう|しましたろう|していません|しませんなら|しないだろう|しなかったら|していました|してください|しなければ|しましたら|しなかった|しますれば|[為す]るだろう|しましょう|できません|しています|しただろう|したろう|しました|できます|しません|しませば|できない|したら|させる|される|できる|[為す]れば|[為す]るな|します|しよう|しない|[為す]る|した|しろ".to_owned(),
    }
}

/// Determines the conjugation type of a word based on its ending and tags.
///
/// The function first checks that the last character is a known verb/adjective
/// ending.  It then looks at the note’s tags to decide between:
///
/// * `IAdjective`, `IxAdjective` – い‑adjectives (plain or x‑form)
/// * `NaAdjective` – な‑adjectives.
/// * `Aru` – “ある” verbs.
/// * `Kuru` – “くる” verbs.
/// * `Suru` – “する” verbs.
/// * `Godan`, `Ichidan` – group 5 or 1 verbs.
///   If no known tag matches, it returns `ConjugationType::None`.
fn get_conjugation_type(note: &AnkiNote) -> ConjugationType {
    let word = &note.fields["1 Word"];
    let verb_end = word.chars().last().unwrap();

    match verb_end {
        'い' | 'う' | 'く' | 'す' | 'つ' | 'ぬ' | 'ふ' | 'む' | 'る' | 'ぐ' | 'ず' | 'づ'
        | 'ぶ' | 'ぷ' => {}
        _ => {
            return ConjugationType::None;
        }
    }

    if note.tags.iter().any(|tag| tag == "adj-いx") {
        return ConjugationType::IxAdjective;
    }

    if note.tags.iter().any(|tag| tag == "adj-い") {
        return ConjugationType::IAdjective;
    }

    if note.tags.iter().any(|tag| tag == "adj-な") {
        return ConjugationType::NaAdjective;
    }

    if note.tags.iter().any(|tag| tag == "v5る-i") {
        return ConjugationType::Aru;
    }

    if note.tags.iter().any(|tag| tag == "vくる") {
        return ConjugationType::Kuru;
    }

    if note
        .tags
        .iter()
        .any(|tag| tag == "vする-i" || tag == "vする-s")
    {
        return ConjugationType::Suru;
    }

    if note.tags.iter().any(|tag| tag.starts_with("v5")) {
        return ConjugationType::Godan;
    }

    if note.tags.iter().any(|tag| tag.starts_with("v1")) {
        return ConjugationType::Ichidan;
    }

    ConjugationType::None
}

/// Possible conjugation types that influence regex construction.
///
/// The variants correspond to the different morphological patterns
/// encountered in Japanese verbs and adjectives.
enum ConjugationType {
    None,
    IAdjective,
    IxAdjective,
    NaAdjective,
    Ichidan,
    Godan,
    Aru,
    Kuru,
    Suru,
}
