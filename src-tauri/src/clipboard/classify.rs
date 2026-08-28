use crate::db::ClipboardItemType;

const CODE_KEYWORDS: &[&str] = &[
    "function ",
    "fn ",
    "func ",
    "def ",
    "class ",
    "import ",
    "from ",
    "const ",
    "let ",
    "var ",
    "return ",
    "public ",
    "private ",
    "package ",
    "impl ",
    "struct ",
    "enum ",
    "interface ",
    "export ",
    "require(",
    "#include",
    "<?php",
    "console.log",
    "=>",
    ":=",
];

pub fn classify_text(text: &str) -> ClipboardItemType {
    if is_url(text) {
        ClipboardItemType::Url
    } else if looks_like_code(text) {
        ClipboardItemType::Code
    } else {
        ClipboardItemType::Text
    }
}

fn is_url(text: &str) -> bool {
    let trimmed = text.trim();
    if trimmed.is_empty() || trimmed.chars().any(char::is_whitespace) {
        return false;
    }

    let lower = trimmed.to_ascii_lowercase();
    lower.starts_with("http://")
        || lower.starts_with("https://")
        || lower.starts_with("ftp://")
        || lower.starts_with("ftps://")
        || lower.starts_with("file://")
        || lower.starts_with("mailto:")
        || (lower.starts_with("www.") && lower.contains('.'))
}

fn looks_like_code(text: &str) -> bool {
    let trimmed = text.trim();
    if trimmed.len() < 8 {
        return false;
    }

    let keyword_hits = CODE_KEYWORDS
        .iter()
        .filter(|keyword| trimmed.contains(*keyword))
        .count();
    let has_braces = trimmed.contains('{') && trimmed.contains('}');
    let semicolon_count = trimmed.matches(';').count();
    let line_count = trimmed.lines().count();
    let indented_lines = trimmed
        .lines()
        .filter(|line| line.starts_with("    ") || line.starts_with('\t'))
        .count();

    keyword_hits >= 2
        || (keyword_hits >= 1 && (has_braces || semicolon_count >= 1 || indented_lines >= 1))
        || (has_braces && semicolon_count >= 2 && line_count >= 2)
}

#[cfg(test)]
mod tests {
    use super::classify_text;
    use crate::db::ClipboardItemType;

    #[test]
    fn detects_urls() {
        assert_eq!(
            classify_text("https://example.com/path"),
            ClipboardItemType::Url
        );
        assert_eq!(classify_text("www.example.com"), ClipboardItemType::Url);
    }

    #[test]
    fn detects_code() {
        let snippet = "fn main() {\n    println!(\"hi\");\n}";
        assert_eq!(classify_text(snippet), ClipboardItemType::Code);
    }

    #[test]
    fn keeps_persian_sentences_as_text() {
        assert_eq!(
            classify_text("این یک جمله معمولی فارسی است."),
            ClipboardItemType::Text
        );
    }

    #[test]
    fn does_not_treat_sentences_with_www_inside_as_url() {
        assert_eq!(
            classify_text("see www.example.com for details"),
            ClipboardItemType::Text
        );
    }
}
