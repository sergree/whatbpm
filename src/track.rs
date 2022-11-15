use crate::Key;
use crate::Meta;
use crate::TopPage;
use anyhow::{anyhow, Result};
use chrono::Duration;
use serde_json::Value;
use std::str::FromStr;

const START_PHRASE: &str = "window.Playables = ";
const END_PHRASE: &str = "window.Sliders";

#[derive(Debug)]
pub struct Track {
    pub bpm: u16,
    pub key: Key,
    pub label: String,
    pub duration: Duration,
    pub specific_genre: Option<String>,
}

struct Context {
    fill_genre: bool,
}

impl TryFrom<(&Value, Context)> for Track {
    type Error = anyhow::Error;

    fn try_from(value_with_ctx: (&Value, Context)) -> Result<Self> {
        let (value, ctx) = value_with_ctx;
        let object = value.as_object().ok_or_else(|| anyhow!("Missing object"))?;

        let bpm = object
            .get("bpm")
            .ok_or_else(|| anyhow!("Missing bpm"))?
            .as_u64()
            .ok_or_else(|| anyhow!("Invalid bpm"))? as u16;

        let key = object
            .get("key")
            .ok_or_else(|| anyhow!("Missing key"))?
            .as_str()
            .map(Key::from_str)
            .ok_or_else(|| anyhow!("Invalid key"))??;

        let label = object
            .get("label")
            .ok_or_else(|| anyhow!("Missing label"))?
            .get("name")
            .ok_or_else(|| anyhow!("Missing label name"))?
            .as_str()
            .ok_or_else(|| anyhow!("Invalid label name"))?
            .to_string();

        let duration_ms = object
            .get("duration")
            .ok_or_else(|| anyhow!("Missing duration"))?
            .get("milliseconds")
            .ok_or_else(|| anyhow!("Missing duration milliseconds"))?
            .as_i64()
            .ok_or_else(|| anyhow!("Invalid duration milliseconds"))?;

        let duration = Duration::milliseconds(duration_ms);

        let specific_genre = match ctx.fill_genre {
            true => Some(
                object
                    .get("genres")
                    .ok_or_else(|| anyhow!("Missing genres"))?
                    .as_array()
                    .ok_or_else(|| anyhow!("Invalid genres"))?
                    .iter()
                    .next()
                    .ok_or_else(|| anyhow!("Empty genres"))?
                    .get("name")
                    .ok_or_else(|| anyhow!("Missing genre name"))?
                    .as_str()
                    .ok_or_else(|| anyhow!("Invalid genre name"))?
                    .to_string(),
            ),
            false => None,
        };

        Ok(Track {
            bpm,
            key,
            label,
            duration,
            specific_genre,
        })
    }
}

pub struct TopTrackVec {
    pub meta: Meta,
    pub tracks: Vec<Track>,
}

impl TryFrom<TopPage> for TopTrackVec {
    type Error = anyhow::Error;

    fn try_from(top_page: TopPage) -> Result<Self> {
        let start_pos = top_page
            .text
            .find(START_PHRASE)
            .ok_or_else(|| anyhow!("Missing START_PHRASE"))?
            + START_PHRASE.len();
        let end_pos = top_page
            .text
            .find(END_PHRASE)
            .ok_or_else(|| anyhow!("Missing END_PHRASE"))?;
        let raw_json = top_page.text[start_pos..end_pos]
            .trim()
            .trim_end_matches(';');
        let json: Value = serde_json::from_str(raw_json)?;
        let fill_genre = top_page.meta.common_genre.is_none();
        Ok(TopTrackVec {
            meta: top_page.meta,
            tracks: json
                .get("tracks")
                .ok_or_else(|| anyhow!("Missing tracks"))?
                .as_array()
                .ok_or_else(|| anyhow!("Invalid tracks"))?
                .iter()
                .filter_map(|v| Track::try_from((v, Context { fill_genre })).ok())
                .collect(),
        })
    }
}
