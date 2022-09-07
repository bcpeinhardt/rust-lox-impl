/// Strips the first and last character of a string.
pub fn strip_quotes(s: String) -> String {
    s.chars().skip(1).take(s.len() - 2).collect()
}

/// Digits 0-9
pub fn is_digit(c: char) -> bool {
    c >= '0' && c <= '9'
}

/// Lowercase and uppercase English letters a-z, A-Z, and underscores.
pub fn is_alpha(l: char) -> bool {
    (l >= 'a' && l <= 'z') || (l >= 'A' && l <= 'Z') || l == '_'
}

/// Lox's definition of a valid alphanumeric sequence. Digits 0-9, lowercase and uppercase english letters a-z, A-Z, and underscores.
pub fn is_alpha_numeric(c: char) -> bool {
    is_alpha(c) || is_digit(c)
}
