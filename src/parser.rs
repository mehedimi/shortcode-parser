use crate::token::Token;
use std::str::Chars;

pub struct Parser<'a> {
    content: &'a str,
    pos: usize,
    tokens: Vec<Token<'a>>,
}

impl<'a> Parser<'a> {
    pub fn new(content: &'a str) -> Self {
        Self {
            content,
            pos: 0,
            tokens: vec![],
        }
    }

    fn get_attr_end_range(&self, iter: &mut Chars, quote: Option<char>, i: &mut usize) -> usize {
        loop {
            let c = iter.next();

            if c.is_none() {
                return *i;
            }

            if c == quote {
                return *i;
            }

            *i += 1;
        }
    }

    fn parse_attr_value(&mut self, attr_str: &'a str) -> Vec<(&'a str, Option<&'a str>)> {
        let mut attrs = vec![];
        let mut iter = attr_str.chars();
        let mut pos = 0;
        let mut i = 0;

        loop {
            let c = iter.next();

            match c {
                Some(' ') => {
                    if pos == i {
                        pos += 1;
                        i += 1;
                        continue;
                    }
                    attrs.push((&attr_str[pos..i], None));
                    pos = i + 1;
                    i += 1;
                }
                Some('=') => {
                    let name = &attr_str[pos..i];

                    i += 2;
                    pos = i;

                    loop {
                        let a = iter.next();

                        match a {
                            Some('"') | Some('\'') => {
                                let value =
                                    &attr_str[pos..self.get_attr_end_range(&mut iter, a, &mut i)];
                                attrs.push((name, Some(value)));

                                i += 1;
                                pos = i;

                                break;
                            }
                            None => break,
                            _ => {
                                i += 1;
                            }
                        }
                    }
                }
                None => break,
                _ => {
                    i += 1;
                }
            }
        }

        if pos != i {
            attrs.push((&attr_str[pos..], None));
        }

        attrs
    }

    fn parse_attrs(&mut self, char_iter: &mut Chars) -> Vec<(&'a str, Option<&'a str>)> {
        let mut attrs = vec![];
        let mut i = self.pos;
        loop {
            let c = char_iter.next();

            match c {
                Some(']') => {
                    attrs = self.parse_attr_value(&self.content[self.pos..i]);
                    i += 1;
                    self.pos = i;
                    break;
                }
                None => break,
                _ => {
                    i += 1;
                }
            }
        }

        attrs
    }

    fn parse_shortcode(&mut self, char_iter: &mut Chars) {
        let mut i = self.pos;

        loop {
            let c = char_iter.next();

            match c {
                Some(' ') => {
                    let name = &self.content[self.pos..i];
                    // Increment position to skip the space
                    i += 1;
                    self.pos = i;
                    let attrs = self.parse_attrs(char_iter);
                    self.tokens.push(Token::SelfCloseAttr(name, attrs));
                    break;
                }
                Some(']') => {
                    let name = &self.content[self.pos..i];
                    if name.starts_with('/') {
                        self.tokens.push(Token::CloseTag(&name[1..]));
                    } else {
                        self.tokens.push(Token::SelfClose(name));
                    }
                    self.pos = i + 1;
                    break;
                }
                None => break,
                _ => {
                    i += 1;
                }
            }
        }
    }

    pub fn parse(&mut self) -> &Vec<Token<'a>> {
        let mut iter = self.content.chars();
        let mut i = 0;

        loop {
            let c = iter.next();
            match c {
                Some('[') => {
                    self.tokens.push(Token::Text(&self.content[self.pos..i]));
                    i += 1;
                    // Set position start of the shortcode tag
                    self.pos = i;
                    self.parse_shortcode(&mut iter);
                    i = self.pos;
                }
                None => break,
                _ => {
                    i += 1;
                }
            }
        }

        if self.pos == 0 {
            self.tokens.push(Token::Text(&self.content[self.pos..]));
        }

        self.tokens.as_ref()
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
        assert_eq!(tokens.len(), 8);
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
        assert_eq!(tokens[6], Token::Text(""));
        assert_eq!(tokens[7], Token::SelfClose("test"));
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
}
