use std::collections::HashMap;

use reqwest::Method;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::anki::anki_note::ID;

use super::anki_note::AnkiNote;

/// Represents a response containing either a successful result or an error.
type Response<T> = Result<T, Box<dyn std::error::Error>>;

/// Represents the payload data sent in an API request. It includes the action, a fixed version,
/// and any additional parameters required for the action.
#[derive(Serialize)]
#[allow(non_snake_case)]
struct PayloadData<T> {
    /// The action identifier specifying which operation to perform on the server.
    action: String,
    /// Fixed version number (always 5) ensuring compatibility with the api implementation.
    version: u8,

    #[serde(skip_serializing_if = "Option::is_none")]
    apiKey: Option<String>,

    /// Additional parameters specific to the action.
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<T>,
}

impl<T> PayloadData<T> {
    /// Creates a new `PayloadData` instance with the specified action and data payload.
    ///
    /// # Arguments
    /// * `action` - A string slice specifying the action.
    /// * `data` - The data to include in the request, which will be serialized into JSON.
    pub fn new(action: &str, api_key: Option<&String>, data: Option<T>) -> Self {
        PayloadData {
            action: action.to_owned(),
            version: 5,
            apiKey: api_key.cloned(),
            params: data,
        }
    }
}

/// Represents the response structure returned by the server for an API request.
#[derive(Deserialize)]
struct ResponseData<T> {
    /// The deserialized result of a successful API call.
    result: Option<T>,
    /// An error message if the request fails.
    error: Option<String>,
}

pub struct AnkiConnect {
    url: String,
    api_key: Option<String>,
}

const VERSION: u8 = 6;

impl Default for AnkiConnect {
    fn default() -> Self {
        Self {
            url: "http://127.0.0.1:8765".to_owned(),
            api_key: None,
        }
    }
}

impl AnkiConnect {
    pub fn new(url: String, api_key: Option<String>) -> Response<Self> {
        let link = Self { url, api_key };
        let version = link.version()?;

        if version != VERSION {
            Err(Box::from(format!(
                "Expected AnkiConnect version '{}' but got '{}' instead! ",
                VERSION, version
            )))
        } else {
            Ok(link)
        }
    }

    pub fn get_url(&self) -> &str {
        &self.url
    }

    pub fn get_api_key(&self) -> Option<&str> {
        if let Some(api_key) = &self.api_key {
            Some(api_key)
        } else {
            None
        }
    }

    /// Every request consists of a JSON-encoded object containing an action, a version, and a set of
    /// contextual params.
    ///
    /// # Arguments
    /// * `action` - A string slice specifying the action to perform.
    /// * `data` - The data payload to send with the request, which must implement `Serialize`.
    ///
    /// # Returns
    /// * A `Response<T>` where `T` is the deserialized result type if successful, or an error.
    fn invoke<T, U>(&self, action: &str, data: Option<T>) -> Response<U>
    where
        T: Serialize,
        U: DeserializeOwned,
    {
        let client = reqwest::blocking::Client::new();

        let payload =
            serde_json::to_string(&PayloadData::new(action, self.api_key.as_ref(), data))?;
        /* println!("Payload: {}", payload); */

        let response = client
            .request(Method::POST, &self.url)
            .body(payload)
            .send()?;

        let response = response.text()?;
        /* println!("Response: {}", response); */
        let response: ResponseData<U> = serde_json::from_str(response.as_str())?;

        if let Some(err) = response.error {
            return Err(Box::from(err));
        }

        if let Some(response) = response.result {
            return Ok(response);
        }

        Err(Box::from("Somehow did not get a response or error"))
    }

    /// Gets the version of the API exposed by this plugin. Currently versions `1` through `5` are defined.
    ///
    /// This should be the first call you make to make sure that your application and AnkiConnect are
    /// able to communicate properly with each other. New versions of AnkiConnect are backwards
    /// compatible; as long as you are using actions which are available in the reported AnkiConnect
    /// version or earlier, everything should work fine.
    ///
    /// # Returns
    /// * The version of AnkiConnect
    pub fn version(&self) -> Response<u8> {
        let data: Option<()> = None;
        self.invoke("version", data)
    }

    /// Displays a confirmation dialog box in Anki asking the user if they wish to upgrade AnkiConnect
    /// to the latest version from the project's
    /// [master](https://raw.githubusercontent.com/FooSoft/anki-connect/master/AnkiConnect.py)
    /// branch on GitHub. Returns a boolean value indicating if the plugin was upgraded or not.
    ///
    /// # Returns
    /// * Whether or not the plugin was upgraded.
    pub fn upgrade(&self) -> Response<bool> {
        let data: Option<()> = None;
        self.invoke("upgrade", data)
    }

    /// Gets the complete list of deck names for the current user.
    ///
    /// # Returns
    /// * A vector of deck names.
    pub fn deck_names(&self) -> Response<Vec<String>> {
        let data: Option<()> = None;
        self.invoke("deckNames", data)
    }

    /// Gets the complete list of deck names and their respective IDs for the current user.
    ///
    /// # Returns
    /// * A HashMap of deck names and their id's.
    pub fn deck_names_and_ids(&self) -> Response<HashMap<String, ID>> {
        let data: Option<()> = None;
        self.invoke("deckNamesAndIds", data)
    }

    /// Accepts an array of card IDs and returns an object with each deck name as a key,
    /// and its value an array of the given cards which belong to it.
    ///
    /// # Arguments
    /// * `cards` - A slice if card id's to get.
    ///
    /// # Returns
    /// * A HashMap of deck names and the given cards which belong to it.
    pub fn get_decks(&self, cards: &[ID]) -> Response<HashMap<String, Vec<ID>>> {
        self.invoke("getDecks", Some(cards))
    }

    /// Moves cards with the given IDs to a different deck, creating the deck if it doesn't exist yet.
    ///
    /// # Arguments
    /// * `deck` - The deck to move to.
    /// * `card_ids` - A slice of card id's to move.
    ///
    /// # Returns
    /// * A error if there was one.
    pub fn change_deck(&self, deck: &str, cards: &[ID]) {
        let mut data: HashMap<String, serde_json::Value> = HashMap::new();
        data.insert("deck".into(), deck.into());
        data.insert("cards".into(), cards.into());

        let _: Response<()> = self.invoke("changeDeck", Some(data));
    }

    /// Deletes decks with the given names. If `cardsToo` is `true` (defaults to `false if` unspecified),
    /// the cards within the deleted decks will also be deleted; otherwise they will be moved to the
    /// default deck.
    ///
    /// # Arguments
    /// * `decks` - The deck to delete.
    /// * `cards_too` - Whether or not to delete the cards inside the deck.
    ///
    /// # Returns
    /// * A error if there was one.
    pub fn delete_decks(&self, decks: &[&str], cards_too: bool) {
        let mut data: HashMap<String, serde_json::Value> = HashMap::new();
        data.insert("decks".into(), decks.into());
        data.insert("cards_too".into(), cards_too.into());

        let _: Response<()> = self.invoke("deleteDecks", Some(data));
    }

    /// Gets the complete list of model names for the current user.
    ///
    /// # Returns
    /// * A vector of model names.
    pub fn model_names(&self) -> Response<Vec<String>> {
        let data: Option<()> = None;
        self.invoke("modelNames", data)
    }

    /// Gets the complete list of model names and their corresponding IDs for the current user.
    ///
    /// # Returns
    /// * A HashMap of model names and it's id.
    pub fn model_names_and_ids(&self) -> Response<HashMap<String, ID>> {
        let data: Option<()> = None;
        self.invoke("modelNamesAndIds", data)
    }

    /// Gets the complete list of field names for the provided model name.
    ///
    /// # Arguments
    /// * `model_name` - The name of the model to get.
    ///
    /// # Returns
    /// * A vector of the models field.
    pub fn model_field_names(&self, model_name: &str) -> Response<Vec<String>> {
        let mut data: HashMap<String, serde_json::Value> = HashMap::new();
        data.insert("modelName".into(), model_name.into());

        self.invoke("modelFieldNames", Some(data))
    }

    /// Returns an object indicating the fields on the question and answer side of each card template
    /// for the given model name. The question side is given first in each array.
    ///
    /// # Arguments
    /// * `model_name` - The name of the model to get.
    ///
    /// # Returns
    /// * A hashmap of the models card templates and the fields used on each side.
    #[allow(clippy::type_complexity)]
    pub fn model_fields_on_templates(
        &self,
        model_name: &str,
    ) -> Response<HashMap<String, (Vec<String>, Vec<String>)>> {
        let mut data: HashMap<String, serde_json::Value> = HashMap::new();
        data.insert("modelName".into(), model_name.into());

        self.invoke("modelFieldsOnTemplates", Some(data))
    }

    /// Creates a note using the given deck and model, with the provided field values and tags.
    /// Returns the identifier of the created note created on success, and `null` on failure.
    ///
    /// AnkiConnect can download audio files and embed them in newly created notes.
    /// The corresponding audio note member is optional and can be omitted.
    /// If you choose to include it, the `url` and `filename` fields must be also defined.
    /// The `skipHash` field can be optionally provided to skip the inclusion of downloaded files with
    /// an MD5 hash that matches the provided value.
    /// This is useful for avoiding the saving of error pages and stub files.
    /// The `fields` member is a list of fields that should play audio when the card is displayed in
    /// Anki.
    ///
    /// # Arguments
    /// * `note` - The note to add.
    ///
    /// # Returns
    /// * The id of the added note.
    pub fn add_note(&self, note: &mut AnkiNote) -> Response<ID> {
        let mut data: HashMap<String, &AnkiNote> = HashMap::new();
        data.insert("note".into(), note);

        let response: Response<ID> = self.invoke("addNote", Some(data));

        // Apply the id
        if let Ok(response) = response {
            note.noteId = Some(response);
        }

        response
    }

    /// Creates multiple notes using the given deck and model, with the provided field values and tags.
    /// Returns an array of identifiers of the created notes (notes that could not be created will
    /// have a `null` identifier).
    /// Please see the documentation for `addNote` for an explanation of objects in the `notes` array.
    ///
    /// # Arguments
    /// * `notes` - The notes to add.
    ///
    /// # Returns
    /// * A vec of id's of the added notes.
    pub fn add_notes(&self, notes: &mut [AnkiNote]) -> Response<Vec<Option<ID>>> {
        let mut data: HashMap<String, &[AnkiNote]> = HashMap::new();
        data.insert("notes".into(), notes);

        let response: Response<Vec<Option<ID>>> = self.invoke("addNotes", Some(data));

        // Apply the id
        if let Ok(response) = response {
            for (note, id) in notes.iter_mut().zip(response.iter()) {
                note.noteId = *id;
            }

            Ok(response)
        } else {
            response
        }
    }

    /// Accepts an array of objects which define parameters for candidate notes (see addNote) and
    /// returns an array of booleans indicating whether or not the parameters at the corresponding
    /// index could be used to create a new note.
    ///
    /// # Arguments
    /// * `notes` - The notes check.
    ///
    /// # Returns
    /// * The a vector of booleans if the given note can be added.
    pub fn can_add_notes(&self, notes: &mut [AnkiNote]) -> Response<Vec<bool>> {
        let mut data: HashMap<String, &[AnkiNote]> = HashMap::new();
        data.insert("notes".into(), notes);

        self.invoke("canAddNotes", Some(data))
    }

    /// Modify the fields of an exist note.
    ///
    /// # Arguments
    /// * `id` - The id of the note.
    /// * `fields` - The fields and the new data.
    pub fn update_note_fields(&self, id: ID, fields: &HashMap<String, String>) {
        let mut fields_json = serde_json::Map::new();

        for (field, data) in fields {
            fields_json.insert(field.clone(), data.clone().into());
        }

        let mut note: HashMap<String, serde_json::Value> = HashMap::new();
        note.insert("id".into(), id.into());
        note.insert("fields".into(), fields_json.into());

        let mut data: HashMap<String, HashMap<String, serde_json::Value>> = HashMap::new();
        data.insert("note".into(), note);

        let _: Response<()> = self.invoke("updateNoteFields", Some(data));
    }

    /// Modify the fields of an existing note.
    /// You can also include audio, video, or picture files which will be added to the note with
    /// an optional audio, video, or picture property.
    /// Please see the documentation for addNote for an explanation of objects in the audio,
    /// video, or picture array.
    ///
    /// # Arguments
    /// * `id` - The id of the note.
    /// * `url` - The url of the audio file.
    /// * `fields` - The fields to add the audio file to.
    /// * `filename` - Optional new file name.
    /// * `skip_hash` - Optional skip hash.
    pub fn add_note_audio(
        &self,
        id: ID,
        url: &str,
        filename: &str,
        fields: &[&str],
        skip_hash: Option<&str>,
    ) {
        let mut audio_json = serde_json::Map::new();
        audio_json.insert("url".into(), url.into());
        audio_json.insert("fields".into(), fields.into());
        audio_json.insert("filename".into(), filename.into());
        if let Some(skip_hash) = skip_hash {
            audio_json.insert("skipHash".into(), skip_hash.into());
        }

        let mut note: HashMap<String, serde_json::Value> = HashMap::new();
        note.insert("id".into(), id.into());
        note.insert("fields".into(), serde_json::Map::new().into());
        note.insert("audio".into(), audio_json.into());

        let mut data: HashMap<String, HashMap<String, serde_json::Value>> = HashMap::new();
        data.insert("note".into(), note);

        let _: Response<()> = self.invoke("updateNoteFields", Some(data));
    }

    /// Adds tags to notes by note ID.
    ///
    /// # Arguments
    /// * `notes` - The id of the notes.
    /// * `tags` - The tags to be added.
    ///
    /// # Returns
    /// * A error if there was one.
    pub fn add_tags(&self, notes: &[ID], tags: &str) {
        let mut data: HashMap<String, serde_json::Value> = HashMap::new();
        data.insert("notes".into(), notes.into());
        data.insert("tags".into(), tags.into());

        let _: Response<()> = self.invoke("addTags", Some(data));
    }

    /// Remove tags from notes by note ID.
    ///
    /// # Arguments
    /// * `notes` - The id of the notes.
    /// * `tags` - The tags to be removed.
    ///
    /// # Returns
    /// * A error if there was one.
    pub fn remove_tags(&self, notes: &[ID], tags: &str) {
        let mut data: HashMap<String, serde_json::Value> = HashMap::new();
        data.insert("notes".into(), notes.into());
        data.insert("tags".into(), tags.into());

        let _: Response<()> = self.invoke("removeTags", Some(data));
    }

    /// Gets the complete list of tags for the current user.
    ///
    /// # Returns
    /// * A vector of all the tags.
    pub fn get_tags(&self) -> Response<Vec<String>> {
        let data: Option<()> = None;
        self.invoke("getTags", Some(data))
    }

    /// Returns an array of note IDs for a given query. Same query syntax as `guiBrowse`.
    ///
    /// # Arguments
    /// * `query` - The search query.
    ///
    /// # Returns
    /// * A vector of all the note id's.
    pub fn find_notes(&self, query: &str) -> Response<Vec<ID>> {
        let mut data: HashMap<String, serde_json::Value> = HashMap::new();
        data.insert("query".into(), query.into());

        self.invoke("findNotes", Some(data))
    }

    /// Returns a list of objects containing for each note ID the note fields, tags, note type and the
    /// cards belonging to the note.
    ///
    /// # Arguments
    /// * `notes` - The id of the notes.
    ///
    /// # Returns
    /// * A vector of all the notes.
    pub fn notes_info(&self, notes: &[ID]) -> Response<Vec<AnkiNote>> {
        let mut data: HashMap<String, serde_json::Value> = HashMap::new();
        data.insert("notes".into(), notes.into());

        /* self.invoke("notesInfo", Some(data)) */

        let res: Vec<HashMap<String, serde_json::Value>> = self.invoke("notesInfo", Some(data))?;

        let res = res
            .iter()
            .map(|entry| {
                let tags: Option<Vec<String>> = entry["tags"]
                    .as_array()
                    .map(|arr| arr.iter().map(|e| e.as_str().unwrap().to_owned()).collect());

                let fields: Option<HashMap<String, String>> =
                    entry["fields"].as_object().map(|arr| {
                        arr.iter()
                            .map(|(k, v)| (k.clone(), v["value"].as_str().unwrap().to_owned()))
                            .collect()
                    });

                let cards: Option<Vec<i64>> = entry["cards"]
                    .as_array()
                    .map(|arr| arr.iter().map(|e| e.as_i64().unwrap()).collect());

                AnkiNote {
                    noteId: entry["noteId"].as_i64(),
                    profile: entry
                        .get("profile")
                        .and_then(|v| v.as_str().map(|s| s.to_owned())),
                    deckName: entry
                        .get("deckName")
                        .and_then(|v| v.as_str().map(|s| s.to_owned())),
                    modelName: entry["modelName"].as_str().unwrap().to_owned(),
                    tags: tags.unwrap(),
                    fields: fields.unwrap(),
                    mod_: entry["mod"].as_i64(),
                    cards,
                    /* audio: None, */
                }
            })
            .collect();

        Ok(res)
    }

    /// Sets specific value of a single card. Given the risk of wreaking havor in the database
    /// when changing some of the values of a card, some of the keys require the argument
    /// "warning_check" set to True. This can be used to set a card's flag, change it's ease
    /// factor, change the review order in a filtered deck and change the column "data"
    /// (not currently used by anki apparantly), and many other values. A list of values and
    /// explanation of their respective utility can be found at AnkiDroid's wiki.
    ///
    /// # Arguments
    /// * `card` - The id's of the card to be set properties of.
    /// * `properties` - The properties to be set.
    ///
    /// # Returns
    /// * A vector of whether or not the property was successfully set.
    pub fn set_specific_value_of_card(
        &self,
        card: ID,
        properties: Vec<(&str, &serde_json::Value)>,
    ) -> Response<Vec<serde_json::Value>> {
        let mut data: HashMap<String, serde_json::Value> = HashMap::new();
        data.insert("card".into(), card.into());
        data.insert("keys".into(), properties.iter().map(|e| e.0).collect());
        data.insert(
            "newValues".into(),
            properties.iter().map(|e| e.1).cloned().collect(),
        );

        self.invoke("setSpecificValueOfCard", Some(data))
    }

    /// Suspend cards by card ID; returns `true` if successful (at least one card wasn't already
    /// suspended) or `false` otherwise.
    ///
    /// # Arguments
    /// * `cards` - The id's of the cards to be suspended.
    ///
    /// # Returns
    /// * A bool if successful.
    pub fn suspend(&self, cards: &[ID]) -> Response<bool> {
        let mut data: HashMap<String, serde_json::Value> = HashMap::new();
        data.insert("cards".into(), cards.into());

        self.invoke("suspend", Some(data))
    }

    /// Unsuspend cards by card ID; returns `true` if successful (at least one card was previously
    /// suspended) or `false` otherwise.
    ///
    /// # Arguments
    /// * `cards` - The id's of the cards to be unsuspended.
    ///
    /// # Returns
    /// * A bool if successful.
    pub fn unsuspend(&self, cards: &[ID]) -> Response<bool> {
        let mut data: HashMap<String, serde_json::Value> = HashMap::new();
        data.insert("cards".into(), cards.into());

        self.invoke("unsuspend", Some(data))
    }

    /// Returns an array indicating whether each of the given cards is suspended (in the same order).
    ///
    /// # Arguments
    /// * `cards` - The id's to get suspension status of.
    ///
    /// # Returns
    /// * A vec of representing whether or not the given card is suspended.
    pub fn are_suspended(&self, cards: &[ID]) -> Response<Vec<bool>> {
        let mut data: HashMap<String, serde_json::Value> = HashMap::new();
        data.insert("cards".into(), cards.into());

        self.invoke("areSuspended", Some(data))
    }

    /// Returns an array indicating whether each of the given cards is due (in the same order).
    /// _Note_: cards in the learning queue with a large interval (over 20 minutes) are treated as not
    /// due until the time of their interval has passed, to match the way Anki treats them when
    /// reviewing.
    ///
    /// # Arguments
    /// * `cards` - The id's to get due status of.
    ///
    /// # Returns
    /// * A vec of representing whether or not the given card is due.
    pub fn are_due(&self, cards: &[ID]) -> Response<Vec<bool>> {
        let mut data: HashMap<String, serde_json::Value> = HashMap::new();
        data.insert("cards".into(), cards.into());

        self.invoke("areDue", Some(data))
    }

    /// Returns an array of the most recent intervals for each given card ID, or a 2-dimensional array
    /// of all the intervals for each given card ID when complete is true. Negative intervals are in
    /// seconds and positive intervals in days.
    ///
    /// # Arguments
    /// * `cards` - The id's to get the intervall of.
    ///
    /// # Returns
    /// * A 2d vec of representing the given card intervall.
    pub fn get_intervals(&self, cards: &[ID]) -> Response<Vec<Vec<i32>>> {
        let mut data: HashMap<String, serde_json::Value> = HashMap::new();
        data.insert("cards".into(), cards.into());

        self.invoke("getIntervals", Some(data))
    }

    /// Returns an array of card IDs for a given query.
    /// Functionally identical to `guiBrowse` but doesn't use the GUI for better performance.
    ///
    /// # Arguments
    /// * `query` - The search query.
    ///
    /// # Returns
    /// * A vec of coresponding card id's.
    pub fn find_cards(&self, query: &str) -> Response<Vec<ID>> {
        let mut data: HashMap<String, serde_json::Value> = HashMap::new();
        data.insert("query".into(), query.into());

        self.invoke("findCards", Some(data))
    }

    /// Returns an unordered array of note IDs for the given card IDs.
    /// For cards with the same note, the ID is only given once in the array.
    ///
    /// # Arguments
    /// * `cards` - The id's to get the intervall of.
    ///
    /// # Returns
    /// * A vec of note id's.
    pub fn cards_to_notes(&self, cards: &[ID]) -> Response<Vec<ID>> {
        let mut data: HashMap<String, serde_json::Value> = HashMap::new();
        data.insert("cards".into(), cards.into());

        self.invoke("cardsToNotes", Some(data))
    }

    /// Set Due Date.
    /// Turns cards into review cards if they are new, and makes them due on a
    /// certain date.
    ///
    /// # Arguments
    /// * `cards` - The id's to set the due date of.
    /// * `days` - The new due date of the card.
    ///   * 0 = today
    ///   * 1! = tomorrow + change interval to 1
    ///   * 3-7 = random choice of 3-7 days
    ///
    /// # Returns
    /// * Whether or not the due date was changed.
    pub fn set_due_date(&self, cards: &[ID], days: &str) -> Response<bool> {
        let mut data: HashMap<String, serde_json::Value> = HashMap::new();
        data.insert("cards".into(), cards.into());
        data.insert("days".into(), days.into());

        self.invoke("setDueDate", Some(data))
    }

    /// Invokes the _Card Browser_ dialog and searches for a given query.
    /// Returns an array of identifiers of the cards that were found.
    ///
    /// # Arguments
    /// * `query` - The search query.
    ///
    /// # Returns
    /// * A vec of card id's.
    pub fn gui_browse(&self, query: &str) -> Response<Vec<i32>> {
        let mut data: HashMap<String, serde_json::Value> = HashMap::new();
        data.insert("query".into(), query.into());

        self.invoke("guiBrowse", Some(data))
    }

    /// Invokes the _Add Cards_ dialog.
    ///
    /// # Returns
    /// * A vec of card id's.
    pub fn gui_add_cards(&self) {
        let data: Option<()> = None;
        let _: Response<()> = self.invoke("guiAddCards", Some(data));
    }

    /// Opens the _Deck Overview_ dialog for the deck with the given name;
    /// returns `true` if succeeded or `false` otherwise.
    ///
    /// # Arguments
    /// * `name` - The name of the deck.
    ///
    /// # Returns
    /// * A bool representinf if it succeeded or not.
    pub fn gui_deck_overview(&self, name: &str) -> Response<bool> {
        let mut data: HashMap<String, serde_json::Value> = HashMap::new();
        data.insert("name".into(), name.into());

        self.invoke("guiDeckOverview", Some(data))
    }

    /// Opens the _Deck Browser_ dialog.
    pub fn gui_deck_browser(&self) {
        let data: Option<()> = None;
        let _: Response<()> = self.invoke("guiDeckBrowser", Some(data));
    }

    /// Starts review for the deck with the given name;
    /// returns `true` if succeeded or `false` otherwise.
    ///
    /// # Arguments
    /// * `name` - The name of the deck.
    ///
    /// # Returns
    /// * A bool representinf if it succeeded or not.
    pub fn gui_deck_review(&self, name: &str) -> Response<bool> {
        let mut data: HashMap<String, serde_json::Value> = HashMap::new();
        data.insert("name".into(), name.into());

        self.invoke("guiDeckReview", Some(data))
    }

    /// Schedules a request to gracefully close Anki. This operation is asynchronous,
    /// so it will return immediately and won't wait until the Anki process actually terminates.
    pub fn gui_exit_anki(&self) {
        let data: Option<()> = None;
        let _: Response<()> = self.invoke("guiExitAnki", Some(data));
    }
}
