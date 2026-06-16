/// Thin wrapper around attribute pairs providing `.get()` lookup without allocation.
///
/// Wraps `&[(&str, Option<&str>)]` and delegates `.iter()` to the slice.
/// Use `.get("key")` for attribute lookups — it returns `Option<&str>`.
/// An attribute without a value (e.g., `[flag]`) or a missing attribute both return `None`.
///
/// Example:
/// ```rust
/// use shortcode_parser::shortcode::Shortcode;
///
/// let mut sc = Shortcode::new();
/// sc.add("greet", |_, attrs| {
///     let name = attrs.get("name").unwrap_or("world");
///     format!("Hello, {name}")
/// });
///
/// assert_eq!(sc.render("[greet name=\"Rust\"]"), "Hello, Rust");
/// ```
#[derive(Debug, PartialEq, Eq)]
pub struct ShortcodeAttrs<'a>(&'a [(&'a str, Option<&'a str>)]);

impl<'a> ShortcodeAttrs<'a> {
    /// Looks up an attribute by name. Returns `Some(&str)` if present with a value, `None` otherwise.
    pub fn get(&self, name: &str) -> Option<&'a str> {
        self.0
            .iter()
            .find(|(k, _)| *k == name)
            .and_then(|(_, v)| *v)
    }

    pub fn iter(&self) -> impl Iterator<Item = &(&'a str, Option<&'a str>)> {
        self.0.iter()
    }

    pub fn new(attrs: &'a [(&'a str, Option<&'a str>)]) -> Self {
        Self(attrs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_with_value() {
        let attrs = ShortcodeAttrs::new(&[("name", Some("Alice"))]);
        assert_eq!(attrs.get("name"), Some("Alice"));
    }

    #[test]
    fn test_get_without_value() {
        let attrs = ShortcodeAttrs::new(&[("flag", None)]);
        assert_eq!(attrs.get("flag"), None);
    }

    #[test]
    fn test_get_missing() {
        let attrs = ShortcodeAttrs::new(&[("name", Some("Alice"))]);
        assert_eq!(attrs.get("missing"), None);
    }

    #[test]
    fn test_get_empty() {
        let attrs = ShortcodeAttrs::new(&[]);
        assert_eq!(attrs.get("anything"), None);
    }

    #[test]
    fn test_iter() {
        let attrs = ShortcodeAttrs::new(&[("a", Some("1")), ("b", None), ("c", Some("3"))]);
        let pairs: Vec<_> = attrs.iter().collect();
        assert_eq!(pairs.len(), 3);
        assert_eq!(pairs[0], &("a", Some("1")));
        assert_eq!(pairs[1], &("b", None));
        assert_eq!(pairs[2], &("c", Some("3")));
    }

    #[test]
    fn test_get_empty_key() {
        let attrs = ShortcodeAttrs::new(&[("", Some("value"))]);
        assert_eq!(attrs.get(""), Some("value"));
    }

    #[test]
    fn test_get_duplicate_keys() {
        let attrs = ShortcodeAttrs::new(&[("id", Some("1")), ("id", Some("2"))]);
        assert_eq!(attrs.get("id"), Some("1"));
    }

    #[test]
    fn test_get_case_sensitive() {
        let attrs = ShortcodeAttrs::new(&[("ID", Some("1"))]);
        assert_eq!(attrs.get("id"), None);
        assert_eq!(attrs.get("ID"), Some("1"));
    }

    #[test]
    fn test_iter_single_flag() {
        let attrs = ShortcodeAttrs::new(&[("autoplay", None)]);
        let pairs: Vec<_> = attrs.iter().collect();
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0], &("autoplay", None));
    }

    #[test]
    fn test_iter_single_value() {
        let attrs = ShortcodeAttrs::new(&[("id", Some("123"))]);
        let pairs: Vec<_> = attrs.iter().collect();
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0], &("id", Some("123")));
    }

    #[test]
    fn test_get_unicode_value() {
        let attrs = ShortcodeAttrs::new(&[("name", Some("世界"))]);
        assert_eq!(attrs.get("name"), Some("世界"));
    }

    #[test]
    fn test_get_empty_string_value() {
        let attrs = ShortcodeAttrs::new(&[("id", Some(""))]);
        assert_eq!(attrs.get("id"), Some(""));
    }

    #[test]
    fn test_get_equals_in_value() {
        let attrs = ShortcodeAttrs::new(&[("url", Some("a=b&c=d"))]);
        assert_eq!(attrs.get("url"), Some("a=b&c=d"));
    }

    #[test]
    fn test_get_ampersand_in_value() {
        let attrs = ShortcodeAttrs::new(&[("src", Some("a&b"))]);
        assert_eq!(attrs.get("src"), Some("a&b"));
    }
}
