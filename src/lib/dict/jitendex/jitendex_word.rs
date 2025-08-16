use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::{
    dict::{dict_parser::ConvertableJmnedicData, jmnedict::jmnedict_word::Glossary},
    entry::{self, Example, Kanji, Word},
    japanese::to_furigana,
};

/// Remaps a tag strings. Returns `None` if the input is an empty string or can be parsed as a number.
///
/// # Arguments
/// * `tag`: A string slice representing the tag to be remapped.
///
/// # Returns
/// - `Some<String>`: The remapped tag if it matches one of the predefined mappings.
/// - `None`: If the input is empty or can be parsed as a number.
pub fn remap_tag(tag: &str) -> Option<String> {
    // Checking if if the input is an empty string or can be parsed as a number.
    if tag.is_empty() || tag.parse::<f64>().is_ok() {
        return None;
    }

    // Remapping tags.
    match tag {
        "N5" => Some("JLPT-N5".to_owned()),
        "N4" => Some("JLPT-N4".to_owned()),
        "N3" => Some("JLPT-N3".to_owned()),
        "N2" => Some("JLPT-N2".to_owned()),
        "N1" => Some("JLPT-N1".to_owned()),
        _ => Some(tag.to_owned()),
    }
}

/// Represents a detailed entry about a Japanese word, including its kanji,
/// kana readings, tags, verb type, ordering value, glossary meanings, and frequency.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct JitendexWord(
    /// The word represented in kanji characters.
    String,
    /// The word's reading in hiragana (kana).
    String,
    /// Verb type if applicable (e.g., "ichidan", "godan"), providing information
    /// about conjugation patterns for verbs.
    String,
    /// Frequency information related to the word's usage, which is crucial for
    /// language learners focusing on commonly used vocabulary.
    String,
    /// An ordering value used to sort or sequence word entries, which can be useful
    /// for organizing vocabulary in a specific order, such as by frequency or level.
    i32,
    /// Glossary entries that provide meanings and context for the word,
    /// allowing users to understand the word's usage in different contexts.
    Glossary,
    /// A unique identifier for the word entry, ensuring each entry can be
    /// uniquely identified within the dataset.
    i32,
    /// Tags associated with the word, typically indicating its part of speech
    /// or other relevant linguistic features.
    String,
);

impl JitendexWord {
    /// Returns the kanji representation of the word.
    pub fn kanji(&self) -> &str {
        &self.0
    }

    /// Returns the kana (hiragana) reading of the word.
    pub fn kana(&self) -> &str {
        &self.1
    }

    /// Parses and returns a set of tags associated with the word.
    pub fn tags(&self) -> HashSet<String> {
        self.3.split(' ').filter_map(remap_tag).collect()
    }

    /// Returns the order value of the word entry.
    pub fn order(&self) -> i32 {
        self.4
    }

    /// Extracts and returns the glossary meanings from the glossary entries.
    pub fn glossary(&self) -> Vec<&str> {
        self.5.get_glossary()
    }

    /// Extracts and returns the example sentances from the glossary entries.
    pub fn example(&self) -> Vec<(String, String)> {
        self.5.get_example()
    }

    /// Returns the unique identifier of the word entry.
    pub fn id(&self) -> i32 {
        self.6
    }

    /// Parses and returns a set of frequency tags associated with the word.
    pub fn frequency(&self) -> HashSet<String> {
        self.2.split(' ').filter_map(remap_tag).collect()
    }
}

impl ConvertableJmnedicData for JitendexWord {
    fn convert_kanji_data(&self, _: &mut HashMap<char, Kanji>) -> Result<(), String> {
        Ok(())
    }

    fn convert_word_data(
        &self,
        words: &mut HashMap<(String, String), Word>,
        kanji_readings: &HashMap<char, HashSet<String>>,
    ) -> Result<(), String> {
        // Create a new Glossary from JMnedict data.
        let glossary = entry::Glossary::new(
            self.order(),
            self.tags(),
            self.glossary()
                .iter()
                .cloned()
                .map(|gloss| gloss.to_owned())
                .collect(),
        );

        // Add or update the word in the words HashMap
        if let Some(word) = words.get_mut(&(self.kanji().to_owned(), self.kana().to_owned())) {
            // Extends the word's data with additional JMnedict entries.
            word.word_id = self.id();
            word.glossary.push(glossary);
            word.frequency.extend(self.frequency().iter().cloned());
            word.examples.extend(
                self.example()
                    .iter()
                    .map(|(jp, en)| Example::new(jp.to_owned(), en.to_owned())),
            );
        } else {
            // Generate Furigana string
            let furigana = to_furigana(self.kanji(), self.kana(), kanji_readings);
            let furigana = if let Some(furigana) = furigana {
                furigana
            } else {
                /* println!(
                    "      Failed to convert: {} {}",
                    entry.kanji(),
                    entry.kana()
                ); */

                format!("{}[{}]", self.kanji(), self.kana())
            };

            // Create and insert new Word instance
            let word = Word::new(
                self.id(),
                furigana,
                vec![glossary],
                self.frequency(),
                self.example()
                    .iter()
                    .map(|(jp, en)| Example::new(jp.to_owned(), en.to_owned()))
                    .collect(),
            );

            words.insert((self.kanji().to_owned(), self.kana().to_owned()), word);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests the parsing of JSON data into a `JitendexWord` struct and verifies its properties, including kanji, kana, tags, order, id, frequency, and glossary meanings.
    #[test]
    fn word() {
        let data: Vec<JitendexWord> =
            serde_json::from_str(include_str!("./test_data.json")).unwrap();

        // Verify kanji values
        assert_eq!(data[0].kanji(), "食べる");
        assert_eq!(data[1].kanji(), "ライトウェルター級");

        // Verify kana readings
        assert_eq!(data[0].kana(), "たべる");
        assert_eq!(data[1].kana(), "ライトウェルターきゅう");

        // Verify tags
        assert_eq!(data[0].tags(), ["v1".to_owned()].into());
        assert_eq!(data[1].tags(), [].into());

        // Verify order values
        assert_eq!(data[0].order(), 200);
        assert_eq!(data[1].order(), 0);

        // Verify word IDs
        assert_eq!(data[0].id(), 1358280);
        assert_eq!(data[1].id(), 1969520);

        // Verify frequency tags
        assert_eq!(data[0].frequency(), ["★".to_owned()].into());
        assert_eq!(data[1].frequency(), [].into());

        // Verify glossary meanings
        assert_eq!(
            data[0].glossary(),
            [
                "to eat",
                "to live on (e.g. a salary)",
                "to live off",
                "to subsist on"
            ]
        );
        assert_eq!(data[1].glossary(), ["light welterweight (boxing)",]);

        // Verify examples
        assert_eq!(
            data[0].example(),
            [
                (
                    "もっと 果[くだ]物[もの]を<b> 食[た]べる</b>べきです。".to_owned(),
                    "You should eat more fruit.".to_owned()
                ),
                (
                    "僕[ぼく]は 脚[きゃく]本[ほん]家[か]で<b> 食[た]べて</b>いく 決[けっ]心[しん]をした。"
                        .to_owned(),
                    "I am determined to make a living as a playwright.".to_owned()
                ),
            ]
        );
        assert_eq!(data[1].example(), []);
    }
}
