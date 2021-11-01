use std::str::FromStr;

pub fn string_at(parts: &[&str], index: usize) -> String {
    parts
        .get(index)
        .map(|p| String::from_str(p).unwrap_or_default())
        .unwrap_or_default()
}
