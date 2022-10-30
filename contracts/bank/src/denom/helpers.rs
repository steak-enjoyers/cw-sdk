/// Return whether the string contains only alphanumeric characters.
/// Note that our definition of "alphanumeric" means within the following charset: 0-9|a-z|A-Z,
/// which is narrower than Unicode's definition, which Rust uses.
pub fn is_alphanumeric(s: &str) -> bool {
    s.chars().all(|ch| matches!(ch, '0'..='9' | 'a'..='z' | 'A'..='Z'))
}

/// Return whether the string starts with a number 0-9.
pub fn starts_with_number(s: &str) -> bool {
    s.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false)
}
