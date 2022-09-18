use std::collections::HashMap;

use crate::token::TokenType;

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

/// Returns a Hashmap of all valid Lox keywords and their respective TokenType variants.
/// TODO: Consider replacing this with a FromStr implementation for TokenType.
pub fn keywords() -> HashMap<String, TokenType> {
    let mut map = HashMap::new();
    map.insert("and".to_owned(), TokenType::And);
    map.insert("class".to_owned(), TokenType::Class);
    map.insert("else".to_owned(), TokenType::Else);
    map.insert("false".to_owned(), TokenType::False);
    map.insert("for".to_owned(), TokenType::For);
    map.insert("fun".to_owned(), TokenType::Fun);
    map.insert("if".to_owned(), TokenType::If);
    map.insert("nil".to_owned(), TokenType::Nil);
    map.insert("or".to_owned(), TokenType::Or);
    map.insert("return".to_owned(), TokenType::Return);
    map.insert("super".to_owned(), TokenType::Super);
    map.insert("this".to_owned(), TokenType::This);
    map.insert("true".to_owned(), TokenType::True);
    map.insert("var".to_owned(), TokenType::Var);
    map.insert("while".to_owned(), TokenType::While);

    map
}
