//! Shortcode registry and rendering utilities.
//!
//! This module exposes a small API to register shortcode handlers and render
//! strings that contain shortcode tags like `[name]`, `[name]content[/name]`,
//! and attributes such as `[name key="value"]`.
//!
//! Basic usage:
//!
//! ```rust
//! use shortcode_parser::shortcode::Shortcode;
//!
//! let mut sc = Shortcode::new();
//! sc.add("hello", |_, _| "Hello, world!".to_string());
//!
//! assert_eq!(sc.render("[hello]"), "Hello, world!");
//! ```

use crate::parser::Parser;
use crate::renderer::Renderer;
use std::borrow::Cow;
use std::collections::HashMap;

/// Function signature for a shortcode handler.
///
/// Parameters:
/// - `content`: Optional inner content between an opening and closing tag, e.g.
///   in `[name]inner[/name]` this would be `Some("inner")`. For self-closing
///   or attribute-only tags like `[name key="v"]` this will be `None`.
/// - `attrs`: Map of attribute names to optional string values. An attribute may
///   appear without a value (e.g., `[name flag]`), in which case the value is
///   `None`. With a value (e.g., `[name key="v"]`) it will be `Some("v")`.
///
/// Return value should be the rendered replacement string for the shortcode.
///
/// Example:
/// ```rust
/// use shortcode_parser::shortcode::Shortcode;
/// use std::collections::HashMap; // only to show the signature type
///
/// let mut sc = Shortcode::new();
/// sc.add("wrap", |content, attrs| {
///     let left = attrs.get("left").and_then(|v| *v).unwrap_or("[");
///     let right = attrs.get("right").and_then(|v| *v).unwrap_or("]");
///     match content {
///         Some(c) => format!("{left}{c}{right}"),
///         None => String::new(),
///     }
/// });
///
/// assert_eq!(sc.render("[wrap left=\"<\" right=\">\"]hi[/wrap]"), "<hi>");
/// ```
pub type ShortcodeFn = fn(Option<&str>, HashMap<&str, Option<&str>>) -> String;

/// A registry of shortcode handlers keyed by their tag names.
///
/// The lifetime parameter `'a` ties the lifetime of stored tag names to the
/// lifetime of the `Shortcode` instance. Each tag name maps to a function with
/// the `ShortcodeFn` signature.
///
/// Example:
/// ```rust
/// use shortcode_parser::shortcode::Shortcode;
///
/// let mut sc = Shortcode::new();
/// sc.add("test", |_, _| "ok".to_string());
/// assert!(sc.has("test"));
/// assert_eq!(sc.render("[test]"), "ok");
/// ```
#[derive(Debug)]
pub struct Shortcode<'a> {
    items: HashMap<&'a str, ShortcodeFn>,
}

impl<'a> Default for Shortcode<'a> {
    /// Creates a default, empty shortcode registry.
    ///
    /// This is equivalent to calling [`Shortcode::new`].
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Shortcode<'a> {
    /// Creates a new, empty shortcode registry.
    ///
    /// Example:
    /// ```rust
    /// use shortcode_parser::shortcode::Shortcode;
    /// let sc = Shortcode::new();
    /// assert_eq!(sc.render("plain"), "plain");
    /// ```
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
        }
    }

    /// Registers a handler function under the given shortcode `name`.
    ///
    /// If a handler already exists for `name`, it is replaced.
    ///
    /// Example:
    /// ```rust
    /// use shortcode_parser::shortcode::Shortcode;
    /// let mut sc = Shortcode::new();
    /// sc.add("upper", |content, _| content.unwrap_or("").to_uppercase());
    /// assert_eq!(sc.render("[upper]hi[/upper]"), "HI");
    /// ```
    pub fn add(&mut self, name: &'a str, func: ShortcodeFn) {
        self.items.insert(name, func);
    }

    /// Returns `true` if a handler is registered under `name`.
    ///
    /// ```rust
    /// use shortcode_parser::shortcode::Shortcode;
    /// let mut sc = Shortcode::new();
    /// sc.add("x", |_, _| "".to_string());
    /// assert!(sc.has("x"));
    /// assert!(!sc.has("y"));
    /// ```
    pub fn has(&self, name: &str) -> bool {
        self.items.contains_key(name)
    }

    /// Retrieves the handler function registered under `name`, if any.
    ///
    /// ```rust
    /// use shortcode_parser::shortcode::Shortcode;
    /// let mut sc = Shortcode::new();
    /// sc.add("ping", |_, _| "pong".to_string());
    /// let f = sc.get("ping").expect("handler");
    /// assert_eq!(f(None, std::collections::HashMap::new()), "pong");
    /// ```
    pub fn get(&self, name: &str) -> Option<&ShortcodeFn> {
        self.items.get(name)
    }

    /// Parses `content` and renders it by replacing all registered shortcodes.
    ///
    /// - If the string contains no tags, the original string is returned as a
    ///   borrowed `Cow::Borrowed` to avoid allocation.
    /// - Otherwise, returns an owned string with all shortcode expansions
    ///   applied.
    ///
    /// Example:
    /// ```rust
    /// use shortcode_parser::shortcode::Shortcode;
    /// let mut sc = Shortcode::new();
    /// sc.add("greet", |_, attrs| {
    ///     let name = attrs.get("name").and_then(|v| *v).unwrap_or("world");
    ///     format!("Hello, {name}")
    /// });
    ///
    /// assert_eq!(sc.render("[greet name=\"Rust\"]"), "Hello, Rust");
    /// assert_eq!(sc.render("plain text"), "plain text");
    /// ```
    pub fn render<'b>(&self, content: &'b str) -> Cow<'b, str> {
        let mut parser = Parser::new(content);
        let tokens = parser.parse();

        // Only one token and it's not a tag
        if tokens.len() == 1 && tokens[0].tag_name().is_none() {
            return Cow::Borrowed(content);
        }

        Cow::Owned(Renderer::new(tokens).render(&self.items))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shortcode() {
        let mut shortcode = Shortcode::new();
        shortcode.add("test", |_, _| "Hello world".to_string());

        assert_eq!(shortcode.render("[test]"), "Hello world");
    }

    #[test]
    fn test_shortcode_with_content() {
        let mut shortcode = Shortcode::new();
        shortcode.add("test", |content, _| format!("T {} T", content.unwrap()));
        assert_eq!(
            shortcode.render("[test]Hello world[/test]"),
            "T Hello world T"
        );
    }

    #[test]
    fn test_shortcode_with_attr() {
        let mut shortcode = Shortcode::new();
        shortcode.add("test", |_, attrs| {
            format!("T {} T", attrs.get("name").unwrap().unwrap())
        });
        assert_eq!(
            shortcode.render("[test name=\"hello world\"]"),
            "T hello world T"
        );
    }

    #[test]
    fn test_plain_text() {
        let shortcode = Shortcode::new();
        assert_eq!(shortcode.render("Hello world"), "Hello world");
    }

    #[test]
    fn test_multiple_shortcodes() {
        let mut shortcode = Shortcode::new();
        shortcode.add("test", |_, _| "Hello world".to_string());
        shortcode.add("test2", |_, _| "Hello world 2".to_string());
        assert_eq!(
            shortcode.render("[test] [test2]"),
            "Hello world Hello world 2"
        );
    }

    #[test]
    fn test_multiple_shortcodes_with_content() {
        let mut shortcode = Shortcode::new();
        shortcode.add("test", |content, _| format!("T {} T", content.unwrap()));
        shortcode.add("test2", |content, _| format!("T {} T", content.unwrap()));
        assert_eq!(
            shortcode.render("[test]Hello world[/test] [test2]Hello world 2[/test2]"),
            "T Hello world T T Hello world 2 T"
        );
    }

    #[test]
    fn test_multiple_shortcodes_with_attr() {
        let mut shortcode = Shortcode::new();
        shortcode.add("test", |_, attrs| {
            format!("T {} T", attrs.get("name").unwrap().unwrap())
        });
        shortcode.add("test2", |_, attrs| {
            format!("T {} T", attrs.get("name").unwrap().unwrap())
        });
        assert_eq!(
            shortcode.render("[test name=\"hello world\"] [test2 name=\"hello world 2\"]"),
            "T hello world T T hello world 2 T"
        );
    }

    #[test]
    fn test_nested_shortcodes() {
        let mut shortcode = Shortcode::new();
        shortcode.add("test", |_, _| "Hello world".to_string());
        shortcode.add("test2", |content, _| format!("T {} T", content.unwrap()));
        assert_eq!(shortcode.render("[test2][test][/test2]"), "T Hello world T");
    }
}
