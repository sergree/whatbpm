#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate tera;

mod camelot_traktor;
mod genre;
mod key;
mod markdown;
mod question_factory;
mod stats;
mod track;
mod view;
mod web;

use crate::genre::Genre;
use crate::key::{Key, RootNote};
use crate::stats::{Meta, StatsVec};
use crate::track::{TopTrackVec, Track};
use crate::web::TopPage;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let top_pages = load_pages().await?;

    let top_track_vecs = top_pages
        .into_iter()
        .map(TopTrackVec::try_from)
        .collect::<Result<Vec<_>>>()?;

    let stats_vec = StatsVec::from(top_track_vecs);

    view::build(stats_vec).await?;

    Ok(())
}

async fn load_pages() -> Result<Vec<TopPage>> {
    let main_page_handle = tokio::spawn(web::get_main());
    let general_top_page_handle = tokio::spawn(web::get_general_top());

    let main_page = main_page_handle.await??;
    let general_top_page = general_top_page_handle.await??;

    let genres = genre::get_genres(&main_page);
    let genre_top_page_handles: Vec<_> = genres
        .into_iter()
        .map(|g| tokio::spawn(async move { web::get_genre_top(g).await }))
        .collect();

    let mut top_pages = Vec::with_capacity(genre_top_page_handles.len() + 1);
    top_pages.push(general_top_page);
    for handle in genre_top_page_handles {
        top_pages.push(handle.await??);
    }

    Ok(top_pages)
}
