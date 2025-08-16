use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Type alias for a note ID.
pub type ID = i64;

/// The AnkiNote struct represents a note in the Anki flashcard system.
#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct AnkiNote {
    /// ID of the note. This is optional and will be None if not provided.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub noteId: Option<ID>,

    /// Profile associated with this note. This is also optional.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,

    /// The name of the deck this note belongs to. This is also optional.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deckName: Option<String>,

    /// The model used for this note (e.g., Basic, Cloze, etc.)
    pub modelName: String,
    /// Tags associated with this note. These are represented as strings and can be empty.
    pub tags: Vec<String>,
    /// Fields in the note. Each field has a name and a value.
    pub fields: HashMap<String, String>,

    // Modification time of this note (optional).
    #[serde(rename = "mod")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mod_: Option<ID>,

    /// IDs of the cards associated with this note. These are optional and can be empty.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cards: Option<Vec<ID>>,
    /*
    /// Audio associated with this note (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio: Option<AnkiNoteAudio>, */
}

/* /// The AnkiNoteAudio struct represents the audio data for a note in the Anki flashcard system.
#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct AnkiNoteAudio {
    /// URL of the audio file.
    pub url: String, //https://assets.languagepod101.com/dictionary/japanese/audiomp3.php?kanji=猫&kana=ねこ,
    /// Name of the audio file.
    pub filename: String, //yomichan_ねこ_猫.mp3,
    /// Skip hash for caching (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skipHash: Option<String>,
    // Fields associated with this audio data.
    pub fields: String,
} */
