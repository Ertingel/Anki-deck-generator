use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::{
    dict::dict_parser::ConvertableJmnedicData,
    entry::{Kanji, Word},
    japanese::{split_kanji_reading, JapaneseStr},
};

/// Represents a kanji character along with its readings and properties,
/// providing comprehensive information about each kanji in a structured format.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct KanjidicEntry(
    /// The kanji character itself, which is the primary focus of this entry.
    char,
    /// Onyomi (Chinese-derived readings) of the kanji, providing insight
    /// into its pronunciation influenced by Chinese roots.
    String,
    /// Kunyomi (Japanese-native readings) of the kanji, offering native Japanese
    /// pronunciations that are essential for understanding the character's usage
    /// in different contexts.
    String,
    /// Frequency or dictionary information related to the kanji, which is valuable
    /// for learners focusing on commonly used kanji characters.
    String,
    /// Meanings of the kanji in English, facilitating comprehension and memorization
    /// for non-native speakers learning Japanese.
    Vec<String>,
    /// Additional properties and metadata about the kanji, stored in a HashMap
    /// for flexibility and ease of access. This allows for various attributes
    /// such as stroke counts, radicals, or other relevant information to be included.
    HashMap<String, String>,
);

impl KanjidicEntry {
    /// Returns the kanji character.
    pub fn kanji(&self) -> char {
        self.0
    }

    /// Returns the onyomi readings as a set of strings.
    pub fn onyomi(&self) -> HashSet<&str> {
        self.1.split(' ').collect()
    }

    /// Returns the kunyomi readings as a set of strings.
    pub fn kunyomi(&self) -> HashSet<&str> {
        self.2.split(' ').collect()
    }

    /// Returns all possible readings (both onyomi and kunyomi) converted to hiragana.
    pub fn readings(&self) -> HashSet<String> {
        let mut out: HashSet<String> = HashSet::new();

        // Process onyomi
        out.extend(
            self.onyomi()
                .iter()
                .filter_map(|r| Some(split_kanji_reading(r)?.1))
                .map(|r| r.to_hiragana()),
        );

        // Process kunyomi
        out.extend(
            self.kunyomi()
                .iter()
                .filter_map(|r| Some(split_kanji_reading(r)?.1))
                .map(|r| r.to_hiragana()),
        );

        out
    }

    /// Returns the meanings of the kanji as a set of strings.
    pub fn meaning(&self) -> HashSet<&str> {
        self.4.iter().map(|x| x.as_str()).collect()
    }

    /// Returns the number of strokes for the kanji, if available.
    pub fn strokes(&self) -> Option<u8> {
        self.5.get("strokes")?.parse::<u8>().ok()
    }

    /// Returns the JLPT level associated with the kanji, if available.
    pub fn jlpt(&self) -> Option<u8> {
        self.5.get("jlpt")?.parse::<u8>().ok()
    }

    /// Returns the tags associated with the kanji.
    pub fn tags(&self) -> HashSet<String> {
        if let Some(jlpt) = self.jlpt() {
            let mut out: HashSet<String> = HashSet::new();

            out.insert(format!("JLPT-N{}", jlpt));

            out
        } else {
            HashSet::new()
        }
    }
}

impl ConvertableJmnedicData for KanjidicEntry {
    fn convert_kanji_data(&self, kanji: &mut HashMap<char, Kanji>) -> Result<(), String> {
        // Creates a new Kanji from Kanjidic entry data.
        kanji.insert(
            self.kanji(),
            Kanji::new(
                self.kanji().to_owned(),
                self.onyomi()
                    .iter()
                    .cloned()
                    .map(|reading| reading.to_owned())
                    .collect(),
                self.kunyomi()
                    .iter()
                    .cloned()
                    .map(|reading| reading.to_owned())
                    .collect(),
                self.meaning()
                    .iter()
                    .cloned()
                    .map(|meaning| meaning.to_owned())
                    .collect(),
                self.strokes(),
                self.tags()
                    .iter()
                    .cloned()
                    .map(|meaning| meaning.to_owned())
                    .collect(),
            ),
        );

        Ok(())
    }

    fn convert_word_data(
        &self,
        _: &mut HashMap<(String, String), Word>,
        _: &HashMap<char, HashSet<String>>,
    ) -> Result<(), String> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests the parsing and properties of the `KanjidicEntry` struct.
    #[test]
    fn kanji() {
        let data: Vec<KanjidicEntry> =
            serde_json::from_str(include_str!("./test_data.json")).unwrap();

        // Test kanji characters
        assert_eq!(data[0].kanji(), '亜',);
        assert_eq!(data[1].kanji(), '唖',);

        // Test onyomi readings
        assert_eq!(data[0].onyomi(), ["ア"].into(),);
        assert_eq!(data[1].onyomi(), ["ア", "アク"].into(),);

        // Test kunyomi readings
        assert_eq!(data[0].kunyomi(), ["つ.ぐ"].into(),);
        assert_eq!(data[1].kunyomi(), ["おし"].into(),);

        // Test converted readings in hiragana
        assert_eq!(
            data[0].readings(),
            ["あ".to_owned(), "つ".to_owned()].into()
        );
        assert_eq!(
            data[1].readings(),
            ["あ".to_owned(), "あく".to_owned(), "おし".to_owned()].into()
        );

        // Test meanings
        assert_eq!(
            data[0].meaning(),
            ["Asia", "rank next", "come after", "-ous"].into(),
        );
        assert_eq!(data[1].meaning(), ["mute", "dumb"].into(),);

        // Test strokes
        assert_eq!(data[0].strokes(), Some(7),);
        assert_eq!(data[1].strokes(), Some(10),);

        // Test JLPT level
        assert_eq!(data[0].jlpt(), Some(1),);
        assert_eq!(data[1].jlpt(), None,);
    }
}
