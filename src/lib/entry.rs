use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::japanese::{split_kanji_reading, JapaneseStr};

/// Represents a Japanese word with its associated data.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Word {
    /// Unique identifier for the word
    pub word_id: i32,
    /// Furigana representation of the word (kanji + kana in furigana format)
    pub furigana: String,
    /// List of glossary entries containing meaning and tags
    pub glossary: Vec<Glossary>,
    /// Set of frequency tags associated with the word
    pub frequency: HashSet<String>,
    /// Set of example sentences
    pub examples: HashSet<Example>,
}

impl Word {
    /// Creates a new Word instance.
    pub fn new(
        word_id: i32,
        furigana: String,
        glossary: Vec<Glossary>,
        frequency: HashSet<String>,
        examples: HashSet<Example>,
    ) -> Self {
        Self {
            word_id,
            furigana,
            glossary,
            frequency,
            examples,
        }
    }

    /// Returns a set of all tags from the word's glossary entries.
    ///
    /// # Description
    /// This function aggregates all unique tags from each glossary entry associated with the word,
    /// ensuring that each tag appears only once. The use of `HashSet` guarantees efficient lookup and
    /// membership testing, as well as automatic deduplication of tags.
    ///
    /// # Return Value
    /// A `HashSet<&str>` containing all unique tags extracted from the word's glossaries. Each tag is a string slice,
    /// providing an immutable reference to the original string data.
    pub fn get_all_tags(&self) -> HashSet<&str> {
        let mut out: HashSet<&str> = HashSet::new();

        out.extend(self.frequency.iter().map(|tag| tag.as_str()));

        for glossary in self.glossary.iter() {
            out.extend(glossary.tags.iter().map(|tag| tag.as_str()));
        }

        out
    }
}

/// Represents a glossary entry containing meaning and tags.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Glossary {
    /// Order of the glossary entry
    pub order: i32,
    /// Set of tags associated with the glossary entry
    pub tags: HashSet<String>,
    /// List of meanings for the word
    pub meaning: Vec<String>,
}

impl Glossary {
    /// Creates a new Glossary entry.
    pub fn new(order: i32, tags: HashSet<String>, meaning: Vec<String>) -> Self {
        Self {
            order,
            tags,
            meaning,
        }
    }
}

/// Represents an example sentence with its Japanese and English translations.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Example {
    /// Japanese text of the example
    pub japanese: String,
    /// English translation of the example sentence
    pub english: String,
}

impl Example {
    /// Creates a new Example instance.
    pub fn new(japanese: String, english: String) -> Self {
        Self { japanese, english }
    }
}

/// Represents a kanji character with its associated data.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Kanji {
    /// The kanji character
    pub kanji: char,
    /// Onyomi (Chinese-derived) readings
    pub onyomi: HashSet<String>,
    /// Kunyomi (Japanese-derived) readings
    pub kunyomi: HashSet<String>,
    /// List of meanings for the kanji
    pub meaning: Vec<String>,
    /// Number of strokes required to write the kanji (if available)
    pub strokes: Option<u8>,
    /// Set of tags associated with the kanji
    pub tags: HashSet<String>,
}

impl Kanji {
    /// Creates a new Kanji instance.
    pub fn new(
        kanji: char,
        onyomi: HashSet<String>,
        kunyomi: HashSet<String>,
        meaning: Vec<String>,
        strokes: Option<u8>,
        tags: HashSet<String>,
    ) -> Self {
        Self {
            kanji,
            onyomi,
            kunyomi,
            meaning,
            strokes,
            tags,
        }
    }

    /// Returns a set of all possible readings for the kanji.
    ///
    /// The readings are returned as Hiragana strings, derived from both Onyomi and Kunyomi readings,
    /// with any trailing comments or notes removed.
    pub fn readings(&self) -> HashSet<String> {
        let mut out: HashSet<String> = HashSet::new();

        // Process onyomi
        out.extend(
            self.onyomi
                .iter()
                .filter_map(|r| Some(split_kanji_reading(r)?.1))
                .map(|r| r.to_hiragana()),
        );

        // Process kunyomi
        out.extend(
            self.kunyomi
                .iter()
                .filter_map(|r| Some(split_kanji_reading(r)?.1))
                .map(|r| r.to_hiragana()),
        );

        out
    }
}
