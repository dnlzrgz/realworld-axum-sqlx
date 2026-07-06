/// Convert a title string to a slug for identifying
/// an article.
pub fn slugify(string: &str) -> String {
    const QUOTE_CHARS: &[char] = &['\'', '"'];

    string
        .split(|c: char| !(QUOTE_CHARS.contains(&c) || c.is_alphanumeric()))
        .map(|s| {
            let mut s = s.replace(QUOTE_CHARS, "");
            s = s.to_lowercase();
            s
        })
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

#[test]
fn test_slugify() {
    assert_eq!(
        slugify("Segfaults and You: When Raw Pointers Go Wrong"),
        "segfaults-and-you-when-raw-pointers-go-wrong"
    );

    assert_eq!(
        slugify("Why are DB Admins Always Shouting?"),
        "why-are-db-admins-always-shouting"
    );

    assert_eq!(
        slugify("Converting to Rust from C: It's as Easy as 1, 2, 3!"),
        "converting-to-rust-from-c-its-as-easy-as-1-2-3"
    )
}
