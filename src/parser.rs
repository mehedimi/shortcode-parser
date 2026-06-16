use crate::token::Token;

/// Char-by-char shortcode tokenizer that works on raw bytes for performance.
///
/// Avoids `str::chars()` (Unicode decoding) by operating on `&[u8]`.
/// Shortcode tag names and attributes are validated ASCII, so we can safely
/// transmute byte slices to `&str` with `from_utf8_unchecked`.
pub struct Parser<'a> {
    content: &'a str,
    bytes: &'a [u8],
    tokens: Vec<Token<'a>>,
}

impl<'a> Parser<'a> {
    pub fn new(content: &'a str) -> Self {
        Self {
            content,
            bytes: content.as_bytes(),
            tokens: vec![],
        }
    }

    /// Advance past consecutive space bytes, returning the new position.
    fn skip_whitespace(&self, pos: usize) -> usize {
        let mut i = pos;
        while let Some(&b' ') = self.bytes.get(i) {
            i += 1;
        }
        i
    }

    /// Parse attribute name/value pairs from the byte slice between tag name and `]`.
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
                // `key=value` or `key=` (value starts next)
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
                            let value = unsafe { std::str::from_utf8_unchecked(&attr_str[pos..i]) };
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

    /// Parse a single shortcode tag starting at `tag_start` (after `[`).
    ///
    /// Returns `Some(pos)` with the byte position after the closing `]`,
    /// or `None` if no `]` is found (unclosed tag treated as raw text).
    /// Pushes the parsed token only when a closing `]` is found.
    fn parse_shortcode(&mut self, tag_start: usize) -> Option<usize> {
        let mut pos = tag_start;

        while pos < self.bytes.len() {
            match self.bytes[pos] {
                // Space after tag name means attributes follow.
                b' ' => {
                    // SAFETY: tag bytes are a subslice of valid UTF-8 content.
                    let name =
                        unsafe { std::str::from_utf8_unchecked(&self.bytes[tag_start..pos]) };
                    pos += 1;
                    pos = self.skip_whitespace(pos);
                    let attrs_start = pos;
                    // Scan to closing `]` for attributes.
                    while pos < self.bytes.len() && self.bytes[pos] != b']' {
                        pos += 1;
                    }
                    if pos < self.bytes.len() {
                        let attrs = self.parse_attr_value(&self.bytes[attrs_start..pos]);
                        self.tokens.push(Token::SelfCloseAttr(name, attrs));
                        return Some(pos + 1); // Skip `]`
                    } else {
                        return None; // No closing `]` found
                    }
                }
                // Closing bracket — tag is complete.
                b']' => {
                    // SAFETY: Same invariant.
                    let name =
                        unsafe { std::str::from_utf8_unchecked(&self.bytes[tag_start..pos]) };
                    if let Some(rest) = name.strip_prefix('/') {
                        self.tokens.push(Token::CloseTag(rest));
                    } else {
                        self.tokens.push(Token::SelfClose(name));
                    }
                    return Some(pos + 1);
                }
                // Advance to next byte
                _ => {
                    pos += 1;
                }
            }
        }

        None // Reached end of content without finding `]`
    }

    /// Tokenize the full input into a flat list of `Token`s.
    ///
    /// Walks the input byte-by-byte looking for `[`. When found, delegates
    /// to `parse_shortcode` which scans to the matching `]`.
    /// If no `]` is found, the unclosed `[` and everything after it is
    /// treated as raw text.
    pub fn parse(&mut self) -> &Vec<Token<'a>> {
        let mut text_start = 0;
        let mut pos = 0;
        let total_len = self.bytes.len();

        while pos < total_len {
            if self.bytes[pos] == b'[' {
                // Push text before this tag (if any).
                if text_start < pos {
                    self.tokens
                        .push(Token::Text(&self.content[text_start..pos]));
                }
                // Try to parse the shortcode tag.
                if let Some(end_pos) = self.parse_shortcode(pos + 1) {
                    // Found closing `]`, continue parsing after it.
                    pos = end_pos;
                    text_start = pos;
                } else {
                    // No closing `]` found — treat the unclosed `[` as raw text.
                    self.tokens
                        .push(Token::Text(&self.content[pos..total_len]));
                    text_start = total_len;
                    break;
                }
            } else {
                pos += 1;
            }
        }

        // If no tags were found, the entire input is a single text token.
        if self.tokens.is_empty() {
            self.tokens.push(Token::Text(&self.content[text_start..]));
        // Otherwise push any trailing text after the last tag.
        } else if text_start < total_len {
            self.tokens.push(Token::Text(&self.content[text_start..]));
        }

        &self.tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_content() {
        let mut parser = Parser::new("");
        let tokens = parser.parse();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Text(""));
    }

    #[test]
    fn test_parse_without_shortcode() {
        let mut parser = Parser::new("Hello world");
        let tokens = parser.parse();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Text("Hello world"));
    }

    #[test]
    fn test_parse_self_close_shortcode() {
        let mut parser = Parser::new("New [shortcode]");
        let tokens = parser.parse();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], Token::Text("New "));
        assert_eq!(tokens[1], Token::SelfClose("shortcode"));
    }

    #[test]
    fn test_parse_self_close_shortcode_with_attr() {
        let mut parser = Parser::new("New [video autoplay loop]");

        let tokens = parser.parse();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], Token::Text("New "));
        assert_eq!(
            tokens[1],
            Token::SelfCloseAttr("video", vec![("autoplay", None), ("loop", None)])
        );

        let mut parser = Parser::new("New [video id=\"123\"]");
        let tokens = parser.parse();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], Token::Text("New "));
        assert_eq!(
            tokens[1],
            Token::SelfCloseAttr("video", vec![("id", Some("123"))])
        );
    }

    #[test]
    fn test_parse_self_close_shortcode_with_attrs() {
        let mut parser = Parser::new("New [video id=\"123\" autoplay loop name=\"hello world\"]");
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
    fn test_parse_multiple_self_close_shortcodes_with_attrs_and_spaces() {
        let mut parser = Parser::new(
            "New [video id=\"123\" autoplay loop name=\"hello world\"] [audio] [video][test]",
        );
        let tokens = parser.parse();
        // 7 tokens: no empty Text("") between adjacent tags.
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
    fn test_parse_enclosed_shortcode() {
        let mut parser = Parser::new("New [bold]Word[/bold]");
        let tokens = parser.parse();

        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0], Token::Text("New "));
        assert_eq!(tokens[1], Token::SelfClose("bold"));
        assert_eq!(tokens[2], Token::Text("Word"));
        assert_eq!(tokens[3], Token::CloseTag("bold"));
    }

    #[test]
    fn test_parse_unclosed_tag() {
        let mut parser = Parser::new("Hello [unclosed");
        let tokens = parser.parse();
        // Preceding text + unclosed tag as separate text tokens.
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], Token::Text("Hello "));
        assert_eq!(tokens[1], Token::Text("[unclosed"));
    }

    #[test]
    fn test_parse_unclosed_tag_with_attr() {
        let mut parser = Parser::new("Hello [video id=\"123");
        let tokens = parser.parse();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], Token::Text("Hello "));
        assert_eq!(tokens[1], Token::Text("[video id=\"123"));
    }

    #[test]
    fn test_parse_unclosed_tag_followed_by_valid() {
        // `[unclosed ` triggers attribute parsing, `[valid` becomes an attr name,
        // and the final `]` closes the tag. This is correct for a lenient parser.
        let mut parser = Parser::new("[unclosed [valid]");
        let tokens = parser.parse();
        assert_eq!(tokens.len(), 1);
        assert_eq!(
            tokens[0],
            Token::SelfCloseAttr("unclosed", vec![("[valid", None)])
        );
    }

    #[test]
    fn test_parse_valid_then_unclosed() {
        let mut parser = Parser::new("[valid] text [unclosed");
        let tokens = parser.parse();
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0], Token::SelfClose("valid"));
        assert_eq!(tokens[1], Token::Text(" text "));
        assert_eq!(tokens[2], Token::Text("[unclosed"));
    }
}
