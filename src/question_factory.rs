pub fn generate(entity: &str, genre: &str) -> String {
    let genre = if genre == "General" { "EDM" } else { genre };
    match entity {
        "BPM" => format!("What is the most commonly used BPM for {genre} at the moment?"),
        "Root Note" => {
            format!("What is the most commonly used root note for {genre} at the moment?")
        }
        "Key" => format!("What is the most commonly used key for {genre} at the moment?"),
        "Label" => {
            format!("What label has released most of the trending {genre} tracks lately?")
        }
        "Genre" => "What EDM genre is the most charting at the moment?".to_string(),
        "Average Duration" => {
            format!("What is the average {genre} track length at the moment?")
        }
        _ => unreachable!(),
    }
}
