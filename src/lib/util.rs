/// Strips the first and last character of a string.
pub fn strip_quotes(s: String) -> String { 
    s.chars().skip(1).take(s.len() - 2).collect()
}