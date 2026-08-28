use crate::persian::normalize_persian;

pub fn to_prefix_query(normalized: &str) -> Option<String> {
    let mut token = String::with_capacity(normalized.len());

    for ch in normalized.chars() {
        if matches!(ch, '"' | '*' | '(' | ')' | ':' | '^' | '{' | '}' | '[' | ']' | '+' | '-') {
            continue;
        }
        token.push(ch);
    }

    if token.is_empty() || is_reserved(&token) {
        return None;
    }

    token.push('*');
    Some(token)
}

pub fn to_match_query(raw: &str) -> Option<String> {
    let raw = raw.trim().replace('-', " ");
    if raw.is_empty() {
        return None;
    }

    let mut terms = Vec::new();
    for piece in raw.split_whitespace() {
        if let Some(term) = to_prefix_query(&normalize_persian(piece)) {
            if !terms.contains(&term) {
                terms.push(term);
            }
        }
    }

    let concatenated = to_prefix_query(&normalize_persian(&raw));

    match (terms.len(), concatenated) {
        (0, None) => None,
        (0, Some(token)) | (1, Some(token)) => Some(token),
        (1, None) => terms.pop(),
        (_, Some(concatenated)) => {
            let and_clause = terms.join(" AND ");
            if concatenated == and_clause {
                Some(and_clause)
            } else {
                Some(format!("({and_clause}) OR {concatenated}"))
            }
        }
        (_, None) => Some(terms.join(" AND ")),
    }
}

fn is_reserved(token: &str) -> bool {
    matches!(
        token.to_ascii_uppercase().as_str(),
        "AND" | "OR" | "NOT" | "NEAR"
    )
}

#[cfg(test)]
mod tests {
    use super::{to_match_query, to_prefix_query};

    #[test]
    fn rejects_empty_tokens() {
        assert_eq!(to_prefix_query(""), None);
        assert_eq!(to_prefix_query("***"), None);
    }

    #[test]
    fn appends_prefix_wildcard() {
        assert_eq!(to_prefix_query("کتاب").as_deref(), Some("کتاب*"));
    }

    #[test]
    fn builds_and_or_query_for_multiple_words() {
        let query = to_match_query("hello world").expect("query");
        assert!(query.contains("AND"), "{query}");
        assert!(query.contains("OR"), "{query}");
        assert!(query.contains("hello*"), "{query}");
        assert!(query.contains("world*"), "{query}");
        assert!(query.contains("helloworld*"), "{query}");
    }

    #[test]
    fn ignores_fts_operators() {
        assert_eq!(to_match_query("\"\" ** ((("), None);
        assert!(to_match_query("کتاب +").is_some());
        let hyphenated = to_match_query("row-49999").expect("hyphen query");
        assert!(!hyphenated.contains('-'), "{hyphenated}");
        assert!(hyphenated.contains("49999*"), "{hyphenated}");
    }

    #[test]
    fn compound_persian_stays_one_prefix_token() {
        assert_eq!(
            to_match_query("خانه‌من").as_deref(),
            Some("خانهمن*")
        );
    }
}
