use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};

use crate::{
    dict::dict_parser::ConvertableJmnedicData,
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

    Some(
        match tag {
            "N5" => "jlpt-N5",
            "N4" => "jlpt-N4",
            "N3" => "jlpt-N3",
            "N2" => "jlpt-N2",
            "N1" => "jlpt-N1",

            "adj-i" => "adj-い",
            "adj-ix" => "adj-いx",
            "adj-ku" => "adj-く",
            "adj-na" => "adj-な",
            "adj-no" => "adj-の",
            "adj-to" => "adj-と",
            "adj-kari" => "adj-かり",
            "adj-shiku" => "adj-しく",
            "adj-taru" => "adj-たる",
            "adj-nari" => "adj-なり",
            "i-adjective" => "い-adjective",
            "i-adj" => "い-adj",
            "ix-adj" => "いx-adj",
            "ku-adj" => "く-adj",
            "na-adj" => "な-adj",
            "no-adj" => "の-adj",
            "to-adj" => "と-adj",
            "kari-adj" => "かり-adj",
            "shiku-adj" => "しく-adj",
            "taru-adj" => "たる-adj",
            "tari-adj" => "なり-adj",

            "adv-to" => "adv-と",
            "to-adv" => "と-adv",

            "vr" => "vり",
            "vk" => "vくる",
            "vs" => "vする",
            "vz" => "vずる",
            "vn" => "vぬ-i",
            "vs-i" => "vする-i",
            "vs-s" => "vする-s",

            "v4k" => "v4く",
            "v4s" => "v4す",
            "v4t" => "v4つ",
            "v4n" => "v4ぬ",
            "v4h" => "v4ふ",
            "v4m" => "v4む",
            "v4r" => "v4る",
            "v4g" => "v4ぐ",
            "v4b" => "v4ぶ",

            "v5u" => "v5う",
            "v5k" => "v5く",
            "v5s" => "v5す",
            "v5t" => "v5つ",
            "v5n" => "v5ぬ",
            "v5m" => "v5む",
            "v5r" => "v5る",
            "v5g" => "v5ぐ",
            "v5b" => "v5ぶ",
            "v5u-s" => "v5う-s",
            "v5k-s" => "v5く-s",
            "v5r-i" => "v5る-i",
            "v5aru" => "v5ある",
            "v5uru" => "v5うる",

            _ => tag,
        }
        .to_owned(),
    )
}

/// Represents a detailed entry about a Japanese word, including its kanji,
/// kana readings, tags, verb type, ordering value, glossary meanings, and frequency.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct JmnedictWord(
    /// The word represented in kanji characters.
    String,
    /// The word's reading in hiragana (kana).
    String,
    /// Tags associated with the word, typically indicating its part of speech
    /// or other relevant linguistic features.
    String,
    /// Verb type if applicable (e.g., "ichidan", "godan"), providing information
    /// about conjugation patterns for verbs.
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
    /// Frequency information related to the word's usage, which is crucial for
    /// language learners focusing on commonly used vocabulary.
    String,
);

impl JmnedictWord {
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
        self.2.split(' ').filter_map(remap_tag).collect()
    }

    /// Returns the order value of the word entry.
    pub fn order(&self) -> i32 {
        self.4
    }

    /// Extracts and returns the glossary meanings from the glossary entries.
    pub fn glossary(&self) -> Vec<&str> {
        self.5.get_glossary()
        /* self.5
        .iter()
        .flat_map(|glossary| glossary.get_glossary())
        .collect()
         */
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
        self.7.split(' ').filter_map(remap_tag).collect()
    }
}

/// Represents different forms of glossary content within a Japanese dictionary entry.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(untagged)]
pub enum Glossary {
    /// A simple string representation of a glossary entry.
    String(String),
    /// A structured content with additional metadata.
    StructuredContent(StructuredContent),
    /// An array of nested glossary entries or items.
    Array(Vec<Glossary>),
}

impl Glossary {
    /// Extracts and returns the glossary meanings from the content based on its type.
    ///
    /// # Arguments
    /// * `self` - A reference to the Glossary enum variant.
    /// * `in_glossary` - A boolean indicating whether we're within a nested glossary context.
    ///
    /// # Returns
    /// A vector of string slices, each representing a glossary meaning extracted from the content.
    pub fn get_glossary(&self) -> Vec<&str> {
        match self {
            // If the content is a string and we're within a glossary context,
            // return it as part of the meanings. Otherwise, return an empty vector.
            Glossary::String(str) => vec![str],

            // For arrays, recursively process each item.
            Glossary::StructuredContent(glossary) => glossary.get_glossary(),

            // For structured content, delegate to the Struct implementation.
            Glossary::Array(list) => list
                .iter()
                .flat_map(|glossary| glossary.get_glossary())
                .collect(),
        }
    }

    /// Extracts and returns the example meanings from the content based on its type.
    ///
    /// # Arguments
    /// * `self` - A reference to the Glossary enum variant.
    /// * `in_example` - A boolean indicating whether we're within a nested example context.
    ///
    /// # Returns
    /// A vector of string slices, each representing a example sentence extracted from the content.
    pub fn get_example(&self) -> Vec<(String, String)> {
        match self {
            Glossary::String(_) => vec![],

            // For arrays, recursively process each item.
            Glossary::StructuredContent(example) => example.get_example(),

            // For structured content, delegate to the Struct implementation.
            Glossary::Array(list) => list
                .iter()
                .flat_map(|example| example.get_example())
                .collect(),
        }
    }
}

/// Represents a structured glossary entry containing type and content information.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct StructuredContent {
    /// The type of content, typically "structured-content".
    r#type: String,
    /// The detailed content of the glossary entry.
    content: Content,
}

impl StructuredContent {
    /// Extracts and returns the glossary meanings from the structured content.
    ///
    /// # Arguments
    ///
    /// * `self` - A reference to the StructuredContent instance.
    ///
    /// # Returns
    ///
    /// A vector of string slices, each representing a glossary meaning extracted from the content.
    pub fn get_glossary(&self) -> Vec<&str> {
        self.content.get_glossary(false)
    }

    /// Extracts and returns the example meanings from the structured content.
    ///
    /// # Arguments
    ///
    /// * `self` - A reference to the StructuredContent instance.
    ///
    /// # Returns
    ///
    /// A vector of string slices, each representing a example meaning extracted from the content.
    pub fn get_example(&self) -> Vec<(String, String)> {
        self.content.get_example(false)
    }
}

/// Represents different forms of content within a structured glossary entry.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(untagged)]
enum Content {
    /// A simple string representation of content.
    String(String),
    /// An array of nested content items.
    Array(Vec<Content>),
    /// A structured content with additional metadata.
    Struct(Box<Struct>),
}

impl Content {
    fn get_text(&self) -> String {
        match self {
            Content::String(str) => str.to_owned(),
            Content::Array(items) => items
                .iter()
                .fold(String::new(), |acc, e| acc + &e.get_text()),
            Content::Struct(glossary_struct) => glossary_struct.get_text(),
        }
    }

    /// Recursively extracts glossary meanings from the content, based on its type.
    ///
    /// # Arguments
    /// * `self` - A reference to the Content enum variant.
    /// * `in_glossary` - A boolean indicating whether we're within a nested glossary context.
    ///
    /// # Returns
    /// A vector of string slices, each representing a glossary meaning extracted from the content.
    fn get_glossary(&self, in_glossary: bool) -> Vec<&str> {
        match self {
            // If the content is a string and we're within a glossary context,
            // return it as part of the meanings. Otherwise, return an empty vector.
            Content::String(str) => {
                if in_glossary {
                    vec![str]
                } else {
                    Vec::new()
                }
            }

            // For arrays, recursively process each item.
            Content::Array(items) => items
                .iter()
                .flat_map(|i| i.get_glossary(in_glossary))
                .collect(),

            // For structured content, delegate to the Struct implementation.
            Content::Struct(glossary_struct) => glossary_struct.get_glossary(in_glossary),
        }
    }

    /// Recursively extracts example meanings from the content, based on its type.
    ///
    /// # Arguments
    /// * `self` - A reference to the Content enum variant.
    /// * `in_example` - A boolean indicating whether we're within a nested example context.
    ///
    /// # Returns
    /// A vector of string slices, each representing a example meaning extracted from the content.
    fn get_example(&self, in_example: bool) -> Vec<(String, String)> {
        match self {
            // If the content is a string and we're within a example context,
            // return it as part of the meanings. Otherwise, return an empty vector.
            Content::String(str) => {
                if in_example {
                    vec![(str.to_owned(), String::new())]
                } else {
                    Vec::new()
                }
            }

            // For arrays, recursively process each item.
            Content::Array(items) => items
                .iter()
                .flat_map(|i| i.get_example(in_example))
                .collect(),

            // For structured content, delegate to the Struct implementation.
            Content::Struct(example_struct) => example_struct.get_example(in_example),
        }
    }
}

/// Represents structured content with additional metadata.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
struct Struct {
    /// The main content of the structured entry.
    content: Option<Content>,

    /// Additional metadata in key-value pairs.
    #[serde(flatten)]
    data: HashMap<String, Value>,
}

impl Struct {
    fn get_text(&self) -> String {
        if self.data_content() == Some("attribution-footnote") {
            return String::new();
        }

        if let Some(content) = &self.content {
            if self.data_content() == Some("example-keyword") {
                return format!("<b>{}</b>", content.get_text());
            }

            if self.data["tag"].as_str() == Some("ruby") {
                if let Content::Array(array) = &content {
                    if array.len() == 2 {
                        return format!(" {}[{}]", array[0].get_text(), array[1].get_text());
                    }
                }
            }

            return content.get_text();
        }

        "".to_owned()
    }

    /// Determines the context of the structured content and delegates to get_glossary accordingly.
    fn get_glossary(&self, in_glossary: bool) -> Vec<&str> {
        match (self.data_content(), &self.content) {
            (Some("glossary"), Some(content)) => content.get_glossary(true),
            (Some("examples"), Some(content)) => content.get_glossary(false),
            (_, Some(content)) => content.get_glossary(in_glossary),
            _ => Vec::new(),
        }
    }

    /// Determines the context of the structured content and delegates to get_example accordingly.
    fn get_example(&self, in_example: bool) -> Vec<(String, String)> {
        if let Some(content) = &self.content {
            let format = Regex::new(r"\] ").unwrap();

            if let Some(data_content) = self.data_content() {
                if data_content == "examples" || data_content == "example-sentence" {
                    if let Content::Array(array) = &content {
                        if array.len() == 2 {
                            let jp = format
                                .replace_all(array[0].get_text().trim(), "]")
                                .into_owned();
                            let en = format
                                .replace_all(array[1].get_text().trim(), "]")
                                .into_owned();

                            return vec![(jp, en)];
                        }
                    }

                    let jp = format
                        .replace_all(content.get_text().trim(), "]")
                        .into_owned();

                    return vec![(jp, String::new())];
                }
            }

            return content.get_example(in_example);
        }

        Vec::new()
    }

    /// Extracts the 'content' value from the data HashMap, if present.
    fn data_content(&self) -> Option<&str> {
        self.data.get("data")?.as_object()?.get("content")?.as_str()
    }
}

impl ConvertableJmnedicData for JmnedictWord {
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
            word.glossary.sort_unstable_by_key(|w| w.order);
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

    /// Tests the parsing of JSON data into a `JmnedictWord` struct and verifies its properties, including kanji, kana, tags, order, id, frequency, and glossary meanings.
    #[test]
    fn word() {
        let data: Vec<JmnedictWord> =
            serde_json::from_str(include_str!("./test_data.json")).unwrap();

        // Verify kanji values
        assert_eq!(data[0].kanji(), "事務的");
        assert_eq!(data[1].kanji(), "事務当局");
        assert_eq!(data[2].kanji(), "事務服");
        assert_eq!(data[3].kanji(), "食べる");

        // Verify kana readings
        assert_eq!(data[0].kana(), "じむてき");
        assert_eq!(data[1].kana(), "じむとうきょく");
        assert_eq!(data[2].kana(), "じむふく");
        assert_eq!(data[3].kana(), "たべる");

        // Verify tags
        assert_eq!(data[0].tags(), ["adj-な".to_owned()].into());
        assert_eq!(data[1].tags(), ["n".to_owned()].into());
        assert_eq!(data[2].tags(), ["n".to_owned()].into());
        assert_eq!(data[3].tags(), ["v1".to_owned(), "vt".to_owned()].into());

        // Verify order values
        assert_eq!(data[0].order(), 1999799);
        assert_eq!(data[1].order(), 1999800);
        assert_eq!(data[2].order(), -200);
        assert_eq!(data[3].order(), 1999800);

        // Verify word IDs
        assert_eq!(data[0].id(), 1314450);
        assert_eq!(data[1].id(), 1314460);
        assert_eq!(data[2].id(), 1314470);
        assert_eq!(data[3].id(), 1358280);

        // Verify frequency tags
        assert_eq!(
            data[0].frequency(),
            ["⭐".to_owned(), "news12k".to_owned()].into()
        );
        assert_eq!(
            data[1].frequency(),
            ["⭐".to_owned(), "news10k".to_owned()].into()
        );
        assert_eq!(data[2].frequency(), [].into());
        assert_eq!(
            data[3].frequency(),
            ["⭐".to_owned(), "ichi".to_owned(), "news13k".to_owned()].into()
        );

        // Verify glossary meanings
        assert_eq!(
            data[0].glossary(),
            ["impersonal", "perfunctory", "robot-like"]
        );
        assert_eq!(data[1].glossary(), ["officials in charge"]);
        assert_eq!(data[2].glossary(), ["work clothes"]);
        assert_eq!(data[3].glossary(), ["to eat"]);

        // Verify examples
        assert_eq!(data[0].example(), []);
        assert_eq!(data[1].example(), []);
        assert_eq!(data[2].example(), []);
        assert_eq!(
            data[3].example(),
            [(
                "もっと果物を食べるべきです。".to_owned(),
                "You should eat more fruit.".to_owned()
            ),]
        );
    }
}
