use std::{
    collections::{HashMap, HashSet},
    fmt, thread,
    time::Duration,
};

use reqwest::Method;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Optional {
    Any,
    Yes,
    No,
}

impl fmt::Display for Optional {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Sort {
    ///Relevance
    Relevance,
    ///Fewest words first
    Words,
    ///Last created first
    Created,
    ///Last modified first
    Modified,
    ///Random
    Random,
}

impl fmt::Display for Sort {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Limit {
    /// Limit to
    Limit,
    Exclude,
}

impl fmt::Display for Limit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TatoebaSearch {
    pub from: String,
    pub has_audio: Optional,
    //pub list: Option<u32>,
    pub native: bool,
    pub original: bool,
    pub orphans: Optional,
    //pub query: String,
    pub sort: Sort,
    ///yes or no
    pub sort_reverse: bool,
    pub tags: HashSet<String>,
    pub to: String,
    pub trans_filter: Limit,
    pub trans_has_audio: Optional,
    pub trans_link: Option<String>,
    pub trans_orphan: Optional,
    pub trans_to: Option<String>,
    pub trans_unapproved: Optional,
    pub trans_user: Option<String>,
    pub unapproved: Optional,
    pub user: Option<String>,
    pub word_count_max: Option<u16>,
    pub word_count_min: Option<u16>,
}

impl Default for TatoebaSearch {
    fn default() -> Self {
        TatoebaSearch {
            from: String::new(),
            has_audio: Optional::Any,
            //list: None,
            native: false,
            original: false,
            orphans: Optional::Any,
            //query: String::new(),
            sort: Sort::Relevance,
            sort_reverse: false,
            tags: HashSet::new(),
            to: String::new(),
            trans_filter: Limit::Limit,
            trans_has_audio: Optional::Any,
            trans_link: None,
            trans_orphan: Optional::Any,
            trans_to: None,
            trans_unapproved: Optional::Any,
            trans_user: None,
            unapproved: Optional::Any,
            user: None,
            word_count_max: None,
            word_count_min: None,
        }
    }
}

impl From<&TatoebaSearch> for HashMap<&'static str, String> {
    fn from(item: &TatoebaSearch) -> Self {
        let mut out = HashMap::new();

        out.insert("from", item.from.clone());
        out.insert("has_audio", item.has_audio.to_string().to_lowercase());
        /* out.insert(
            "list",
            item.list
                .map_or(String::new(), |list| list.to_string().to_lowercase()),
        ); */
        out.insert("native", if item.native { "yes" } else { "no" }.to_owned());
        out.insert(
            "original",
            if item.original { "yes" } else { "no" }.to_owned(),
        );
        out.insert("orphans", item.orphans.to_string().to_lowercase());
        //out.insert("query", item.query.clone());
        out.insert("sort", item.sort.to_string().to_lowercase());
        out.insert(
            "sort_reverse",
            if item.sort_reverse { "yes" } else { "no" }.to_owned(),
        );
        out.insert(
            "tags",
            item.tags
                .iter()
                .cloned()
                .reduce(|a, b| a + " " + &b)
                .unwrap_or_default(),
        );
        out.insert("to", item.to.clone());
        out.insert("trans_filter", item.trans_filter.to_string().to_lowercase());
        out.insert(
            "trans_has_audio",
            item.trans_has_audio.to_string().to_lowercase(),
        );
        out.insert("trans_link", item.trans_link.clone().unwrap_or_default());
        out.insert("trans_orphan", item.trans_orphan.to_string().to_lowercase());
        out.insert("trans_to", item.trans_to.clone().unwrap_or_default());
        out.insert(
            "trans_unapproved",
            item.trans_unapproved.to_string().to_lowercase(),
        );
        out.insert("trans_user", item.trans_user.clone().unwrap_or_default());
        out.insert("unapproved", item.unapproved.to_string().to_lowercase());
        out.insert("user", item.user.clone().unwrap_or_default());
        out.insert(
            "word_count_max",
            item.word_count_max
                .map_or(String::new(), |count| count.to_string()),
        );
        out.insert(
            "word_count_min",
            item.word_count_min
                .map_or(String::new(), |count| count.to_string()),
        );

        out.retain(|_, value| !value.is_empty());

        out
    }
}

impl TatoebaSearch {
    pub fn new(from: &str, to: &str) -> Self {
        TatoebaSearch {
            from: from.to_owned(),
            to: to.to_owned(),
            ..Default::default()
        }
    }

    pub fn search(
        &self,
        query: &str,
        page: usize,
    ) -> Result<TatoebaResponse, Box<dyn std::error::Error>> {
        // https://en.wiki.tatoeba.org/articles/show/api#
        let params: HashMap<&'static str, String> = self.into();
        let params: String = params
            .into_iter()
            .map(|(k, v)| k.to_owned() + "=" + &v)
            .reduce(|a, b| a + "&" + b.as_str())
            .unwrap_or_default();

        let url =
            format!("https://dev.tatoeba.org/eng/api_v0/search?query={query}&page={page}&{params}");

        println!("\nTatoeba url: {}", &url);

        let client = reqwest::blocking::Client::new();
        let response = client.request(Method::GET, &url).send()?;
        let response = response.text()?;
        let response: TatoebaResponse = serde_json::from_str(response.as_str())?;

        Ok(response)
    }

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
    fn from(search: &'a TatoebaSearch, querry: &'a str, delay: Option<Duration>) -> Self {
        if let Some(delay) = delay {
            thread::sleep(delay);
        }

        TatoebaSearchIter {
            search,
            querry,
            response: search.search(querry, 1).ok(),
            delay,
        }
    }
}

impl<'a> Iterator for TatoebaSearchIter<'a> {
    type Item = TatoebaResult;

    fn next(&mut self) -> Option<Self::Item> {
        let response = self.response.as_mut()?;
        let out = response.results.pop();

        if response.results.is_empty() {
            let next_page = response.paging.Sentences.page + 1;

            self.response = if next_page <= response.paging.Sentences.pageCount {
                if let Some(delay) = self.delay {
                    thread::sleep(delay);
                }

                self.search.search(self.querry, next_page).ok()
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
    pub results: Vec<TatoebaResult>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[allow(non_snake_case)]
pub struct TatoebaPaging {
    pub Sentences: TatoebaPagingSentences,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[allow(non_snake_case)]
pub struct TatoebaPagingSentences {
    pub finder: String,
    pub page: usize,
    pub current: u32,
    pub count: u32,
    pub perPage: usize,
    pub start: u32,
    pub end: u32,
    pub prevPage: bool,
    pub nextPage: bool,
    pub pageCount: usize,
    pub sort: String,
    pub direction: bool,
    pub limit: Option<String>,
    pub sortDefault: bool,
    pub directionDefault: bool,
    pub scope: Option<String>,
    pub completeSort: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TatoebaResult {
    pub id: u32,
    pub text: String,
    pub lang: String,
    pub correctness: i32,
    pub script: Option<String>,
    pub license: String,
    pub translations: Vec<Vec<TatoebaResultTranslation>>,
    pub transcriptions: Vec<TatoebaResultTranscription>,
    pub audios: Vec<TatoebaResultAudio>,
    pub user: Option<TatoebaResultUser>,
    pub lang_name: String,
    pub dir: String,
    pub lang_tag: String,
    pub is_favorite: Option<bool>,
    pub is_owned_by_current_user: bool,
    pub permissions: Option<Vec<String>>,
    pub max_visible_translations: u32,
    pub current_user_review: Option<TatoebaResultUser>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[allow(non_snake_case)]
pub struct TatoebaResultTranslation {
    pub id: u32,
    pub text: String,
    pub lang: String,
    pub correctness: i32,
    pub script: Option<String>,
    pub transcriptions: Vec<TatoebaResultTranscription>,
    pub audios: Vec<TatoebaResultAudio>,
    pub isDirect: Option<bool>,
    pub lang_name: String,
    pub dir: String,
    pub lang_tag: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[allow(non_snake_case)]
pub struct TatoebaResultTranscription {
    pub id: u32,
    pub sentence_id: u32,
    pub script: Option<String>,
    pub text: String,
    pub user_id: Option<u32>,
    pub needsReview: bool,
    pub modified: String,
    pub user: Option<TatoebaResultUser>,
    pub readonly: bool,
    #[serde(rename = "type")]
    pub type_: String,
    pub html: String,
    pub markup: Option<String>,
    pub info_message: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TatoebaResultAudio {
    pub id: u32,
    pub external: Option<String>,
    pub sentence_id: Option<u32>,
    pub user: Option<TatoebaResultUser>,
    pub author: String,
    pub attribution_url: Option<String>,
    pub license: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TatoebaResultUser {
    pub username: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse() {
        let _: TatoebaResponse =
            serde_json::from_str(include_str!("./test_data_old.json")).unwrap();
    }
}
