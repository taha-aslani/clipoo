//! Persian text normalization for search indexing and duplicate detection.
//!
//! The original clipboard text is never mutated. Callers must keep the source
//! string and store this output only in `normalized_content` / FTS.

pub fn normalize_persian(input: &str) -> String {
    let mut output = String::with_capacity(input.len());

    for ch in input.chars() {
        match ch {
            '\u{064A}' => output.push('\u{06CC}'),
            '\u{0643}' => output.push('\u{06A9}'),
            '\u{0624}' => output.push('\u{0648}'),
            '\u{0623}' | '\u{0625}' | '\u{0622}' => output.push('\u{0627}'),
            '\u{0621}' | '\u{0626}' => {}
            '\u{0640}' | '\u{200B}' | '\u{200C}' | '\u{200D}' | '\u{FEFF}' => {}
            '\u{064B}'..='\u{065F}' | '\u{0670}' => {}
            '\u{0610}'..='\u{061A}' | '\u{06D6}'..='\u{06ED}' => {}
            ch if ch.is_whitespace() => {}
            ch => output.push(ch),
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::normalize_persian;

    #[test]
    fn maps_arabic_yeh_to_persian_yeh() {
        assert_eq!(normalize_persian("مي"), normalize_persian("می"));
        assert_eq!(normalize_persian("می"), "می");
    }

    #[test]
    fn maps_arabic_kaf_to_persian_kaf() {
        assert_eq!(normalize_persian("كتاب"), normalize_persian("کتاب"));
        assert_eq!(normalize_persian("کتاب"), "کتاب");
    }

    #[test]
    fn folds_hamza_so_masool_matches() {
        assert_eq!(normalize_persian("مسئول"), normalize_persian("مسول"));
        assert_eq!(normalize_persian("مسول"), "مسول");
    }

    #[test]
    fn removes_zwnj_and_spaces_for_compound_words() {
        assert_eq!(
            normalize_persian("خانه من"),
            normalize_persian("خانه‌من")
        );
    }

    #[test]
    fn leaves_the_original_string_untouched() {
        let original = "كتاب می‌شود";
        let snapshot = original.to_string();
        let _ = normalize_persian(original);
        assert_eq!(original, snapshot);
        assert_ne!(normalize_persian(original), original);
    }
}
