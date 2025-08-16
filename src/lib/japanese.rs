use std::collections::{HashMap, HashSet};

use regex::Regex;

/// Trait for converting between Japanese Katakana and Hiragana characters
pub trait JapaneseChar {
    /// Converts a single Japanese character from Katakana to Hiragana.
    ///
    /// # Notes:
    /// - Works with full-width Katakana (U+30A1-U+30F6).
    /// - Returns same character if not Katakana.
    fn to_hiragana(&self) -> Self;

    /// Converts a single Japanese character from Hiragana to Katakana.
    ///
    /// # Notes:
    /// - Works with full-width Hiragana (U+3041-U+3096).
    /// - Returns same character if not Hiragana.
    fn to_katakana(&self) -> Self;
}

impl JapaneseChar for char {
    /// Converts Katakana to Hiragana by subtracting 96 from Unicode scalar.
    /// Matches full-width Katakana (U+30A1-U+30F6).
    fn to_hiragana(&self) -> Self {
        match self {
            // Matches full-width Katakana characters and converts them to Hiragana
            '\u{30A1}'..='\u{30F6}' => std::char::from_u32(*self as u32 - 96).unwrap(),
            _ => *self,
        }
    }

    /// Converts Hiragana to Katakana by adding 96 to Unicode scalar.
    /// Matches full-width Hiragana (U+3041-U+3096).
    fn to_katakana(&self) -> Self {
        match self {
            // Matches full-width Hiragana characters and converts them to Katakana
            '\u{3041}'..='\u{3096}' => std::char::from_u32(*self as u32 + 96).unwrap(),
            _ => *self,
        }
    }
}

/// Trait for string-level Kana conversions
pub trait JapaneseStr {
    /// Converts string from Katakana to Hiragana.
    fn to_hiragana(&self) -> String;

    /// Converts string from Hiragana to Katakana.
    fn to_katakana(&self) -> String;

    /// Extracts kana readings from kanji-kana pairs in a string.
    fn to_kana(&self) -> String;

    /// Extracts kanji from kanji-kana pairs in a string.
    fn to_kanji(&self) -> String;
}

impl JapaneseStr for &str {
    /// Converts a string from Katakana to Hiragana by applying `to_hiragana` to each character.
    fn to_hiragana(&self) -> String {
        self.chars().map(|c| c.to_hiragana()).collect()
    }

    /// Converts a string from Hiragana to Katakana by applying `to_katakana` to each character.
    fn to_katakana(&self) -> String {
        self.chars().map(|c| c.to_katakana()).collect()
    }

    /// Extracts kana readings using regex pattern.
    fn to_kana(&self) -> String {
        let regex = Regex::new(r" ?(?<kanji>[^\s\[\]]+?)\[(?<kana>[^\s\[\]]+?)\]").unwrap();
        regex.replace_all(self, "${kana}").to_string()
    }

    /// Extracts kanji using regex pattern.
    fn to_kanji(&self) -> String {
        let regex = Regex::new(r" ?(?<kanji>[^\s\[\]]+?)\[(?<kana>[^\s\[\]]+?)\]").unwrap();
        regex.replace_all(self, "${kanji}").to_string()
    }
}

impl JapaneseStr for String {
    /// Converts a String from Katakana to Hiragana by applying `to_hiragana` to each character.
    fn to_hiragana(&self) -> String {
        self.chars().map(|c| c.to_hiragana()).collect()
    }

    /// Converts a String from Hiragana to Katakana by applying `to_katakana` to each character.
    fn to_katakana(&self) -> String {
        self.chars().map(|c| c.to_katakana()).collect()
    }

    /// Extracts kana readings using regex pattern.
    fn to_kana(&self) -> String {
        let regex = Regex::new(r" ?(?<kanji>[^\s\[\]]+?)\[(?<kana>[^\s\[\]]+?)\]").unwrap();
        regex.replace_all(self, "${kana}").to_string()
    }

    /// Extracts kanji using regex pattern.
    fn to_kanji(&self) -> String {
        let regex = Regex::new(r" ?(?<kanji>[^\s\[\]]+?)\[(?<kana>[^\s\[\]]+?)\]").unwrap();
        regex.replace_all(self, "${kanji}").to_string()
    }
}

/// Parses a kanji reading string into its components: prefix, main reading, okurigana, and suffix.
///
/// The input string follows the format:
/// - readings (with '-' to indicate prefixes/suffixes, and '.' to separate a reading from its okurigana)
///
/// # Return Value
///
/// An `Option` containing a tuple with the following fields:
///
/// - `prefix`: `Option<String>` - The prefix part of the reading, if present.
/// - `reading`: `String` - The main part of the reading.
/// - `okurigana`: `Option<String>` - The okurigana part of the reading, if present.
/// - `suffix`: `Option<String>` - The suffix part of the reading, if present.
///
/// If the input string does not match the expected format, returns `None`.
#[allow(clippy::type_complexity)]
pub fn split_kanji_reading(
    reading: &str,
) -> Option<(Option<String>, String, Option<String>, Option<String>)> {
    // Regex breakdown:
    // - `^` : Start of the string.
    // - `(?:(?<prefix>[^\-. 	]*)-)?` : Captures an optional prefix consisting of non-special characters followed by a hyphen.
    // - `(?<reading>[^\-. 	]+)` : Captures the main reading part (required) as one or more non-special characters.
    // - `(?:\.(?<okurigana>[^\-. 	]+))?` : Captures an optional okurigana following a dot.
    // - `(?:-(?<suffix>[^\-. 	]*))?` : Captures an optional suffix following a hyphen at the end of the string.
    // - `$` : End of the string.
    let re = Regex::new(r"^(?:(?<prefix>[^\-. 	]*)-)?(?<reading>[^\-. 	]+)(?:\.(?<okurigana>[^\-. 	]+))?(?:-(?<suffix>[^\-. 	]*))?$").unwrap();

    // Attempt to match the reading string
    let captures = re.captures(reading)?;

    // Extract matched groups into Option<String> values
    let reading = captures.name("reading").map(|x| x.as_str().to_owned())?;
    let prefix = captures.name("prefix").map(|x| x.as_str().to_owned());
    let okurigana = captures.name("okurigana").map(|x| x.as_str().to_owned());
    let suffix = captures.name("suffix").map(|x| x.as_str().to_owned());

    Some((prefix, reading, okurigana, suffix))
}

/// Converts a given kanji word into its furigana representation using kana readings.
///
/// The function processes both kanji and kana strings to map each kanji character to its corresponding
/// reading from the provided `kanji_readings` HashMap. It then constructs a string where each kanji is
/// paired with its reading in brackets, separated by spaces as needed.
///
/// # Arguments
/// * `kanji`: The kanji representation of the word (e.g., "気の毒").
/// * `kana`: The kana representation of the same word (e.g., "きのどく").
/// * `kanji_readings`: A HashMap mapping each kanji character to a set of possible readings.
///
/// # Return Value
/// Returns an Option containing the furigana string if successful, or None if no valid mapping is found.
pub fn to_furigana(
    kanji: &str,
    kana: &str,
    kanji_readings: &HashMap<char, HashSet<String>>,
) -> Option<String> {
    // Creates blocks of tuples containing (kanji character, whether it's a kanji, and its possible readings in parentheses)
    let blocks: Vec<(String, bool, String)> = kanji
        .chars()
        .map(|char| {
            // For each kanji character:
            // 1. Look up its possible readings in the kanji_readings map
            // 2. If found, join them with "|" as a fallback option for matching
            // 3. Return a tuple of (kanji, is_kanji, reading_options)
            if let Some(readings) = kanji_readings.get(&char) {
                let joined = readings.iter().fold(String::default(), |mut acc, a| {
                    if acc.is_empty() {
                        acc += a;
                    } else {
                        acc += &format!("|{}", a);
                    }
                    acc
                });

                (char.to_string(), true, format!("({})", joined))
            } else {
                (char.to_string(), false, format!("({})", char))
            }
        })
        .collect();

    // First attempt to match without adding any wildcards
    if let Some(out) = to_furigana_blocks_check(kana, &blocks) {
        return Some(out);
    }

    // Reconstruct blocks to check grouped kanji
    let regex = Regex::new(r"\(").ok()?;
    let mut preceded_by_kanji = !blocks.first()?.1;
    let blocks: Vec<(String, bool, String)> = blocks
        .into_iter()
        .fold(
            Vec::new(),
            |mut out: Vec<(String, bool, String)>, (kanji, is_kanji, kana)| {
                if is_kanji == preceded_by_kanji {
                    let last = out.last_mut().unwrap();
                    last.0 += &kanji;
                    last.2 += &kana;
                } else {
                    out.push((kanji, is_kanji, kana));
                }

                preceded_by_kanji = is_kanji;
                out
            },
        )
        .into_iter()
        .map(|(kanji, is_kanji, kana)| {
            let kana = format!("({})", regex.replace_all(&kana, "(?:"));

            (kanji, is_kanji, kana)
        })
        .collect();

    // Try again with the reconstructed blocks
    to_furigana_blocks_check(kana, &blocks)
}

/// Helper function that checks if the constructed regex from `to_furigana` matches the given kana string.
///
/// # Arguments:
/// * `kana`: The kana string to match against.
/// * `blocks`: A slice of tuples containing (kanji character, whether it's a kanji, and its possible readings in parentheses).
///
/// # Returns:
/// An Option containing the furigana string if a valid mapping is found, otherwise None.
fn to_furigana_blocks_check(kana: &str, blocks: &[(String, bool, String)]) -> Option<String> {
    // Tries each block as a possible wildcard position for regex construction
    for wildcard_index in 0..=blocks.len() {
        if wildcard_index != 0 && !blocks[wildcard_index - 1].1 {
            continue;
        }

        // Construct regex pattern with wildcards at specified index
        let regex: String = blocks
            .iter()
            .enumerate()
            .map(|(i, (_, _, kana))| {
                if i + 1 == wildcard_index {
                    "(.+)" // Wildcard for any characters at this position
                } else {
                    kana // Use the exact pattern otherwise
                }
            })
            .fold(String::new(), |a, b| a + b);

        // Create and validate the regex pattern
        let regex = Regex::new(&format!("^{}$", regex)).ok()?;

        // Check if the kana string matches the generated pattern
        if let Some(captures) = regex.captures(kana) {
            // Build the final furigana string from captures
            let mut captures = captures.iter();
            captures.next().unwrap()?;

            let mut out = String::new();
            let mut preceded_by_kanji = true;

            for (kanji, is_kanji, _) in blocks {
                // Get the corresponding kana reading from captures
                let kana: &str = captures.next().unwrap()?.as_str();

                // If current character is kanji:
                // - Add a space before if previous was not kanji
                // - Append kanji[reading] to the output string
                if *is_kanji {
                    if !preceded_by_kanji {
                        out += " ";
                    }

                    out += &format!("{}[{}]", kanji, kana);
                } else {
                    // If current character is not kanji:
                    out += kanji;
                }

                // Update the flag for whether we're preceded by a kanji
                preceded_by_kanji = *is_kanji;
            }
            // Return the constructed furigana string
            return Some(out);
        }
    }

    // If no valid pattern is found, return None
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Hiragana character sets for testing conversions
    const HIRAGANA_DATA: &str = "ぁあぃいぅうぇえぉおかがきぎくぐけげこごさざしじすずせぜそぞただちぢっつづてでとどなにぬねのはばぱひびぴふぶぷへべぺほぼぽまみむめもゃやゅゆょよらりるれろゎわゐゑをんゔゕゖ";
    /// Katakana character sets for testing conversions
    const KATAKANA_DATA: &str = "ァアィイゥウェエォオカガキギクグケゲコゴサザシジスズセゼソゾタダチヂッツヅテデトドナニヌネノハバパヒビピフブプヘベペホボポマミムメモャヤュユョヨラリルレロヮワヰヱヲンヴヵヶ";

    /// Tests character-level conversion between Hiragana and Katakana
    #[test]
    fn convert_char() {
        let hiragana = HIRAGANA_DATA.chars();
        let katakana = KATAKANA_DATA.chars();

        // Test each corresponding pair of characters
        for (hira, kata) in hiragana.zip(katakana) {
            // Convert Katakana to Hiragana and ensure it matches the expected Hiragana character
            assert_eq!(hira, kata.to_hiragana());

            // Convert Hiragana to Katakana and ensure it matches the expected Katakana character
            assert_eq!(kata, hira.to_katakana());
        }
    }

    /// Tests string-level conversion between Hiragana and Katakana
    #[test]
    fn convert_string() {
        let hiragana = HIRAGANA_DATA;
        let katakana = KATAKANA_DATA;

        // Convert Katakana string to Hiragana and ensure it matches the expected Hiragana string
        assert_eq!(hiragana, katakana.to_hiragana());

        // Converting Hiragana string to Katakana should return the same string
        assert_eq!(katakana, hiragana.to_katakana());

        assert_eq!(
            "きけのどくきょうとっきゅう",
            "気[き]気[け]の 毒[どく] 今日[きょう] 特[とっ]急[きゅう]".to_kana()
        );

        assert_eq!(
            "気気の毒今日特急",
            "気[き]気[け]の 毒[どく] 今日[きょう] 特[とっ]急[きゅう]".to_kanji()
        );
    }

    #[test]
    fn test_split_kanji_reading() {
        // Test basic case with no prefix or suffix
        assert_eq!(
            split_kanji_reading("ばら"),
            Some((None, "ばら".to_owned(), None, None)),
        );

        // Test case with a prefix and okurigana
        assert_eq!(
            split_kanji_reading("ち.らす"),
            Some((None, "ち".to_owned(), Some("らす".to_owned()), None)),
        );

        // Test case with a leading hyphen indicating a prefix
        assert_eq!(
            split_kanji_reading("-ち.らす"),
            Some((
                Some("".to_owned()),
                "ち".to_owned(),
                Some("らす".to_owned()),
                None
            )),
        );

        // Test case ending with a hyphen indicating a suffix
        assert_eq!(
            split_kanji_reading("ひと-"),
            Some((None, "ひと".to_owned(), None, Some("".to_owned())))
        );
    }

    /// Tests the `to_furigana` function with various input cases.
    ///
    /// Includes examples where:
    /// - Multiple kanji have different possible readings.
    /// - The function correctly identifies and pairs each kanji with its reading.
    /// - The output format includes spaces between tokens as needed.
    #[test]
    fn test_to_furigana() {
        let mut kanji_readings: HashMap<char, HashSet<String>> = HashMap::new();
        kanji_readings.insert('気', ["き".to_owned(), "け".to_owned()].into());
        kanji_readings.insert('毒', ["どく".to_owned()].into());
        kanji_readings.insert(
            '今',
            ["こん".to_owned(), "きん".to_owned(), "いま".to_owned()].into(),
        );
        kanji_readings.insert(
            '日',
            [
                "にち".to_owned(),
                "じつ".to_owned(),
                "ひ".to_owned(),
                "び".to_owned(),
                "か".to_owned(),
            ]
            .into(),
        );
        kanji_readings.insert('特', ["とく".to_owned()].into());
        kanji_readings.insert(
            '急',
            ["きゅう".to_owned(), "いそ".to_owned(), "せ".to_owned()].into(),
        );
        kanji_readings.insert(
            '建',
            [
                "けん".to_owned(),
                "こん".to_owned(),
                "だ".to_owned(),
                "た".to_owned(),
            ]
            .into(),
        );
        kanji_readings.insert(
            '物',
            ["ぶつ".to_owned(), "もつ".to_owned(), "もの".to_owned()].into(),
        );

        // Test case 1: Simple kanji with single reading
        assert_eq!(
            to_furigana("気の毒", "きのどく", &kanji_readings),
            Some("気[き]の 毒[どく]".to_owned())
        );

        // Test case 2: Multiple readings for a kanji
        assert_eq!(
            to_furigana("気気の毒", "きけのどく", &kanji_readings),
            Some("気[き]気[け]の 毒[どく]".to_owned())
        );

        // Test case 3: Non standard readings for a kanji
        assert_eq!(
            to_furigana("今日", "きょう", &kanji_readings),
            Some("今日[きょう]".to_owned())
        );

        assert_eq!(
            to_furigana("特急", "とっきゅう", &kanji_readings),
            Some("特[とっ]急[きゅう]".to_owned())
        );

        assert_eq!(
            to_furigana("建物", "たてもの", &kanji_readings),
            Some("建[たて]物[もの]".to_owned())
        );
    }
}
