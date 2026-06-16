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
}
