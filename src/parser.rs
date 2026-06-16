use crate::tokenizer::{TokenSegment, Tokenizer};
use crate::token::Token;

/// Interprets raw token segments into `Token` enum values.
///
/// Handles tag name extraction (including `/` prefix for close tags)
/// and attribute parsing (`key="value" flag` format).
pub struct Parser<'a> {
    segments: Vec<TokenSegment<'a>>,
}

impl<'a> Parser<'a> {
    pub fn new(content: &'a str) -> Self {
        let segments = Tokenizer::new(content).tokenize();
        Self { segments }
    }

    /// Interpret all segments and return the token list.
    pub fn parse(&self) -> Vec<Token<'a>> {
        self.segments.iter().map(|seg| self.interpret(seg)).collect()
    }

    /// Interpret a single raw segment into a `Token`.
    fn interpret(&self, segment: &TokenSegment<'a>) -> Token<'a> {
        match segment {
            TokenSegment::Text(text) => Token::Text(text),
            TokenSegment::Tag(raw) => self.parse_tag(raw),
        }
    }

    /// Interpret a raw tag byte slice into the appropriate `Token` variant.
    ///
    /// The slice is the content between `[` and `]` (brackets excluded).
    /// - If it contains a space, the part before the space is the tag name
    ///   and the rest is parsed as attributes.
    /// - If it contains no space and starts with `/`, it's a close tag.
    /// - Otherwise it's a self-closing tag.
    fn parse_tag(&self, raw: &'a [u8]) -> Token<'a> {
        // Find the first space to separate tag name from attributes.
        let space_pos = raw.iter().position(|&b| b == b' ');

        if let Some(pos) = space_pos {
            // Tag has attributes.
            let name = unsafe { std::str::from_utf8_unchecked(&raw[..pos]) };
            let attr_bytes = &raw[pos + 1..];
            let attrs = self.parse_attr_value(attr_bytes);
            Token::SelfCloseAttr(name, attrs)
        } else {
            // No attributes — check if it's a close tag.
            // SAFETY: raw is a subslice of valid UTF-8 content.
            let name = unsafe { std::str::from_utf8_unchecked(raw) };
            if let Some(tag_name) = name.strip_prefix('/') {
                Token::CloseTag(tag_name)
            } else {
                Token::SelfClose(name)
            }
        }
    }

    /// Parse attribute name/value pairs from raw bytes.
    ///
    /// Format: `key="value" flag key2="value2"`
    /// Returns empty vec for whitespace-only input.
    fn parse_attr_value(&self, attr_str: &'a [u8]) -> Vec<(&'a str, Option<&'a str>)> {
        let mut attrs = vec![];
        let mut pos = 0;
        let mut i = 0;
        let len = attr_str.len();

        while i < len {
            match attr_str[i] {
                // Space separates attributes. Push the name collected since `pos`.
                b' ' => {
                    if pos != i {
                        // SAFETY: `attr_str` is a subslice of valid UTF-8 content.
                        // Attribute names in shortcodes are ASCII identifiers.
                        attrs.push((
                            unsafe { std::str::from_utf8_unchecked(&attr_str[pos..i]) },
                            None,
                        ));
                    }
                    pos = i + 1;
                    i += 1;
                }
                // `key=value` — parse until quote or next space.
                b'=' => {
                    // SAFETY: Same invariant — attr_str is a subslice of valid UTF-8.
                    let name = unsafe { std::str::from_utf8_unchecked(&attr_str[pos..i]) };
                    i += 1;

                    // Skip any whitespace between `=` and the opening quote.
                    while i < len && attr_str[i] != b'"' && attr_str[i] != b'\'' {
                        i += 1;
                    }
                    // Found opening quote — parse until closing quote.
                    if i < len {
                        let quote = attr_str[i];
                        i += 1;
                        pos = i;
                        while i < len && attr_str[i] != quote {
                            i += 1;
                        }
                        if i < len {
                            // SAFETY: Quoted values are subslices of valid UTF-8 content.
                            let value =
                                unsafe { std::str::from_utf8_unchecked(&attr_str[pos..i]) };
                            attrs.push((name, Some(value)));
                        }
                        i += 1; // Skip closing quote
                        pos = i;
                    }
                }
                // Advance to next byte
                _ => {
                    i += 1;
                }
            }
        }

        // Push trailing name without value (e.g., `flag` at end: `[tag flag]`)
        if pos < len {
            // SAFETY: Same invariant.
            attrs.push((
                unsafe { std::str::from_utf8_unchecked(&attr_str[pos..]) },
                None,
            ));
        }

        attrs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty() {
        let parser = Parser::new("");
        let tokens = parser.parse();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Text(""));
    }

    #[test]
    fn test_parse_without_shortcode() {
        let parser = Parser::new("Hello world");
        let tokens = parser.parse();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Text("Hello world"));
    }

    #[test]
    fn test_parse_self_close() {
        let parser = Parser::new("New [shortcode]");
        let tokens = parser.parse();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], Token::Text("New "));
        assert_eq!(tokens[1], Token::SelfClose("shortcode"));
    }

    #[test]
    fn test_parse_self_close_with_attr() {
        let parser = Parser::new("New [video autoplay loop]");
        let tokens = parser.parse();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], Token::Text("New "));
        assert_eq!(
            tokens[1],
            Token::SelfCloseAttr("video", vec![("autoplay", None), ("loop", None)])
        );

        let parser = Parser::new("New [video id=\"123\"]");
        let tokens = parser.parse();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], Token::Text("New "));
        assert_eq!(
            tokens[1],
            Token::SelfCloseAttr("video", vec![("id", Some("123"))])
        );
    }

    #[test]
    fn test_parse_self_close_with_attrs() {
        let parser = Parser::new("New [video id=\"123\" autoplay loop name=\"hello world\"]");
        let tokens = parser.parse();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], Token::Text("New "));
        assert_eq!(
            tokens[1],
            Token::SelfCloseAttr(
                "video",
                vec![
                    ("id", Some("123")),
                    ("autoplay", None),
                    ("loop", None),
                    ("name", Some("hello world"))
                ]
            )
        );
    }

    #[test]
    fn test_parse_multiple() {
        let parser = Parser::new(
            "New [video id=\"123\" autoplay loop name=\"hello world\"] [audio] [video][test]",
        );
        let tokens = parser.parse();
        assert_eq!(tokens.len(), 7);
        assert_eq!(tokens[0], Token::Text("New "));
        assert_eq!(
            tokens[1],
            Token::SelfCloseAttr(
                "video",
                vec![
                    ("id", Some("123")),
                    ("autoplay", None),
                    ("loop", None),
                    ("name", Some("hello world"))
                ]
            )
        );
        assert_eq!(tokens[2], Token::Text(" "));
        assert_eq!(tokens[3], Token::SelfClose("audio"));
        assert_eq!(tokens[4], Token::Text(" "));
        assert_eq!(tokens[5], Token::SelfClose("video"));
        assert_eq!(tokens[6], Token::SelfClose("test"));
    }

    #[test]
    fn test_parse_enclosed() {
        let parser = Parser::new("New [bold]Word[/bold]");
        let tokens = parser.parse();

        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0], Token::Text("New "));
        assert_eq!(tokens[1], Token::SelfClose("bold"));
        assert_eq!(tokens[2], Token::Text("Word"));
        assert_eq!(tokens[3], Token::CloseTag("bold"));
    }

    #[test]
    fn test_parse_unclosed_tag() {
        let parser = Parser::new("Hello [unclosed");
        let tokens = parser.parse();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], Token::Text("Hello "));
        assert_eq!(tokens[1], Token::Text("[unclosed"));
    }

    #[test]
    fn test_parse_unclosed_tag_with_attr() {
        let parser = Parser::new("Hello [video id=\"123");
        let tokens = parser.parse();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], Token::Text("Hello "));
        assert_eq!(tokens[1], Token::Text("[video id=\"123"));
    }

    #[test]
    fn test_parse_unclosed_tag_followed_by_valid() {
        let parser = Parser::new("[unclosed [valid]");
        let tokens = parser.parse();
        // `[unclosed ` triggers attribute parsing, `[valid` becomes an attr name,
        // and the final `]` closes the tag. This is correct for a lenient parser.
        assert_eq!(tokens.len(), 1);
        assert_eq!(
            tokens[0],
            Token::SelfCloseAttr("unclosed", vec![("[valid", None)])
        );
    }

    #[test]
    fn test_parse_valid_then_unclosed() {
        let parser = Parser::new("[valid] text [unclosed");
        let tokens = parser.parse();
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0], Token::SelfClose("valid"));
        assert_eq!(tokens[1], Token::Text(" text "));
        assert_eq!(tokens[2], Token::Text("[unclosed"));
    }
}
