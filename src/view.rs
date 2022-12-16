use crate::StatsVec;
use crate::{camelot_traktor, markdown, question_factory};
use anyhow::Result;
use chrono::{DateTime, NaiveDateTime, Utc};
use serde_json::to_string;
use serde_json::value::{from_value, to_value, Value};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tera::Result as TeraResult;
use tera::{Context, Function, Tera};
use tokio::fs;
use tokio::io::AsyncWriteExt;

const TEMPLATE_PATH: &str = "templates/**/*.html";
const STATIC_PATH: &str = "static";
const BUILD_PATH: &str = "build";
const HISTORY_PATH: &str = "history";
const INDEX_NAME: &str = "index.html";
const JSON_NAME: &str = "latest.json";
const TERA_FILTER_VALUE: &str = "value";
const FILTER_HUMAN_DT: &str = "human_datetime";
const PATTERN_HUMAN_DT: &str = "%F";
const FILTER_FIELD_DT: &str = "field_datetime";
const PATTERN_FIELD_DT: &str = "%+";
const PATTERN_HISTORY_NAME: &str = "%F.json";
const FILTER_WELCOME: &str = "welcome";
const WELCOME_MARKDOWN_PATH: &str = "WELCOME.md";
const VIDEO_PATTERN: &str = "<p>{VIDEO}</p>";
const FILTER_KEY_EXTRA_INFO: &str = "key_extra_info";
const FUNCTION_HUMAN_QUESTION: &str = "human_question";
const NO_TARGET_BLANK_PATTERN: &str = "a href=";
const TARGET_BLANK_PATTERN: &str = "a target=\"_blank\" href=";

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let mut tera = match Tera::new(TEMPLATE_PATH) {
            Ok(t) => t,
            Err(e) => {
                println!("Parsing error(s): {}", e);
                ::std::process::exit(1);
            }
        };
        tera.register_filter(FILTER_HUMAN_DT, human_datetime);
        tera.register_filter(FILTER_FIELD_DT, field_datetime);
        tera.register_filter(FILTER_WELCOME, render_welcome_markdown);
        tera.register_filter(FILTER_KEY_EXTRA_INFO, key_extra_info);
        tera.register_function(FUNCTION_HUMAN_QUESTION, make_human_question());
        tera
    };
}

// We could use Tera's `dt` filter for this
// But it throws a runtime error when formatting `%Z` / `%+` specifiers
fn format_datetime(filter_name: &str, value: &Value, pattern: &str) -> TeraResult<Value> {
    let timestamp = try_get_value!(filter_name, TERA_FILTER_VALUE, i64, value);
    // We know that this datetime is good because we generated it ourselves
    let datetime = DateTime::<Utc>::from_utc(
        NaiveDateTime::from_timestamp_opt(timestamp, 0).unwrap(),
        Utc,
    );
    let formatted = format!("{}", datetime.format(pattern));
    Ok(to_value(formatted).unwrap())
}

fn human_datetime(value: &Value, _: &HashMap<String, Value>) -> TeraResult<Value> {
    format_datetime(FILTER_HUMAN_DT, value, PATTERN_HUMAN_DT)
}

fn field_datetime(value: &Value, _: &HashMap<String, Value>) -> TeraResult<Value> {
    format_datetime(FILTER_FIELD_DT, value, PATTERN_FIELD_DT)
}

fn render_welcome_markdown(value: &Value, _: &HashMap<String, Value>) -> TeraResult<Value> {
    let youtube_html = try_get_value!(FILTER_WELCOME, TERA_FILTER_VALUE, String, value);
    let rendered = markdown::render(
        WELCOME_MARKDOWN_PATH,
        &[
            markdown::Replace {
                what: VIDEO_PATTERN,
                with: &youtube_html,
            },
            markdown::Replace {
                what: NO_TARGET_BLANK_PATTERN,
                with: TARGET_BLANK_PATTERN,
            },
        ],
    );
    Ok(to_value(rendered).unwrap())
}

fn key_extra_info(value: &Value, _: &HashMap<String, Value>) -> TeraResult<Value> {
    let key = try_get_value!(FILTER_KEY_EXTRA_INFO, TERA_FILTER_VALUE, String, value);
    let extra_info = camelot_traktor::get_key_extra_info(&key);
    Ok(to_value(extra_info).unwrap())
}

fn build_path(file_name: &str) -> PathBuf {
    Path::new(BUILD_PATH).join(file_name)
}

async fn save_file(content: String, path: impl AsRef<Path>) -> Result<()> {
    let mut file = fs::File::create(path).await?;
    file.write_all(content.as_bytes()).await?;

    Ok(())
}

async fn render_html(stats_vec: &StatsVec) -> Result<()> {
    let result = TEMPLATES.render(INDEX_NAME, &Context::from_serialize(stats_vec)?)?;
    save_file(result, build_path(INDEX_NAME)).await
}

async fn render_json(stats_vec: &StatsVec, record_history: bool) -> Result<()> {
    let result = to_string(stats_vec)?;
    let latest_path = build_path(JSON_NAME);
    save_file(result, &latest_path).await?;

    if record_history {
        let history_folder_path = build_path(HISTORY_PATH);
        fs::create_dir_all(&history_folder_path).await?;

        let history_path = history_folder_path.join(
            stats_vec
                .updated_at
                .format(PATTERN_HISTORY_NAME)
                .to_string(),
        );
        fs::copy(latest_path, history_path).await?;
    };
    Ok(())
}

/// Credits to https://nick.groenen.me/notes/recursively-copy-files-in-rust/
/// It is blocking because async version it not trivial: boxes, pins, lifetimes, etc.
/// Check https://doc.rust-lang.org/error_codes/E0733.html
fn copy_recursively_blocking(
    source: impl AsRef<Path>,
    destination: impl AsRef<Path>,
) -> Result<()> {
    std::fs::create_dir_all(&destination)?;
    for entry in std::fs::read_dir(source)? {
        let entry = entry?;
        let filetype = entry.file_type()?;
        if filetype.is_dir() {
            copy_recursively_blocking(entry.path(), destination.as_ref().join(entry.file_name()))?;
        } else {
            std::fs::copy(entry.path(), destination.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

pub async fn build(stats_vec: StatsVec) -> Result<()> {
    fs::create_dir_all(BUILD_PATH).await?;

    render_html(&stats_vec).await?;
    render_json(&stats_vec, true).await?;

    copy_recursively_blocking(STATIC_PATH, BUILD_PATH)?;

    Ok(())
}

fn make_human_question() -> impl Function {
    Box::new(move |args: &HashMap<String, Value>| -> TeraResult<Value> {
        match (args.get("entity"), args.get("genre")) {
            (Some(entity), Some(genre)) => match (
                from_value::<String>(entity.clone()),
                from_value::<String>(genre.clone()),
            ) {
                (Ok(entity), Ok(genre)) => {
                    Ok(to_value(question_factory::generate(&entity, &genre)).unwrap())
                }
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
    })
}
