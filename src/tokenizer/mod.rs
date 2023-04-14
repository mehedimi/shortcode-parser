use crate::tokenizer::token::Token;
use std::collections::HashMap;

mod render;
mod token;

pub struct Parser {
    tokens: Vec<Token>,
    state: State,
    text: String,
    tag: String,
    is_tag_end: bool,
    attrs: HashMap<String, Option<String>>,
    attr_key: String,
    attr_value: String,
    attr_quote: char,
}

impl Parser {
    pub fn new() -> Parser {
        Parser {
            tokens: Vec::new(),
            state: State::Text,
            text: String::new(),
            tag: String::new(),
            is_tag_end: false,
            attrs: HashMap::new(),
            attr_key: String::new(),
            attr_value: String::new(),
            attr_quote: '"'
        }
    }

    pub fn parse(&mut self, content: &String) -> Vec<Token> {
        for char in content.chars() {
            match self.state {
                State::Text => self.parse_text(&char),
                State::TagStart => self.parse_tag_start(&char),
                State::AttrKey => self.parse_attr_key(&char),
                State::AttrValueStart => self.parse_attr_value_start(&char),
                State::AttrValueEnd => self.parse_attr_value_end(&char)
            }
        }

        if !self.text.is_empty() {
            self.add_text_tag();
            self.text.clear();
        }

        let tokens = self.tokens.clone();

        self.tokens.clear();

        return tokens;
    }

    fn parse_text(&mut self, char: &char) {
        if *char == '[' {
            if !self.text.is_empty() {
                self.tokens.push(Token::Text {
                    content: self.text.clone(),
                });
                self.text.clear();
            }

            self.state = State::TagStart;
        } else {
            self.text.push(*char);
        }
    }

    fn parse_tag_start(&mut self, char: &char) {
        match char {
            ' ' => {
                self.state = State::AttrKey;
            }
            '/' => {
                self.is_tag_end = true;
            }
            ']' => {
                if self.is_tag_end {
                    let start_tag_index = self.get_start_tag_index();

                    match start_tag_index {
                        Some(index) => {
                            let start_tag = self.tokens.get(index).unwrap();
                            let children = &self.tokens[(index + 1)..];
                            match start_tag {
                                Token::InlineTag { tag } => {
                                    self.tokens.splice(
                                        index..,
                                        Vec::from([Token::NestedInlineTag {
                                            tag: tag.clone(),
                                            children: Vec::from(children),
                                        }]),
                                    );
                                }
                                Token::AttributeTag { tag, attrs } => {
                                    self.tokens.splice(
                                        index..,
                                        Vec::from([Token::NestedAttributeTag {
                                            tag: tag.clone(),
                                            attrs: attrs.clone(),
                                            children: Vec::from(children),
                                        }]),
                                    );
                                }
                                _ => {}
                            }
                        }
                        None => self.tokens.push({
                            Token::Text {
                                content: "[".to_owned() + &self.tag.clone(),
                            }
                        }),
                    }
                    self.is_tag_end = false;
                } else {
                    self.add_inline_tag();
                }

                self.tag.clear();
                self.state = State::Text;
            }
            _ => {
                self.tag.push(*char);
            }
        }
    }

    fn parse_attr_key(&mut self, char: &char) {
        match char {
            '=' => {
                self.state = State::AttrValueStart;
            }
            ']' => {
                self.add_attribute_inline_tag();

                self.tag.clear();
                self.attrs.clear();
                self.state = State::Text;
            },
            ' ' => {
                self.add_inline_attr();
                self.attr_key.clear();
            },
            _ => {
                self.attr_key.push(*char);
            }
        }
    }

    fn parse_attr_value_start(&mut self, char: &char) {
        if *char == '"' || *char == '\'' {
            self.state = State::AttrValueEnd;
            self.attr_quote = *char;
        }
    }

    fn parse_attr_value_end(&mut self, char: &char) {
        if *char == self.attr_quote {
            self.add_to_attrs();
            self.attr_key.clear();
            self.attr_value.clear();
            self.state = State::AttrKey;
        } else {
            self.attr_value.push(*char);
        }
    }

    fn get_start_tag_index(&self) -> Option<usize> {
        self.tokens.iter().position(|token| match token.tag_name() {
            None => false,
            Some(tag) => self.tag == tag,
        })
    }

    fn add_inline_tag(&mut self) {
        self.tokens.push(Token::InlineTag {
            tag: self.tag.clone(),
        });
    }

    fn add_text_tag(&mut self) {
        self.tokens.push(Token::Text {
            content: self.text.clone(),
        })
    }

    fn add_attribute_inline_tag(&mut self) {
        if !self.attr_key.is_empty() {
            self.attrs.insert(self.attr_key.clone(), None);
        }

        self.tokens.push(Token::AttributeTag {
            tag: self.tag.clone(),
            attrs: self.attrs.clone(),
        })
    }

    fn add_to_attrs(&mut self) {
        self.attrs.insert(
            self.attr_key.trim().clone().to_string(),
            Some(self.attr_value.clone()),
        );
    }

    fn add_inline_attr(&mut self) {
        self.attrs.insert(self.attr_key.clone(), None);
    }
}

enum State {
    Text,
    TagStart,
    AttrKey,
    AttrValueStart,
    AttrValueEnd,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parsing_only_text() {
        let text = String::from("Demo random something");

        let parsed_text = Parser::new().parse(&text);

        assert_eq!(1, parsed_text.len());

        match parsed_text.get(0).unwrap() {
            Token::Text { content } => {
                assert_eq!("Demo random something", *content)
            }
            _ => panic!("test_parsing_only_text"),
        }
    }

    #[test]
    fn test_parsing_inline_empty_attributes_shortcode() {
        let token = Parser::new().parse(&"[test]".to_string());

        assert_eq!(1, token.len());

        match token.get(0).unwrap() {
            Token::InlineTag { tag } => {
                assert_eq!("test", tag)
            }
            _ => panic!("Test failed: test_parsing_inline_empty_attributes_shortcode"),
        }
    }
    #[test]
    fn test_parsing_inline_empty_attributes_shortcode_with_texts() {
        let tokens = Parser::new().parse(&"[test] hello".to_string());

        assert_eq!(2, tokens.len());

        for token in tokens {
            match token {
                Token::Text { content } => {
                    assert_eq!(" hello", content)
                }
                Token::InlineTag { tag } => {
                    assert_eq!("test", tag)
                }
                _ => {
                    panic!(
                        "Test failed: test_parsing_inline_empty_attributes_shortcode_with_texts"
                    );
                }
            }
        }
    }

    #[test]
    fn test_parsing_inline_tag_with_attribute() {
        let tokens = Parser::new().parse(&"[test need-color]".to_string());

        assert_eq!(1, tokens.len());

        for token in tokens {
            match token {
                Token::AttributeTag { tag, attrs } => {
                    assert_eq!("test", tag);
                    assert_eq!(1, attrs.len());

                    assert_eq!(true, attrs.get("need-color").unwrap().is_none());
                }
                _ => {
                    panic!("Test failed: test_parsing_inline_tag_with_attribute");
                }
            }
        }
    }

    #[test]
    fn test_parsing_inline_multiple_attributes() {
        let tokens = Parser::new().parse(&"[test a b c='value']".to_string());

        assert_eq!(1, tokens.len());

        for token in tokens {
            match token {
                Token::AttributeTag { tag, attrs } => {
                    assert_eq!("test", tag);
                    assert_eq!(3, attrs.len());

                    assert_eq!(true, attrs.get("a").unwrap().is_none());
                    assert_eq!(true, attrs.get("b").unwrap().is_none());
                    assert_eq!(true, attrs.get("c").unwrap().is_some());
                }
                _ => {
                    panic!("Test failed: test_parsing_inline_multiple_attributes");
                }
            }
        }
    }

    #[test]
    fn test_parsing_inline_attribute() {
        let tokens = Parser::new().parse(&"[test color=\"red\"]".to_string());

        assert_eq!(1, tokens.len());

        for token in tokens {
            match token {
                Token::AttributeTag { tag, attrs } => {
                    assert_eq!("test", tag);
                    assert_eq!(1, attrs.len());

                    assert_eq!("red", attrs.get("color").unwrap().clone().unwrap());
                }
                _ => {
                    panic!("Test failed: test_parsing_inline_tag_with_attribute");
                }
            }
        }
    }

    #[test]
    fn test_parsing_inline_tag_attribute_with_single_quote() {
        let tokens = Parser::new().parse(&"[test color='single quote']".to_string());

        assert_eq!(1, tokens.len());

        for token in tokens {
            match token {
                Token::AttributeTag { tag, attrs } => {
                    assert_eq!("test", tag);
                    assert_eq!(1, attrs.len());

                    assert_eq!("single quote", attrs.get("color").unwrap().clone().unwrap());
                }
                _ => {
                    panic!("Test failed: test_parsing_inline_tag_attribute_with_single_quote");
                }
            }
        }
    }

    #[test]
    fn test_parsing_nested_attribute_tags() {
        let tokens = Parser::new().parse(&"[style color=\"red\"] [hello] [/style]".to_string());

        assert_eq!(1, tokens.len());

        for token in tokens {
            match token {
                Token::NestedAttributeTag {
                    tag,
                    attrs,
                    children,
                } => {
                    assert_eq!("style", tag);
                    assert_eq!(1, attrs.len());

                    assert_eq!("red", attrs.get("color").unwrap().clone().unwrap());

                    assert_eq!(3, children.len());
                }
                _ => {
                    panic!("Test failed: test_parsing_en_closing_tag");
                }
            }
        }
    }

    #[test]
    fn test_parsing_multiple_nested_attribute_tags() {
        let tokens = Parser::new().parse(&"[style color=\"red\"][row][text][/row][/style]".to_string());

        assert_eq!(1, tokens.len());

        for token in tokens {
            match token {
                Token::NestedAttributeTag {
                    tag,
                    attrs,
                    children,
                } => {
                    assert_eq!("style", tag);
                    assert_eq!(1, attrs.len());

                    assert_eq!("red", attrs.get("color").unwrap().clone().unwrap());

                    assert_eq!(1, children.len());

                    for child in children {
                        match child {
                            Token::NestedInlineTag { tag, children } => {
                                assert_eq!("row", tag);
                                assert_eq!(1, children.len());
                            }
                            _ => panic!(
                                "Test failed: test_parsing_multiple_nested_attribute_tags > child"
                            ),
                        }
                    }
                }
                _ => {
                    panic!("Test failed: test_parsing_multiple_nested_attribute_tags");
                }
            }
        }
    }
}
