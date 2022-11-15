use scraper::{Html, Selector};
use std::collections::HashSet;

const GENRE_SELECTOR: &str = ".genre-drop-list__genre";
const NAME_ATTR: &str = "data-name";
const URL_ATTR: &str = "href";
const IGNORED_GENRES: &[&str] = &["DJ Tools"];

lazy_static! {
    static ref IGNORED_GENRE_SET: HashSet<&'static str> = IGNORED_GENRES.iter().copied().collect();
}

pub struct Genre {
    pub name: String,
    pub url: String,
}

pub fn get_genres(html: &str) -> Vec<Genre> {
    let document = Html::parse_document(html);

    // I use .unwrap() because of the
    // https://github.com/causal-agent/scraper/issues/77
    let selector = Selector::parse(GENRE_SELECTOR).unwrap();

    let mut unique_names: HashSet<&str> = HashSet::new();
    let mut genres = vec![];

    for element in document.select(&selector) {
        let name = element.value().attr(NAME_ATTR);
        let url = element.value().attr(URL_ATTR);
        if let (Some(name), Some(url)) = (name, url) {
            {
                if IGNORED_GENRE_SET.contains(name) {
                    continue;
                }
                if unique_names.contains(name) {
                    continue;
                }
                unique_names.insert(name);
                genres.push(Genre {
                    name: name.to_string(),
                    url: url.to_string(),
                })
            }
        }
    }

    genres
}
