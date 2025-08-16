use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::{
    dict::dict_parser::ConvertableJmnedicData,
    entry::{Kanji, Word},
    japanese::to_furigana,
};

/// Represents a frequency entry that includes basic information about a word,
/// focusing on its kanji form and default frequency value, along with additional metadata.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct JmnedictFrequency(
    /// The word represented in kanji characters.
    String,
    /// Default value `freq`
    String,
    /// Additional properties and metadata related to the word's frequency entry,
    /// stored in a struct for organized access. This can include details like JLPT
    /// levels, contextual usage frequencies, or other relevant attributes.
    Properties,
);

impl JmnedictFrequency {
    /// Returns the word in kanji for this frequency entry.
    pub fn kanji(&self) -> &str {
        &self.0
    }

    /// Returns the reading of the word in kana.
    pub fn kana(&self) -> &str {
        self.2.kana()
    }

    /// Returns the JLPT level associated with this frequency entry.
    pub fn jlpt(&self) -> u8 {
        self.2.jlpt()
    }

    /// Returns the tags associated with this frequency entry.
    pub fn tags(&self) -> HashSet<String> {
        let mut out: HashSet<String> = HashSet::new();

        out.insert(format!("JLPT-N{}", self.jlpt()));

        out
    }
}

/// Contains properties of a word, including its reading and frequency data.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
struct Properties {
    frequency: FrequencyData,
    reading: String,
}

impl Properties {
    /// Returns the reading of the word in kana.
    fn kana(&self) -> &str {
        &self.reading
    }

    /// Returns the JLPT level associated with this properties entry by delegating to the frequency data.
    fn jlpt(&self) -> u8 {
        self.frequency.jlpt()
    }
}

/// Represents frequency data including its display value and numerical value.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[allow(non_snake_case)]
struct FrequencyData {
    /// The display representation of the frequency (e.g., "N5")
    displayValue: String,
    /// The numerical value associated with the frequency
    value: u8,
}

impl FrequencyData {
    /// Returns the JLPT level value associated with this frequency data.
    fn jlpt(&self) -> u8 {
        self.value
    }
}

impl ConvertableJmnedicData for JmnedictFrequency {
    fn convert_kanji_data(&self, _: &mut HashMap<char, Kanji>) -> Result<(), String> {
        Ok(())
    }

    fn convert_word_data(
        &self,
        words: &mut HashMap<(String, String), Word>,
        kanji_readings: &HashMap<char, HashSet<String>>,
    ) -> Result<(), String> {
        // Add or update the frequency data for an existing word
        if let Some(word) = words.get_mut(&(self.kanji().to_owned(), self.kana().to_owned())) {
            // Extend the word's frequency data.
            word.frequency.extend(self.tags().iter().cloned());
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
            let word = Word::new(0, furigana, Vec::new(), self.tags(), HashSet::new());

            words.insert((self.kanji().to_owned(), self.kana().to_owned()), word);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests the parsing of JSON data into a Frequency struct and verifies its properties, including word, reading, and JLPT level.
    #[test]
    fn frequency() {
        let data: Vec<JmnedictFrequency> = serde_json::from_str(TEST_DATA).unwrap();

        // Verifies that the parsed frequency entry has the expected word, reading, and JLPT level.

        // Test kanji
        assert_eq!(data[0].kanji(), "会う",);

        // Test kana
        assert_eq!(data[0].kana(), "あう",);

        // Test JLPT level
        assert_eq!(data[0].jlpt(), 5,);
    }

    /// The JSON data used for testing parsing of Frequency structs.
    const TEST_DATA: &str = r#"
[
    [
        "会う",
        "freq",
        {
            "frequency": {
                "displayValue": "N5",
                "value": 5
            },
            "reading": "あう"
        }
    ]
]
"#;
}
