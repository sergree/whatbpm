use crate::Genre;
use crate::Meta;
use anyhow::Result;

const BEATPORT_URL: &str = "https://www.beatport.com";
const MAIN_PATH: &str = "";
const TOP_PATH: &str = "/top-100";

async fn get(path: &str, add_top_path: bool) -> Result<(String, String)> {
    let url = match add_top_path {
        true => format!("{BEATPORT_URL}{path}{TOP_PATH}"),
        false => format!("{BEATPORT_URL}{path}"),
    };

    println!("Getting  {} ...", &url);
    let text = reqwest::get(&url).await?.text().await?;
    println!("Complete {}", &url);

    Ok((url, text))
}

pub struct TopPage {
    pub meta: Meta,
    pub text: String,
}

pub async fn get_main() -> Result<String> {
    get(MAIN_PATH, false).await.map(|r| r.1)
}

async fn get_top(genre: Option<Genre>) -> Result<TopPage> {
    let path = genre.as_ref().map_or(MAIN_PATH, |g| &g.url);
    let (url, text) = get(path, true).await?;

    Ok(TopPage {
        meta: Meta::new(genre.map(|g| g.name), url),
        text,
    })
}

pub async fn get_general_top() -> Result<TopPage> {
    get_top(None).await
}

pub async fn get_genre_top(genre: Genre) -> Result<TopPage> {
    get_top(Some(genre)).await
}
