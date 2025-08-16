use std::{
    collections::{HashMap, HashSet},
    fmt, thread,
    time::Duration,
};

use reqwest::Method;
use serde::{Deserialize, Serialize};

/// Limit according to sentence origin. All sentences fall in two sets: *unknown* and *known*.
/// The set *known* is composed of two subsets: *original* + *translation*.
///
/// **Allowed:** `original` | `translation` | `known` | `unknown`
///
/// # Examples:
/// * `original` (sentences not added as translations of other sentences)
/// * `translation` (sentences added as translations of other sentences)
/// * `known` (sentences we know have been added or not as translations of other sentences)
/// * `unknown` (sentences we do not know whether or not they have been added as translations of other sentences)
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TatoebaOrigin {
    /// sentences not added as translations of other sentences
    Original,
    /// sentences added as translations of other sentences
    Translation,
    /// sentences we know have been added or not as translations of other sentences
    Known,
    /// sentences we do not know whether or not they have been added as translations of other sentences
    Unknown,
}

impl fmt::Display for TatoebaOrigin {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let str = match self {
            TatoebaOrigin::Original => "original",
            TatoebaOrigin::Translation => "translation",
            TatoebaOrigin::Known => "known",
            TatoebaOrigin::Unknown => "unknown",
        };

        write!(f, "{str}")
    }
}

/// Sort order of the sentences. Prefix the value with minus - to reverse that order.
///
/// **Pattern:** `-?(relevance|words|created|modified|random)`
///
/// # Examples:
/// * `relevance` (prioritize sentences with exact matches, then sentences containing all the searched words, then shortest sentences)
/// * `words` (order by number of words (or, if the language does not use spaces as word separators, by number of characters), shortest first)
/// * `-words` (order by number of words, longest first)
/// * `created` (order by sentence creation date (newest first))
/// * `-created` (order by sentence creation date (oldest first))
/// * `modified` (order by last sentence modification (last modified first))
/// * `random` (randomly sort results)
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TatoebaSort {
    /// `relevance` (prioritize sentences with exact matches, then sentences containing all the searched words, then shortest sentences)
    Relevance,
    /// `words` (order by number of words (or, if the language does not use spaces as word separators, by number of characters), shortest first)
    Shortest,
    /// `-words` (order by number of words, longest first)
    Longest,
    /// `created` (order by sentence creation date (newest first))
    Newest,
    /// `-created` (order by sentence creation date (oldest first))
    Oldest,
    /// `modified` (order by last sentence modification (last modified first))
    Modified,
    /// `random` (randomly sort results)
    Random,
}

impl fmt::Display for TatoebaSort {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let str = match self {
            TatoebaSort::Relevance => "relevance",
            TatoebaSort::Shortest => "words",
            TatoebaSort::Longest => "-words",
            TatoebaSort::Newest => "created",
            TatoebaSort::Oldest => "-created",
            TatoebaSort::Modified => "modified",
            TatoebaSort::Random => "random",
        };

        write!(f, "{str}")
    }
}

/// https://api.tatoeba.org/unstable#?route=get-/unstable/sentences
///
/// Allows to search for sentences based on some criteria. By default, all sentences are returned,
/// including sentences you might want to filter out, such as unapproved or orphaned
/// (that is, likely not proofread) ones.
/// To filter sentences, use any combination of the parameters described below.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TatoebaSearch {
    //q: Option<String>,//The search query. The query must follow ManticoreSearch query syntax.
    //after: Option<String>,//Cursor start position. This parameter is used to paginate results using keyset pagination method. After fetching the first page, if there are more results, you get a cursor_end value along with the results. To get the second page of results, execute the same query with the added after=<cursor_end> parameter. If there are more results, the second page will containg another cursor_end you can use to get the third page, and so on.
    /// A comma-separated list of languages to search in.
    ///
    /// # Examples:
    /// * epo (sentences in Esperanto)
    /// * epo,sun (sentences in Esperanto or Sundanese)
    pub lang: HashSet<String>,

    /// Limit to sentences having the provided number of words. For languages with word boundaries,
    /// the number of words is used. For other languages, the number of characters is used.
    ///
    /// **Constraints:** Min length: 1<br>
    /// **Pattern:** `!?([0-9]+-[0-9]+|[0-9]+-|-[0-9]+)(,([0-9]+-[0-9]+|[0-9]+-|-[0-9]+))*`
    ///
    /// # Examples:
    /// * `10-` (10 words or more)
    /// * `-10` (10 words or less)
    /// * `5-10` (between 5 and 10 words)
    /// * `7` (exactly 7 words)
    /// * `!3` (any number of words but 3)
    /// * `1,10` (either 1 or 10 words)
    /// * `2-4,10-11` (2, 3, 4, 10 or 11 words)
    /// * `!2-` (1 word only)
    /// * `!2-5` (1 word, or more than 5)
    /// * `!2-5,10-` (1 word, or between 6 and 9 words)
    pub word_count: (Option<usize>, Option<usize>),

    /// Limit to sentences owned by the provided username.
    /// Make sure to combine with is_orphan filter in a way that makes sense.
    ///
    /// **Pattern:** `!?[0-9a-zA-Z_]*(,[0-9a-zA-Z_]*)*`
    ///
    /// # Examples:
    /// * `gillux` (sentences owned by gillux)
    /// * `gillux,ajip` (sentences owned by gillux or ajip)
    /// * `!gillux` (sentences orphan or owned by a different member than gillux)
    /// * `!gillux,ajip` (sentences orphan or owned by a member who is neither gillux nor ajip)
    pub owner: HashSet<String>,

    /// Limit to orphan sentences (if value is `yes`) or sentences owned by someone (if value is `no`).
    /// Make sure to combine with owner filter in a way that makes sense.
    ///
    /// **Allowed:** `yes` | `no`
    pub is_orphan: Option<bool>,

    /// Limit to [unapproved sentences](https://en.wiki.tatoeba.org/articles/show/faq#why-are-some-sentences-in-red?)
    /// (if value is `yes`) or exclude unapproved sentences (if value is `no`).
    ///
    /// **Allowed:** `yes` | `no`
    pub is_unapproved: Option<bool>,

    /// Limit to sentences having one or more audio recordings (if value is `yes`) or no audio
    /// recording (if value is `no`).
    ///
    /// **Allowed:** `yes` | `no`
    pub has_audio: Option<bool>,

    /// Limit to sentences having the provided tag. This parameter can be provided multiple times
    /// to search for sentences having multiple tags at the same time.
    ///
    /// **Pattern:** `!?[^,]+(,[^,]+)*`
    ///
    /// Examples:
    /// * `OK` (sentences tagged as `OK`)
    /// * `idiom` (sentences tagged as `idiom`)
    /// * `idiom,proverb` (sentences tagged as `idiom` or `proverb` (or both))
    /// * `!OK` (exclude sentences tagged as `OK`)
    /// * `!idiom,proverb` (exclude sentences tagged as `idiom` or `proverb` (or both))
    pub tag: HashSet<String>,

    /// Limit to sentences present on the provided list id. This parameter can be provided
    /// multiple times to search for sentences present on multiple lists at the same time.
    ///
    /// **Pattern:** `!?[0-9]+(,[0-9]+)*`
    ///
    /// # Examples:
    /// * `123` (sentences on list `123`)
    /// * `123,456` (sentences on list `123` or list `456` (or both))
    /// * `!123` (exclude sentences on list `123`)
    /// * `!123,456` (exclude sentences on list `123` or list `456` (or both))
    pub list: HashSet<String>,

    /// Limit to sentences owned by a self-identified native speaker (if value is `yes`) or a
    /// self-identified non-native speaker (if the value is `no`).
    /// This parameter can only be used when searching in a single language (not several).
    ///
    /// **Allowed:** `yes` | `no`
    pub is_native: Option<bool>,

    /// Limit according to sentence origin. All sentences fall in two sets: *unknown* and *known*.
    /// The set *known* is composed of two subsets: *original* + *translation*.
    ///
    /// **Allowed:** `original` | `translation` | `known` | `unknown`
    ///
    /// # Examples:
    /// * `original` (sentences not added as translations of other sentences)
    /// * `translation` (sentences added as translations of other sentences)
    /// * `known` (sentences we know have been added or not as translations of other sentences)
    /// * `unknown` (sentences we do not know whether or not they have been added as translations of other sentences)
    pub origin: Option<TatoebaOrigin>,

    /// Limit to sentences having translations in this language.
    ///
    /// **Pattern:** `!?[a-z]{3,4}(,[a-z]{3,4})*`
    ///
    /// # Examples:
    /// * `epo` (sentences having translation(s) in Esperanto)
    /// * `epo,sun` (sentences having translation(s) in Esperanto or Sundanese)
    /// * `!epo,sun` (sentences having translation(s) in a language that is not Esperanto or Sundanese)
    pub trans_lang: HashSet<String>,

    /// Limit to sentences having directly-linked translation(s) if value is `yes`,
    /// or indirectly-linked translations (i.e. translations of translations) if the value is `no`.
    ///
    /// **Allowed:** `yes` | `no`
    pub trans_is_direct: Option<bool>,

    /// Limit to sentences having translation(s) owned by the provided username.
    /// Make sure to combine with `trans:is_orphan` filter in a way that makes sense.
    ///
    /// **Pattern:** `!?[0-9a-zA-Z_]*(,[0-9a-zA-Z_]*)*`
    ///
    /// # Examples:
    /// * `gillux` (sentences having translation(s) owned by `gillux`)
    /// * `gillux,ajip` (sentences having translation(s) owned by `gillux` or `ajip`)
    /// * `!gillux` (sentences having translation(s) owned by a different member than `gillux` or `orphan`)
    /// * `!gillux,ajip` (sentences having translation(s) that are orphan or owned by a member who is neither `gillux` nor `ajip`)
    pub trans_owner: HashSet<String>,

    /// Limit to sentences having [unapproved](https://en.wiki.tatoeba.org/articles/show/faq#why-are-some-sentences-in-red?)
    /// translation(s) (if value is `yes`) or having translation(s) not marked as unapproved (if value is `no`).
    ///
    /// **Allowed:** `yes` | `no`
    pub trans_is_unapproved: Option<bool>,

    /// Limit to sentences having orphan translations (if value is `yes`) or translations owned by
    /// someone (if value is `no`). Make sure to combine with `trans:owner` filter in a way that
    /// makes sense.
    ///
    /// **Allowed:** `yes` | `no`
    pub trans_is_orphan: Option<bool>,

    /// Limit to sentences having translation(s) having one or more audio recordings (if value is
    /// `yes`) or no audio recording (if value is `no`).
    ///
    /// **Allowed:** `yes` | `no`
    pub trans_has_audio: Option<bool>,

    /// Limit according to the presence of translations. Zero (`0`) or non-zero (`!0`) are the
    /// only allowed values.
    ///
    /// **Pattern:** `!?0`
    ///
    /// # Examples:
    /// * `0` (sentences not having any translation)
    /// * `!0` (sentences having translation(s))
    pub trans_count: Option<bool>,

    /// Sort order of the sentences. Prefix the value with minus - to reverse that order.
    ///
    /// **Pattern:** `-?(relevance|words|created|modified|random)`
    ///
    /// # Examples:
    /// * `relevance` (prioritize sentences with exact matches, then sentences containing all the searched words, then shortest sentences)
    /// * `words` (order by number of words (or, if the language does not use spaces as word separators, by number of characters), shortest first)
    /// * `-words` (order by number of words, longest first)
    /// * `created` (order by sentence creation date (newest first))
    /// * `-created` (order by sentence creation date (oldest first))
    /// * `modified` (order by last sentence modification (last modified first))
    /// * `random` (randomly sort results)
    pub sort: Option<TatoebaSort>,

    /// Maximum number of sentences in the response.
    pub limit: Option<usize>,
    // By default, all the translations of matched sentences are returned,
    // regardless of how translations filters were used.
    // Here you can limit the language of the translations that will be displayed in the result,
    // using a comma-separated list of languages codes.
    // You may also use an empty value to not display any translation.
    //
    // # Examples:
    // * `epo` (only show translations in `Esperanto`, if any)
    // * `epo,sun` (only show translations in `Esperanto` and `Sundanese`, if any)
    //pub showtrans: HashSet<String>,
}

/// Utility that inserts a boolean filter into the query map.
///
/// * `key` – The name of the parameter.
/// * `value` – If `Some`, it is converted to `"yes"` or `"no"`.
///
/// This helper keeps the conversion logic in one place and makes the
/// main `From<&TatoebaSearch>` implementation easier to read.
fn insert_search_bool(
    out: &mut HashMap<&'static str, String>,
    key: &'static str,
    value: Option<bool>,
) {
    if let Some(value) = value {
        let value = if value { "yes" } else { "no" };
        out.insert(key, value.to_owned());
    }
}

/// Utility that inserts a comma‑separated list of strings into the query map.
///
/// It is used for all fields that are represented as `HashSet<String>` in
/// `TatoebaSearch`. Empty sets are ignored so that no unnecessary parameter
/// ends up in the URL.
fn insert_search_hashset(
    out: &mut HashMap<&'static str, String>,
    key: &'static str,
    value: &HashSet<String>,
) {
    if !value.is_empty() {
        let value = value
            .iter()
            .map(|lang| lang.as_str())
            .collect::<Vec<&str>>()
            .join(",");

        out.insert(key, value);
    }
}

impl From<&TatoebaSearch> for HashMap<&'static str, String> {
    /// `From<&TatoebaSearch>` implementation converts a search into query‑parameters.
    ///
    /// Each field is mapped to its string representation.  
    /// * The `showtrans` parameter currently uses `item.trans_lang`, which is
    ///   likely a mistake – the struct contains no `showtrans` field (it was
    ///   commented out). If you want to expose that option, add a dedicated
    ///   field and use it here.
    fn from(item: &TatoebaSearch) -> Self {
        let mut out = HashMap::new();

        insert_search_hashset(&mut out, "lang", &item.lang);

        // word_count is stored as (min, max).  The docs mention an
        // `!` prefix for exclusions, but the struct does not support it.
        match item.word_count {
            (Some(min), Some(max)) => {
                out.insert("word_count", format!("{min}-{max}"));
            }
            (Some(min), None) => {
                out.insert("word_count", format!("{min}-"));
            }
            (None, Some(max)) => {
                out.insert("word_count", format!("-{max}"));
            }
            _ => {}
        }

        insert_search_hashset(&mut out, "owner", &item.owner);
        insert_search_bool(&mut out, "is_orphan", item.is_orphan);
        insert_search_bool(&mut out, "is_unapproved", item.is_unapproved);
        insert_search_bool(&mut out, "has_audio", item.has_audio);
        insert_search_hashset(&mut out, "tag", &item.tag);
        insert_search_hashset(&mut out, "list", &item.list);
        insert_search_bool(&mut out, "is_native", item.is_native);

        if let Some(origin) = item.origin {
            out.insert("origin", origin.to_string());
        }

        insert_search_hashset(&mut out, "trans:lang", &item.trans_lang);
        insert_search_bool(&mut out, "trans:is_direct", item.trans_is_direct);
        insert_search_hashset(&mut out, "trans:owner", &item.trans_owner);
        insert_search_bool(&mut out, "trans:is_unapproved", item.trans_is_unapproved);
        insert_search_bool(&mut out, "trans:is_orphan", item.trans_is_orphan);
        insert_search_bool(&mut out, "trans:has_audio", item.trans_has_audio);

        if let Some(trans_count) = item.trans_count {
            let trans_count = if trans_count { "!0" } else { "0" };

            out.insert("trans:count", trans_count.to_owned());
        }

        if let Some(sort) = item.sort {
            out.insert("sort", sort.to_string());
        }

        if let Some(limit) = item.limit {
            out.insert("limit", limit.to_string());
        }

        insert_search_hashset(&mut out, "showtrans", &item.trans_lang);

        out
    }
}

impl TatoebaSearch {
    /// Builds a new search with the source and target languages pre‑filled.
    ///
    /// The returned `TatoebaSearch` has only two fields set:
    /// * `lang` – contains the source language.
    /// * `trans_lang` – contains the target language.
    ///   All other filters are left at their default values.
    pub fn new(from: &str, to: &str) -> Self {
        let mut lang = HashSet::new();
        lang.insert(from.to_owned());

        let mut trans_lang = HashSet::new();
        trans_lang.insert(to.to_owned());

        TatoebaSearch {
            lang,
            trans_lang,
            ..Default::default()
        }
    }

    /// Serialises the search into a URL string.
    ///
    /// `querry` is the free‑text query part of the request.  
    /// If `after` is supplied it will be appended as a key/value pair to enable
    /// keyset pagination.
    pub fn to_string(&self, querry: &str, after: Option<&str>) -> String {
        let mut params: HashMap<&'static str, String> = self.into();

        if let Some(after) = after {
            params.insert("after", after.to_owned());
        }

        let params = params
            .iter()
            .map(|(key, value)| format!("{key}={value}"))
            .collect::<Vec<String>>()
            .join("&");

        format!("https://api.tatoeba.org/unstable/sentences?q={querry}&{params}")
    }

    /// Executes the HTTP request for a single page of results.
    ///
    /// The method returns the parsed `TatoebaResponse`.
    /// Errors are propagated as boxed trait objects so that
    /// callers can decide how to handle them.
    pub fn search(
        &self,
        querry: &str,
        after: Option<&str>,
    ) -> Result<TatoebaResponse, Box<dyn std::error::Error>> {
        let url = self.to_string(querry, after);
        /* println!("\nTatoeba url: {}", &url); */

        let client = reqwest::blocking::Client::new();
        let response = client.request(Method::GET, &url).send()?;
        let response = response.text()?;
        let response: TatoebaResponse = serde_json::from_str(response.as_str())?;

        Ok(response)
    }

    /// Returns an iterator that lazily fetches pages of results.
    ///
    /// `query` is the free‑text query.  
    /// `delay` allows throttling between requests; it is applied *before* each
    /// page load (including the first one).
    pub fn search_iter<'a>(
        &'a self,
        query: &'a str,
        delay: Option<Duration>,
    ) -> TatoebaSearchIter<'a> {
        TatoebaSearchIter::from(self, query, delay)
    }
}

pub struct TatoebaSearchIter<'a> {
    search: &'a TatoebaSearch,
    querry: &'a str,
    response: Option<TatoebaResponse>,
    delay: Option<Duration>,
}

impl<'a> TatoebaSearchIter<'a> {
    /// Helper used by `search_iter` to initialise the iterator.
    ///
    /// The first request is performed immediately (after an optional sleep).
    /// Subsequent pages are fetched lazily inside the `Iterator` implementation.
    fn from(search: &'a TatoebaSearch, querry: &'a str, delay: Option<Duration>) -> Self {
        if let Some(delay) = delay {
            thread::sleep(delay);
        }

        TatoebaSearchIter {
            search,
            querry,
            response: search.search(querry, None).ok(),
            delay,
        }
    }
}

impl<'a> Iterator for TatoebaSearchIter<'a> {
    type Item = TatoebaEntry;

    /// Pulls the next `TatoebaEntry` from the iterator.
    ///
    /// The implementation works as follows:
    /// 1. If the current page is exhausted (`response.data.is_empty()`), it fetches
    ///    the next page using the cursor returned by the API.
    /// 2. Entries are returned one by one via `pop()`.  
    ///
    /// **Note:** `Vec::pop` removes from the back; if you want FIFO order,
    /// consider reversing the vector or using `remove(0)`
    fn next(&mut self) -> Option<Self::Item> {
        let response = self.response.as_mut()?;

        // End of all pages.
        if response.data.is_empty() {
            self.response = None;
            return None;
        }

        // see note above
        let out = response.data.pop();

        // If the current page is now empty, fetch the next one (if any).
        if response.data.is_empty() {
            self.response = if let Some(cursor_end) = &response.paging.cursor_end {
                if let Some(delay) = self.delay {
                    thread::sleep(delay);
                }

                self.search.search(self.querry, Some(cursor_end)).ok()
            } else {
                None
            }
        }

        out
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TatoebaResponse {
    pub paging: TatoebaPaging,
    pub data: Vec<TatoebaEntry>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TatoebaPaging {
    pub total: usize,
    pub has_next: bool,
    pub cursor_end: Option<String>,
    pub next: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TatoebaEntry {
    pub id: u32,
    pub text: String,
    pub lang: String,
    pub script: Option<String>,
    pub license: String,
    pub owner: String,
    pub transcriptions: Vec<TatoebaTranscription>,
    pub audios: Vec<TatoebaAudio>,
    pub translations: Vec<Vec<TatoebaTranslation>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[allow(non_snake_case)]
pub struct TatoebaTranscription {
    pub script: String,
    pub text: String,
    pub needsReview: bool,
    #[serde(rename = "type")]
    pub type_: String,
    pub html: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TatoebaAudio {
    pub author: String,
    pub attribution_url: String,
    pub license: String,
    pub download_url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TatoebaTranslation {
    pub id: u32,
    pub text: String,
    pub lang: String,
    pub script: Option<String>,
    pub license: String,
    pub owner: String,
    pub transcriptions: Vec<TatoebaTranscription>,
    pub audios: Vec<TatoebaAudio>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse() {
        let _: TatoebaResponse = serde_json::from_str(include_str!("./test_data.json")).unwrap();
    }
}
