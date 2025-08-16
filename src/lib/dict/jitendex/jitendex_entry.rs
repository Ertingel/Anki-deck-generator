use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::{
    dict::{dict_parser::ConvertableJmnedicData, kanjidic::kanjidic_kanji},
    entry::Word,
};

use super::{super::jmnedict::jmnedict_frequency, jitendex_word};

/// Represents an entry containing information about various aspects of the Japanese language,
/// including words, kanji characters, and their associated properties.
/// The entry can be one of several types, each encapsulated in its own variant.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(untagged)]
pub enum JitendexEntry {
    /// A word entry that contains detailed information about a Japanese word,
    /// including its kanji representation, kana readings, tags, verb type,
    /// ordering value, glossary meanings, and frequency identifier.
    Word(jitendex_word::JitendexWord),

    /// A frequency entry that includes the kanji form of a word along with
    /// its default frequency value and additional metadata properties.
    Frequency(jmnedict_frequency::JmnedictFrequency),

    /// A kanji entry that provides detailed information about a single kanji character,
    /// including its various readings (both onyomi and kunyomi), its frequency or dictionary
    /// information, meanings in English, and additional metadata stored in a HashMap.
    Kanji(kanjidic_kanji::KanjidicEntry),

    /// A catch-all variant for any unexpected JSON value that does not fit into the predefined categories.
    /// This is used to handle unknown data types gracefully during deserialization.
    Unknown(serde_json::Value),
}

impl ConvertableJmnedicData for JitendexEntry {
    fn convert_kanji_data(
        &self,
        kanji: &mut std::collections::HashMap<char, crate::entry::Kanji>,
    ) -> Result<(), String> {
        match self {
            JitendexEntry::Word(jmnedict_word) => jmnedict_word.convert_kanji_data(kanji),
            JitendexEntry::Frequency(jmnedict_frequency) => {
                jmnedict_frequency.convert_kanji_data(kanji)
            }
            JitendexEntry::Kanji(kanjidic_entry) => kanjidic_entry.convert_kanji_data(kanji),
            JitendexEntry::Unknown(_) => Err("Unknown value".to_owned()),
        }
    }

    fn convert_word_data(
        &self,
        words: &mut HashMap<(String, String), Word>,
        kanji_readings: &HashMap<char, HashSet<String>>,
    ) -> Result<(), String> {
        match self {
            JitendexEntry::Word(jmnedict_word) => {
                jmnedict_word.convert_word_data(words, kanji_readings)
            }
            JitendexEntry::Frequency(jmnedict_frequency) => {
                jmnedict_frequency.convert_word_data(words, kanji_readings)
            }
            JitendexEntry::Kanji(kanjidic_entry) => {
                kanjidic_entry.convert_word_data(words, kanji_readings)
            }
            JitendexEntry::Unknown(data) => {
                // Handle unrecognized entry types
                Err(format!(
                    "Failed to convert: {} {}\n{}",
                    data[0],
                    data[1],
                    serde_json::to_string_pretty(data).unwrap()
                ))
            }
        }
    }
}
