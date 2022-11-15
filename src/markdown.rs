use comrak::{markdown_to_html, ComrakOptions};
use std::fs;
use std::path::Path;

pub struct Replace<'a> {
    pub what: &'a str,
    pub with: &'a str,
}

pub fn render(markdown_path: impl AsRef<Path>, replace_items: &[Replace]) -> String {
    // We will output a blank if the file is not found
    let markdown_input = fs::read_to_string(markdown_path).unwrap_or_default();

    let mut markdown_output = markdown_to_html(&markdown_input, &ComrakOptions::default());

    for item in replace_items {
        // Yes, it's pretty clunky, but it fits our purpose
        markdown_output = markdown_output.replace(item.what, item.with)
    }

    markdown_output
}
