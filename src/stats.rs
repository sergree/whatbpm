use crate::Key;
use crate::RootNote;
use crate::TopTrackVec;
use crate::Track;
use chrono::prelude::*;
use chrono::Duration;
use serde::Serialize;
use serde_with::{serde_as, DurationSeconds, TimestampSeconds};
use std::cmp::Reverse;
use std::collections::{BTreeMap, HashMap};

const LOWEST_BPM: u16 = 91;
const HIGHEST_BPM: u16 = LOWEST_BPM * 2;

#[serde_as]
#[derive(Serialize, Debug)]
pub struct Meta {
    pub common_genre: Option<String>,
    #[serde_as(as = "TimestampSeconds<i64>")]
    pub updated_at: DateTime<Utc>,
    pub based_on: String,
}

impl Meta {
    pub fn new(common_genre: Option<String>, based_on: String) -> Self {
        Self {
            common_genre,
            updated_at: Utc::now(),
            based_on,
        }
    }
}

#[derive(Serialize, Clone, Copy, Default, Debug)]
pub struct AverageBarCount {
    pub count: u16,
    pub bpm: u16,
}

impl AverageBarCount {
    fn new(average_duration: Duration, bpm: &[StandardItem<u16>]) -> Self {
        let seconds = average_duration.num_seconds() as f32;

        // We know it's not empty
        let first_top_bpm = bpm.iter().next().unwrap().value;

        let count_precise = (seconds * (first_top_bpm as f32)) / 240_f32; // Divide by (60 seconds * 4 beats)
        let count = (count_precise).round() as u16;

        Self {
            count,
            bpm: first_top_bpm,
        }
    }
}

#[derive(Serialize, Debug)]
pub struct StandardItem<T: Serialize> {
    pub value: T,
    pub count: usize,
}

#[serde_as]
#[derive(Serialize, Debug)]
pub struct Standard {
    pub bpm: Vec<StandardItem<u16>>,
    pub root_note: Vec<StandardItem<RootNote>>,
    pub key: Vec<StandardItem<Key>>,
    pub label: Vec<StandardItem<String>>,
    pub genre: Option<Vec<StandardItem<String>>>,
    #[serde_as(as = "DurationSeconds<i64>")]
    pub average_duration: Duration,
    pub average_bar_count: AverageBarCount,
}

impl From<Vec<Track>> for Standard {
    fn from(tracks: Vec<Track>) -> Self {
        // We rely on the fact that there are not 0 tracks, their absence is very unlikely
        // So we can safely .unwrap() here
        let make_genre_stats = tracks.first().unwrap().specific_genre.is_some();

        let mut bpm_hm: HashMap<u16, usize> = HashMap::new();
        let mut root_note_hm: HashMap<RootNote, usize> = HashMap::new();
        let mut key_hm: HashMap<Key, usize> = HashMap::new();
        let mut label_hm: HashMap<String, usize> = HashMap::new();
        let mut genre_hm: Option<HashMap<String, usize>> = match make_genre_stats {
            true => Some(HashMap::new()),
            false => None,
        };
        let mut average_duration: Duration = Duration::milliseconds(0);
        let tracks_len = tracks.len();

        for track in tracks {
            let bpm = bpm_hm.entry(track.bpm).or_insert(0);
            *bpm += 1;
            let root_note = root_note_hm.entry(track.key.root_note.clone()).or_insert(0);
            *root_note += 1;
            let key = key_hm.entry(track.key).or_insert(0);
            *key += 1;
            let label = label_hm.entry(track.label).or_insert(0);
            *label += 1;
            if let Some(genre_hm) = genre_hm.as_mut() {
                // If the first track has a special genre, then all the others have too
                // So we can safely .unwrap() here
                let genre = genre_hm.entry(track.specific_genre.unwrap()).or_insert(0);
                *genre += 1;
            }
            average_duration = average_duration + track.duration;
        }

        let mut bpm_hm_fixed: HashMap<u16, usize> = HashMap::new();
        for (bpm_initial, count) in bpm_hm {
            let bpm_fixed = if bpm_initial < LOWEST_BPM {
                bpm_initial * 2
            } else if bpm_initial >= HIGHEST_BPM {
                bpm_initial / 2
            } else {
                bpm_initial
            };

            let bpm = bpm_hm_fixed.entry(bpm_fixed).or_insert(0);
            *bpm += count;
        }

        let bpm: Vec<StandardItem<_>> = bpm_hm_fixed
            .into_iter()
            .map(|(value, count)| StandardItem { value, count })
            .collect();
        let root_note: Vec<StandardItem<_>> = root_note_hm
            .into_iter()
            .map(|(value, count)| StandardItem { value, count })
            .collect();
        let key: Vec<StandardItem<_>> = key_hm
            .into_iter()
            .map(|(value, count)| StandardItem { value, count })
            .collect();
        let label: Vec<StandardItem<_>> = label_hm
            .into_iter()
            .map(|(value, count)| StandardItem { value, count })
            .collect();
        let genre: Option<Vec<StandardItem<_>>> = genre_hm.map(|hm| {
            hm.into_iter()
                .map(|(value, count)| StandardItem { value, count })
                .collect()
        });
        average_duration = average_duration / tracks_len as i32;

        let mut standard = Self {
            bpm,
            root_note,
            key,
            label,
            genre,
            average_duration,
            average_bar_count: AverageBarCount::default(), // We can't know it until we know the top BPM
        };

        standard.sort();

        standard.average_bar_count = AverageBarCount::new(standard.average_duration, &standard.bpm);

        standard
    }
}

impl Standard {
    fn sort(&mut self) {
        self.bpm.sort_unstable_by_key(|k| k.value);
        self.bpm.sort_by_key(|k| Reverse(k.count));

        self.root_note
            .sort_unstable_by(|a, b| a.value.cmp(&b.value));
        self.root_note.sort_by_key(|k| Reverse(k.count));

        self.key.sort_unstable_by(|a, b| a.value.cmp(&b.value));
        self.key.sort_by_key(|k| Reverse(k.count));

        self.label
            .sort_unstable_by(|a, b| a.value.to_lowercase().cmp(&b.value.to_lowercase()));
        self.label.sort_by_key(|k| Reverse(k.count));

        if let Some(genre) = &mut self.genre {
            genre.sort_unstable_by(|a, b| a.value.to_lowercase().cmp(&b.value.to_lowercase()));
            genre.sort_by_key(|k| Reverse(k.count));
        }
    }
}

#[derive(Serialize, Debug)]
pub struct FoldedItem<T: Serialize> {
    pub count: usize,
    pub values: Vec<T>,
}

#[serde_as]
#[derive(Serialize, Debug)]
pub struct Folded {
    pub bpm: Vec<FoldedItem<u16>>,
    pub root_note: Vec<FoldedItem<RootNote>>,
    pub key: Vec<FoldedItem<Key>>,
    pub label: Vec<FoldedItem<String>>,
    pub genre: Option<Vec<FoldedItem<String>>>,
    #[serde_as(as = "DurationSeconds<i64>")]
    pub average_duration: Duration,
    pub average_bar_count: AverageBarCount,
}

impl From<&Standard> for Folded {
    fn from(standard: &Standard) -> Self {
        let mut bpm_btm: BTreeMap<usize, Vec<u16>> = BTreeMap::new();
        let mut root_note_btm: BTreeMap<usize, Vec<RootNote>> = BTreeMap::new();
        let mut key_btm: BTreeMap<usize, Vec<Key>> = BTreeMap::new();
        let mut label_btm: BTreeMap<usize, Vec<String>> = BTreeMap::new();
        let mut genre_btm: Option<BTreeMap<usize, Vec<String>>> =
            standard.genre.as_ref().map(|_| BTreeMap::new());

        for item in &standard.bpm {
            let bpm_vec = bpm_btm.entry(item.count).or_default();
            bpm_vec.push(item.value);
        }

        for item in &standard.root_note {
            let root_note_vec = root_note_btm.entry(item.count).or_default();
            root_note_vec.push(item.value.clone());
        }

        for item in &standard.key {
            let key_vec = key_btm.entry(item.count).or_default();
            key_vec.push(item.value.clone());
        }

        for item in &standard.label {
            let label_vec = label_btm.entry(item.count).or_default();
            label_vec.push(item.value.to_string());
        }

        if let Some(genre_btm) = genre_btm.as_mut() {
            // We have already checked the existence of the standard.genre above
            // So we can safely .unwrap() here
            for item in standard.genre.as_ref().unwrap() {
                let genre_vec = genre_btm.entry(item.count).or_default();
                genre_vec.push(item.value.to_string());
            }
        }

        let bpm: Vec<FoldedItem<u16>> = bpm_btm
            .into_iter()
            .rev()
            .map(|(count, values)| FoldedItem { count, values })
            .collect();
        let root_note: Vec<FoldedItem<RootNote>> = root_note_btm
            .into_iter()
            .rev()
            .map(|(count, values)| FoldedItem { count, values })
            .collect();
        let key: Vec<FoldedItem<Key>> = key_btm
            .into_iter()
            .rev()
            .map(|(count, values)| FoldedItem { count, values })
            .collect();
        let label: Vec<FoldedItem<String>> = label_btm
            .into_iter()
            .rev()
            .map(|(count, values)| FoldedItem { count, values })
            .collect();
        let genre: Option<Vec<FoldedItem<String>>> = genre_btm.map(|btm| {
            btm.into_iter()
                .rev()
                .map(|(count, values)| FoldedItem { count, values })
                .collect()
        });
        let average_duration = standard.average_duration;
        let average_bar_count = standard.average_bar_count;

        Self {
            bpm,
            root_note,
            key,
            label,
            genre,
            average_duration,
            average_bar_count,
        }
    }
}

#[derive(Serialize, Debug)]
pub struct Stats {
    pub meta: Meta,
    pub standard: Standard,
    pub folded: Folded,
}

impl From<TopTrackVec> for Stats {
    fn from(top_track_vec: TopTrackVec) -> Self {
        let TopTrackVec { meta, tracks } = top_track_vec;
        let standard = Standard::from(tracks);
        let folded = Folded::from(&standard);
        Self {
            meta,
            standard,
            folded,
        }
    }
}

#[serde_as]
#[derive(Serialize, Debug)]
pub struct StatsVec {
    #[serde_as(as = "TimestampSeconds<i64>")]
    pub updated_at: DateTime<Utc>,
    pub stats: Vec<Stats>,
}

impl From<Vec<TopTrackVec>> for StatsVec {
    fn from(top_track_vecs: Vec<TopTrackVec>) -> Self {
        Self {
            // We already know that this vec is not empty
            // So we can safely .unwrap() here
            updated_at: top_track_vecs
                .iter()
                .map(|s| s.meta.updated_at)
                .max()
                .unwrap_or_default(),
            stats: top_track_vecs.into_iter().map(Stats::from).collect(),
        }
    }
}
