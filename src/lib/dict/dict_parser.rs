use std::{
    collections::{HashMap, HashSet},
    fs::{self, File},
    io::{self, Read},
    path::Path,
};

use serde::de::DeserializeOwned;
use zip::ZipArchive;

use crate::entry::{Kanji, Word};

/// Parses a JSON string into a vector of objects of type T.
///
/// This function takes a string slice containing JSON data and parses it
/// into a collection of objects. The parsing is done using the `serde_json`
/// library, which automatically converts the JSON structure into Rust
/// types based on the provided type information.
///
/// # Type Parameters
/// - `T`: The type of objects to parse from the JSON string. This type must implement
///   `DeserializeOwned` trait.
///
/// # Arguments
/// - `data`: A string slice containing the JSON data to be parsed.
///
/// # Returns
/// - `Option<Vec<T>>`: An `Option` wrapping a vector of parsed objects if successful,
///   or `None` if parsing fails.
pub fn parse_bank<T>(data: &str) -> Option<Vec<T>>
where
    T: DeserializeOwned,
{
    serde_json::from_str(data).ok()
}

/// Parses a ZIP file into a vector of objects of type T.
///
/// This function reads a ZIP archive from the specified path, iterates over its entries,
/// and parses each relevant entry into a collection of objects. The function skips entries
/// that do not start with "term_", "kanji_", or have an unknown filename.
///
/// The parsing process involves reading each entry's content as a JSON string and converting it
/// into Rust objects using the `serde_json` library. These objects are then collected into a vector
/// which is returned upon completion.
///
/// # Type Parameters
/// - `T`: The type of objects to parse from the ZIP file entries. This type must implement
///   `DeserializeOwned` trait, allowing it to be deserialized from JSON data.
///
/// # Arguments
/// - `path`: A reference to a `Path` specifying the location of the ZIP file.
///
/// # Returns
/// - `io::Result<Vec<T>>`: A result containing a vector of parsed objects if successful,
///   or an error if any I/O operation fails.
///
/// # Notes
/// - **Entry Filtering**: The function filters ZIP entries based on their filenames. It processes
///   entries whose filenames start with "term_" or "kanji_", as well as those named "UNKNOWN".
///   All other entries are ignored.
/// - **Progress Feedback**: During processing, the function prints progress information to the console,
///   indicating which files are being processed and whether they were skipped.
/// - **Error Handling**: If any I/O operation fails (e.g., file not found, read error), the function
///   returns an `io::Result` with the corresponding error. It does not handle errors internally beyond
///   propagating them up.
///
/// # Panics
/// The function does not panic but returns errors for handling by the caller.
pub fn parse_zipfile<T>(path: &Path) -> io::Result<Vec<T>>
where
    T: DeserializeOwned,
{
    println!("Reading file {}:", path.to_str().unwrap());

    // Open the ZIP archive and create a mutable ZipArchive instance.
    let mut zip_archive = ZipArchive::new(File::open(path)?)?;

    // Retrieve the total number of entries in the ZIP archive to track progress.
    let bank_count = zip_archive.len();
    let mut data: Vec<T> = Vec::new();

    // Iterate over each entry in the ZIP archive by index.
    for index in 0..bank_count {
        // Access the current entry by its index.
        let mut bank = zip_archive.by_index(index).unwrap();

        // Retrieve the enclosed name (path within the ZIP) of the entry.
        let path = bank.enclosed_name().unwrap();
        let filename = path.file_name().unwrap().to_str().unwrap_or("UNKNOWN");
        let path = path.to_str().unwrap_or("UNKNOWN");

        // Skip entries that do not match the expected patterns or are unknown.
        if !(filename.starts_with("term_")
            || filename.starts_with("kanji_")
            || filename == "UNKNOWN")
        {
            println!("  {}/{} File: {} [IGNORED]", index, bank_count, path);

            continue;
        } else {
            // Print the processing status for valid entries.
            println!("  {}/{} File: {}", index, bank_count, path);
        }

        // Read the contents of the entry into a string.
        let mut raw_data = String::new();
        bank.read_to_string(&mut raw_data)?;

        // Parse the JSON string into objects of type T and append to data.
        data.append(&mut parse_bank::<T>(&raw_data).unwrap());
    }

    // Print a completion message after processing all entries.
    println!();

    Ok(data)
}

/// Parses a directory containing ZIP files into a vector of objects of type T.
///
/// This function reads all files in the specified directory, treating each file as a ZIP
/// archive. For each ZIP file, it calls `parse_zipfile` to extract data and aggregates all
/// parsed objects into a single vector.
///
/// # Type Parameters
/// - `T`: The type of objects to parse from the ZIP files. This type must implement
///   `DeserializeOwned` trait.
///
/// # Arguments
/// - `path`: A reference to a `Path` specifying the location of the directory containing
///   ZIP files.
///
/// # Returns
/// - `io::Result<Vec<T>>`: A result containing a vector of parsed objects if successful,
///   or an error if any I/O operation fails.
pub fn parse_directory<T>(path: &Path) -> io::Result<Vec<T>>
where
    T: DeserializeOwned,
{
    // Read all entries in the specified directory.
    let paths = fs::read_dir(path)?;

    // Initialize an empty vector to collect parsed data.
    let mut data: Vec<T> = Vec::new();

    // Process each entry in the directory.
    for path in paths {
        // For each entry, parse the ZIP file and append its data.
        data.append(&mut parse_zipfile::<T>(&path?.path())?);
    }

    Ok(data)
}

/// Trait for converting JMnedict data into structured formats.
pub trait ConvertableJmnedicData {
    /// Converts Kanji data from the entry into a HashMap.
    ///
    /// # Arguments
    /// * `kanji` - Mutable reference to the HashMap collecting Kanji information.
    ///
    /// # Returns
    /// * `Result<(), String>` - Ok(()) if successful, Err(message) if an error occurs.
    fn convert_kanji_data(&self, kanji: &mut HashMap<char, Kanji>) -> Result<(), String>;

    /// Converts Word data from the entry into a HashMap.
    ///
    /// # Arguments
    /// * `words` - Mutable reference to the HashMap collecting word information.
    /// * `kanji_readings` - Precomputed readings of all kanji characters.
    ///
    /// # Returns
    /// * `Result<(), String>` - Ok(()) if successful, Err(message) if an error occurs.
    fn convert_word_data(
        &self,
        words: &mut HashMap<(String, String), Word>,
        kanji_readings: &HashMap<char, HashSet<String>>,
    ) -> Result<(), String>;
}

/// Converts JMnedict data into a structured format containing Kanji and Word information.
///
/// This function processes an array of Jmnedict entries and constructs two HashMaps:
/// - `kanji`: Maps each Kanji character to its detailed information including readings and meanings
/// - `words`: Maps word furigana representations to their information including glossary, frequency, and examples. Furigana is constructed by combining the kanji and kana parts.
///
/// # Arguments
/// * `data` - A slice of Jmnedict entries containing Kanji and Word/Frequency data
///
/// # Returns
/// * `(HashMap<char, Kanji>, HashMap<String, Word>)`
///   - First map: Kanji character to Kanji info
///   - Second map: Furigana string to Word info  
pub fn convert_data<T>(data: &[T]) -> (HashMap<char, Kanji>, HashMap<String, Word>)
where
    T: ConvertableJmnedicData,
{
    // Print initial conversion message
    println!("Converting data:");

    // Converts and formats all kanji entries from the data
    let kanji = convert_kanji_data(data);

    // Converts and formats word data using precomputed kanji readings
    let words = convert_word_data(&kanji, data);

    (kanji, words)
}

/// Converts Kanji data from the entries into a HashMap.
///
/// This function processes each Jmnedict entry and constructs a HashMap that maps
/// each Kanji character to its detailed information, including readings and meanings.
///
/// # Arguments
/// * `data` - A slice of Jmnedict entries containing Kanji data
///
/// # Returns
/// * `HashMap<char, Kanji>` - Maps each Kanji character to its detailed information
pub fn convert_kanji_data<T>(data: &[T]) -> HashMap<char, Kanji>
where
    T: ConvertableJmnedicData,
{
    // Process and collect Kanji information from the entries
    println!("Converting Kanji:");
    let mut kanji: HashMap<char, Kanji> = HashMap::new();
    for (count, entry) in data.iter().enumerate() {
        // Print progress for every 5% of total entries or at the end
        if count % (data.len() / 20) == 0 {
            println!(
                "  {:>3}% kanji",
                ((count as f32 / data.len() as f32) * 100.0).round()
            );
        }

        let _ = entry.convert_kanji_data(&mut kanji);
    }

    kanji
}

/// Converts Word data from the entries into a HashMap.
///
/// This function processes each Jmnedict entry and constructs a HashMap that maps
/// word furigana representations to their detailed information including glossary, frequency,
/// and examples. Furigana is constructed by combining the kanji and kana parts of words.
///
/// # Arguments
/// * `kanji` - A precomputed HashMap that maps each Kanji character to its detailed info
/// * `data` - A slice of Jmnedict entries containing Word data
///
/// # Returns
/// * `HashMap<String, Word>` - Maps word furigana representations to their detailed information
pub fn convert_word_data<T>(kanji: &HashMap<char, Kanji>, data: &[T]) -> HashMap<String, Word>
where
    T: ConvertableJmnedicData,
{
    // Process and collect Word information from the entries
    println!("\nConverting Words:");
    let mut words: HashMap<(String, String), Word> = HashMap::new();

    // Create a map of Kanji readings for later use in word processing
    let kanji_readings: HashMap<char, HashSet<String>> = kanji
        .values()
        .map(|kanji| (kanji.kanji, kanji.readings()))
        .collect();

    // Iterate through each entry to build Word data
    for (count, entry) in data.iter().enumerate() {
        // Print progress for every 5% of total entries or at the end
        if count % (data.len() / 20) == 0 {
            println!(
                "  {:>3}% words",
                ((count as f32 / data.len() as f32) * 100.0).round()
            );
        }

        // Add or update the word data for a word entry
        let result = entry.convert_word_data(&mut words, &kanji_readings);

        if let Err(message) = result {
            // Handle unrecognized entry types
            println!("{}", message);
            panic!()
        }
    }

    // Sorting glossary
    for (_, word) in words.iter_mut() {
        word.glossary.sort_unstable_by_key(|d| -d.order);
    }

    // Print a blank line after Words processing
    println!();

    // Convert word map from (kanji, kana) tuples to Furigana strings
    let words: HashMap<String, Word> = words
        .clone()
        .into_values()
        .map(|word| (word.furigana.clone(), word))
        .collect();

    // Return the processed Kanji and Words data
    words
}
