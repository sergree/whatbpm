use std::collections::HashMap;

lazy_static! {
    static ref GUIDE: HashMap<&'static str, &'static str> = {
        // It's easier to just list them manually in this case
        HashMap::from([
            ("A min", "8A | 1m | relative to C maj"),
            ("Bb min", "3A | 8m | relative to Db maj"),
            ("B min", "10A | 3m | relative to D maj"),
            ("C min", "5A | 10m | relative to Eb maj"),
            ("C# min", "12A | 5m | relative to E maj"),
            ("D min", "7A | 12m | relative to F maj"),
            ("Eb min", "2A | 7m | relative to F# maj"),
            ("E min", "9A | 2m | relative to G maj"),
            ("F min", "4A | 9m | relative to Ab maj"),
            ("F# min", "11A | 4m | relative to A maj"),
            ("G min", "6A | 11m | relative to Bb maj"),
            ("G# min", "1A | 6m | relative to B maj"),
            ("A maj", "11B | 4d | relative to F# min"),
            ("Bb maj", "6B | 11d | relative to G min"),
            ("B maj", "1B | 6d | relative to G# min"),
            ("C maj", "8B | 1d | relative to A min"),
            ("Db maj", "3B | 8d | relative to Bb min"),
            ("D maj", "10B | 3d | relative to B min"),
            ("Eb maj", "5B | 10d | relative to C min"),
            ("E maj", "12B | 5d | relative to C# min"),
            ("F maj", "7B | 12d | relative to D min"),
            ("F# maj", "2B | 7d | relative to Eb min"),
            ("G maj", "9B | 2d | relative to E min"),
            ("Ab maj", "4B | 9d | relative to F min"),
        ])
    };
}

pub fn get_key_extra_info(key: &str) -> &'static str {
    // We can safely .unwrap() here, because we defined these literals in key.rs
    GUIDE.get(key).unwrap()
}
