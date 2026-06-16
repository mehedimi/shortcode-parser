/// Byte-level scanner that identifies text regions and tag boundaries.
///
/// Walks the input looking for `[` and `]`, extracting raw segments.
/// Does not interpret tag names or attributes — that is the parser's job.
/// If an unclosed `[` is encountered, everything from there to the end
/// is treated as raw text.
pub struct Tokenizer<'a> {
    content: &'a str,
    bytes: &'a [u8],
}

/// A raw segment produced by the tokenizer.
/// Either plain text between tags, or the raw bytes between `[` and `]`.
#[derive(Debug, PartialEq)]
pub enum TokenSegment<'a> {
    Text(&'a str),
    Tag(&'a [u8]),
}

impl<'a> Tokenizer<'a> {
    pub fn new(content: &'a str) -> Self {
        Self {
            content,
            bytes: content.as_bytes(),
        }
    }

    /// Scan the input and return raw segments.
    ///
    /// Returns a list of `TokenSegment` values:
    /// - `Text` for regions between tags
    /// - `Tag` for the raw bytes between `[` and `]` (brackets excluded)
    ///
    /// If no tags are found, returns a single `Text` segment covering
    /// the entire input. If an unclosed `[` is encountered, the rest
    /// of the input becomes a single `Text` segment.
    pub fn tokenize(&self) -> Vec<TokenSegment<'a>> {
        let mut segments = vec![];
        let mut text_start = 0;
        let mut pos = 0;
        let total_len = self.bytes.len();

        while pos < total_len {
            if self.bytes[pos] == b'[' {
                // Push text before this tag (if any).
                if text_start < pos {
                    segments.push(TokenSegment::Text(&self.content[text_start..pos]));
                }
                let bracket_pos = pos;
                // Scan to closing `]`.
                pos += 1; // Skip `[`
                let tag_start = pos;
                while pos < total_len && self.bytes[pos] != b']' {
                    pos += 1;
                }
                if pos < total_len {
                    // Found closing `]` — emit tag segment.
                    segments.push(TokenSegment::Tag(&self.bytes[tag_start..pos]));
                    pos += 1; // Skip `]`
                    text_start = pos;
                } else {
                    // No closing `]` — treat `[` and everything after as text.
                    segments.push(TokenSegment::Text(&self.content[bracket_pos..total_len]));
                    text_start = total_len;
                    break;
                }
            } else {
                pos += 1;
            }
        }

        // Push any remaining text after the last tag.
        if text_start < total_len {
            segments.push(TokenSegment::Text(&self.content[text_start..]));
        } else if segments.is_empty() {
            // No tags found and no trailing text — entire input is text.
            segments.push(TokenSegment::Text(self.content));
        }

        segments
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_empty() {
        let tok = Tokenizer::new("");
        let segs = tok.tokenize();
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0], TokenSegment::Text(""));
    }

    #[test]
    fn test_tokenize_no_tags() {
        let tok = Tokenizer::new("Hello world");
        let segs = tok.tokenize();
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0], TokenSegment::Text("Hello world"));
    }

    #[test]
    fn test_tokenize_self_close() {
        let tok = Tokenizer::new("New [shortcode]");
        let segs = tok.tokenize();
        assert_eq!(segs.len(), 2);
        assert_eq!(segs[0], TokenSegment::Text("New "));
        assert_eq!(segs[1], TokenSegment::Tag(b"shortcode"));
    }

    #[test]
    fn test_tokenize_with_attrs() {
        let tok = Tokenizer::new("New [video id=\"123\"]");
        let segs = tok.tokenize();
        assert_eq!(segs.len(), 2);
        assert_eq!(segs[0], TokenSegment::Text("New "));
        assert_eq!(segs[1], TokenSegment::Tag(b"video id=\"123\""));
    }

    #[test]
    fn test_tokenize_multiple() {
        let tok = Tokenizer::new("[a] [b]");
        let segs = tok.tokenize();
        // Empty text segments at boundaries are skipped.
        assert_eq!(segs.len(), 3);
        assert_eq!(segs[0], TokenSegment::Tag(b"a"));
        assert_eq!(segs[1], TokenSegment::Text(" "));
        assert_eq!(segs[2], TokenSegment::Tag(b"b"));
    }

    #[test]
    fn test_tokenize_enclosed() {
        let tok = Tokenizer::new("[bold]Word[/bold]");
        let segs = tok.tokenize();
        assert_eq!(segs.len(), 3);
        assert_eq!(segs[0], TokenSegment::Tag(b"bold"));
        assert_eq!(segs[1], TokenSegment::Text("Word"));
        assert_eq!(segs[2], TokenSegment::Tag(b"/bold"));
    }

    #[test]
    fn test_tokenize_unclosed_tag() {
        let tok = Tokenizer::new("Hello [unclosed");
        let segs = tok.tokenize();
        assert_eq!(segs.len(), 2);
        assert_eq!(segs[0], TokenSegment::Text("Hello "));
        assert_eq!(segs[1], TokenSegment::Text("[unclosed"));
    }

    #[test]
    fn test_tokenize_unclosed_after_valid() {
        let tok = Tokenizer::new("[valid] text [unclosed");
        let segs = tok.tokenize();
        assert_eq!(segs.len(), 3);
        assert_eq!(segs[0], TokenSegment::Tag(b"valid"));
        assert_eq!(segs[1], TokenSegment::Text(" text "));
        assert_eq!(segs[2], TokenSegment::Text("[unclosed"));
    }

    #[test]
    fn test_tokenize_empty_brackets() {
        let tok = Tokenizer::new("[]");
        let segs = tok.tokenize();
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0], TokenSegment::Tag(b""));
    }

    #[test]
    fn test_tokenize_multiple_empty_brackets() {
        let tok = Tokenizer::new("[][]");
        let segs = tok.tokenize();
        assert_eq!(segs.len(), 2);
        assert_eq!(segs[0], TokenSegment::Tag(b""));
        assert_eq!(segs[1], TokenSegment::Tag(b""));
    }

    #[test]
    fn test_tokenize_whitespace_only() {
        let tok = Tokenizer::new("   ");
        let segs = tok.tokenize();
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0], TokenSegment::Text("   "));
    }

    #[test]
    fn test_tokenize_multiple_unclosed() {
        let tok = Tokenizer::new("[a [b [c");
        let segs = tok.tokenize();
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0], TokenSegment::Text("[a [b [c"));
    }

    #[test]
    fn test_tokenize_unclosed_in_middle() {
        let tok = Tokenizer::new("[valid] text [unclosed more [another");
        let segs = tok.tokenize();
        assert_eq!(segs.len(), 3);
        assert_eq!(segs[0], TokenSegment::Tag(b"valid"));
        assert_eq!(segs[1], TokenSegment::Text(" text "));
        assert_eq!(segs[2], TokenSegment::Text("[unclosed more [another"));
    }

    #[test]
    fn test_tokenize_tag_with_special_chars() {
        let tok = Tokenizer::new("[video id=\"a=b&c=d\"]");
        let segs = tok.tokenize();
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0], TokenSegment::Tag(b"video id=\"a=b&c=d\""));
    }

    #[test]
    fn test_tokenize_unicode_content() {
        let tok = Tokenizer::new("[greet name=\"世界\"]");
        let segs = tok.tokenize();
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0], TokenSegment::Tag("greet name=\"世界\"".as_bytes()));
    }

    #[test]
    fn test_tokenize_unicode_text() {
        let tok = Tokenizer::new("こんにちは [emoji]世界![/emoji]");
        let segs = tok.tokenize();
        // Flat scanner finds both [emoji] and [/emoji].
        assert_eq!(segs.len(), 4);
        assert_eq!(segs[0], TokenSegment::Text("こんにちは "));
        assert_eq!(segs[1], TokenSegment::Tag(b"emoji"));
        assert_eq!(segs[2], TokenSegment::Text("世界!"));
        assert_eq!(segs[3], TokenSegment::Tag(b"/emoji"));
    }

    #[test]
    fn test_tokenize_tag_with_hyphens() {
        let tok = Tokenizer::new("[my-short-code]");
        let segs = tok.tokenize();
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0], TokenSegment::Tag(b"my-short-code"));
    }

    #[test]
    fn test_tokenize_tag_with_underscores() {
        let tok = Tokenizer::new("[my_shortcode]");
        let segs = tok.tokenize();
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0], TokenSegment::Tag(b"my_shortcode"));
    }

    #[test]
    fn test_tokenize_tag_with_numbers() {
        let tok = Tokenizer::new("[shortcode123]");
        let segs = tok.tokenize();
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0], TokenSegment::Tag(b"shortcode123"));
    }

    #[test]
    fn test_tokenize_nested_brackets_unclosed() {
        // Tokenizer finds the first ] pair, then the rest is text.
        let tok = Tokenizer::new("[outer [inner] text");
        let segs = tok.tokenize();
        assert_eq!(segs.len(), 2);
        assert_eq!(segs[0], TokenSegment::Tag(b"outer [inner"));
        assert_eq!(segs[1], TokenSegment::Text(" text"));
    }

    #[test]
    fn test_tokenize_just_open_bracket() {
        let tok = Tokenizer::new("[");
        let segs = tok.tokenize();
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0], TokenSegment::Text("["));
    }

    #[test]
    fn test_tokenize_just_close_bracket() {
        let tok = Tokenizer::new("]");
        let segs = tok.tokenize();
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0], TokenSegment::Text("]"));
    }

    #[test]
    fn test_tokenize_consecutive_tags() {
        let tok = Tokenizer::new("[a][b][c]");
        let segs = tok.tokenize();
        assert_eq!(segs.len(), 3);
        assert_eq!(segs[0], TokenSegment::Tag(b"a"));
        assert_eq!(segs[1], TokenSegment::Tag(b"b"));
        assert_eq!(segs[2], TokenSegment::Tag(b"c"));
    }

    #[test]
    fn test_tokenize_mixed_nested() {
        // Tokenizer is a flat bracket scanner; it finds all ] pairs.
        let tok = Tokenizer::new("[a][b [c][/b][/a]");
        let segs = tok.tokenize();
        assert_eq!(segs.len(), 4);
        assert_eq!(segs[0], TokenSegment::Tag(b"a"));
        assert_eq!(segs[1], TokenSegment::Tag(b"b [c"));
        assert_eq!(segs[2], TokenSegment::Tag(b"/b"));
        assert_eq!(segs[3], TokenSegment::Tag(b"/a"));
    }

    #[test]
    fn test_tokenize_empty_after_tag() {
        let tok = Tokenizer::new("[tag]");
        let segs = tok.tokenize();
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0], TokenSegment::Tag(b"tag"));
    }

    #[test]
    fn test_tokenize_text_before_and_after() {
        let tok = Tokenizer::new("before [tag] after");
        let segs = tok.tokenize();
        assert_eq!(segs.len(), 3);
        assert_eq!(segs[0], TokenSegment::Text("before "));
        assert_eq!(segs[1], TokenSegment::Tag(b"tag"));
        assert_eq!(segs[2], TokenSegment::Text(" after"));
    }

    #[test]
    fn test_tokenize_deeply_nested() {
        let tok = Tokenizer::new("[a][b][c][d][/d][/c][/b][/a]");
        let segs = tok.tokenize();
        // Tokenizer finds all bracket pairs; no trailing text after last tag.
        assert_eq!(segs.len(), 8);
        assert_eq!(segs[0], TokenSegment::Tag(b"a"));
        assert_eq!(segs[1], TokenSegment::Tag(b"b"));
        assert_eq!(segs[2], TokenSegment::Tag(b"c"));
        assert_eq!(segs[3], TokenSegment::Tag(b"d"));
        assert_eq!(segs[4], TokenSegment::Tag(b"/d"));
        assert_eq!(segs[5], TokenSegment::Tag(b"/c"));
        assert_eq!(segs[6], TokenSegment::Tag(b"/b"));
        assert_eq!(segs[7], TokenSegment::Tag(b"/a"));
    }

    #[test]
    fn test_tokenize_single_quote_attr() {
        let tok = Tokenizer::new("[video id='123']");
        let segs = tok.tokenize();
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0], TokenSegment::Tag(b"video id='123'"));
    }

    #[test]
    fn test_tokenize_empty_attr_value() {
        let tok = Tokenizer::new("[video id='']");
        let segs = tok.tokenize();
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0], TokenSegment::Tag(b"video id=''"));
    }

    #[test]
    fn test_tokenize_trailing_slash_bracket() {
        let tok = Tokenizer::new("[/");
        let segs = tok.tokenize();
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0], TokenSegment::Text("[/"));
    }

    #[test]
    fn test_tokenize_multiple_close_tags() {
        let tok = Tokenizer::new("[/a][/b][/c]");
        let segs = tok.tokenize();
        assert_eq!(segs.len(), 3);
        assert_eq!(segs[0], TokenSegment::Tag(b"/a"));
        assert_eq!(segs[1], TokenSegment::Tag(b"/b"));
        assert_eq!(segs[2], TokenSegment::Tag(b"/c"));
    }

    #[test]
    fn test_tokenize_space_only_tag() {
        let tok = Tokenizer::new("[ ]");
        let segs = tok.tokenize();
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0], TokenSegment::Tag(b" "));
    }

    #[test]
    fn test_tokenize_mixed_valid_unclosed_valid() {
        let tok = Tokenizer::new("[a] unclosed [b]");
        let segs = tok.tokenize();
        assert_eq!(segs.len(), 3);
        assert_eq!(segs[0], TokenSegment::Tag(b"a"));
        assert_eq!(segs[1], TokenSegment::Text(" unclosed "));
        assert_eq!(segs[2], TokenSegment::Tag(b"b"));
    }

    #[test]
    fn test_tokenize_html_in_attr() {
        let tok = Tokenizer::new("[video html='<div>']");
        let segs = tok.tokenize();
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0], TokenSegment::Tag(b"video html='<div>'"));
    }

    #[test]
    fn test_tokenize_nested_close_in_content() {
        // Tokenizer is a flat bracket scanner; it finds all ] pairs.
        let tok = Tokenizer::new("[outer]text[/inner]more[/outer]");
        let segs = tok.tokenize();
        assert_eq!(segs.len(), 5);
        assert_eq!(segs[0], TokenSegment::Tag(b"outer"));
        assert_eq!(segs[1], TokenSegment::Text("text"));
        assert_eq!(segs[2], TokenSegment::Tag(b"/inner"));
        assert_eq!(segs[3], TokenSegment::Text("more"));
        assert_eq!(segs[4], TokenSegment::Tag(b"/outer"));
    }

    #[test]
    fn test_tokenize_very_long_attr() {
        let long_val = "x".repeat(1000);
        let input = format!("[video id=\"{}\"]", long_val);
        let tok = Tokenizer::new(&input);
        let segs = tok.tokenize();
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0], TokenSegment::Tag(format!("video id=\"{}\"", long_val).as_bytes()));
    }

    #[test]
    fn test_tokenize_only_close_tag() {
        let tok = Tokenizer::new("[/tag]");
        let segs = tok.tokenize();
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0], TokenSegment::Tag(b"/tag"));
    }

    #[test]
    fn test_tokenize_double_space_in_tag() {
        let tok = Tokenizer::new("[video  id=\"123\"]");
        let segs = tok.tokenize();
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0], TokenSegment::Tag(b"video  id=\"123\""));
    }

    #[test]
    fn test_tokenize_bracket_in_text() {
        let tok = Tokenizer::new("a > b [tag] c < d");
        let segs = tok.tokenize();
        assert_eq!(segs.len(), 3);
        assert_eq!(segs[0], TokenSegment::Text("a > b "));
        assert_eq!(segs[1], TokenSegment::Tag(b"tag"));
        assert_eq!(segs[2], TokenSegment::Text(" c < d"));
    }

    #[test]
    fn test_tokenize_newlines() {
        let tok = Tokenizer::new("[tag]\ncontent\n[/tag]");
        let segs = tok.tokenize();
        assert_eq!(segs.len(), 3);
        assert_eq!(segs[0], TokenSegment::Tag(b"tag"));
        assert_eq!(segs[1], TokenSegment::Text("\ncontent\n"));
        assert_eq!(segs[2], TokenSegment::Tag(b"/tag"));
    }

    #[test]
    fn test_tokenize_tabs() {
        let tok = Tokenizer::new("[tag]\tcontent\t[/tag]");
        let segs = tok.tokenize();
        assert_eq!(segs.len(), 3);
        assert_eq!(segs[0], TokenSegment::Tag(b"tag"));
        assert_eq!(segs[1], TokenSegment::Text("\tcontent\t"));
        assert_eq!(segs[2], TokenSegment::Tag(b"/tag"));
    }

    #[test]
    fn test_tokenize_mixed_whitespace() {
        let tok = Tokenizer::new("   [tag]   [/tag]   ");
        let segs = tok.tokenize();
        // Leading text, tag, middle text, tag, trailing text.
        assert_eq!(segs.len(), 5);
        assert_eq!(segs[0], TokenSegment::Text("   "));
        assert_eq!(segs[1], TokenSegment::Tag(b"tag"));
        assert_eq!(segs[2], TokenSegment::Text("   "));
        assert_eq!(segs[3], TokenSegment::Tag(b"/tag"));
        assert_eq!(segs[4], TokenSegment::Text("   "));
    }
}
