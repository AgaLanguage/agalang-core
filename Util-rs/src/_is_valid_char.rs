pub fn is_valid_char(valid_chars: &str, eval_char: char) -> bool {
    for c in valid_chars.chars() {
        if c == eval_char {
            return true;
        }
    }
    false
}