use crate::tokenizer::token::Token;
use std::collections::HashMap;
use std::ops::Deref;

mod render;
mod token;

enum ParseState {
    Text,
    TagStart,
    AttrKey,
    AttrValueStart,
    AttrValue,
}

pub fn parse(content: String) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut parse_state = ParseState::Text;

    let mut temp_text = String::new();
    let mut temp_tag = String::new();
    let mut is_tag_end = false;
    let mut attrs: HashMap<String, String> = HashMap::new();

    let mut temp_attr_key = String::new();
    let mut temp_attr_value = String::new();

    for char in content.chars() {
        match parse_state {
            ParseState::Text => {
                if char == '[' {
                    if !temp_text.is_empty() {
                        tokens.push(Token::Text {
                            content: temp_text.clone(),
                        });
                        temp_text.clear();
                    }

                    parse_state = ParseState::TagStart;
                } else {
                    temp_text.push(char);
                }
            }
            ParseState::TagStart => {
                if char == ' ' {
                    parse_state = ParseState::AttrKey;
                    // If this is end tag
                } else if char == '/' {
                    is_tag_end = true;
                } else if char == ']' {
                    if is_tag_end {
                        // Finding starting tag
                        let start_index = tokens.iter().position(|token| {
                            return match token {
                                Token::InlineTag { tag } => tag.deref() == temp_tag,
                                Token::AttributeTag { tag, .. } => tag.deref() == temp_tag,
                                _ => false,
                            };
                        });

                        match start_index {
                            Some(index) => {
                                let start_tag = tokens.get(index).unwrap();
                                let children = &tokens[(index + 1)..];
                                match start_tag {
                                    Token::InlineTag { tag } => {
                                        tokens.splice(
                                            index..,
                                            Vec::from([Token::NestedInlineTag {
                                                tag: tag.clone(),
                                                children: Vec::from(children),
                                            }]),
                                        );
                                    }
                                    Token::AttributeTag { tag, attrs } => {
                                        tokens.splice(
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
                            None => tokens.push({
                                Token::Text {
                                    content: "[".to_owned() + &temp_tag.clone(),
                                }
                            }),
                        }
                        is_tag_end = false;
                    } else {
                        tokens.push(Token::InlineTag {
                            tag: temp_tag.clone(),
                        });
                    }
                    temp_tag.clear();
                    parse_state = ParseState::Text;
                } else {
                    temp_tag.push(char);
                }
            }
            ParseState::AttrKey => {
                if char == '=' {
                    parse_state = ParseState::AttrValueStart;
                } else if char == ']' {
                    tokens.push(Token::AttributeTag {
                        tag: temp_tag.clone(),
                        attrs: attrs.clone(),
                    });
                    temp_tag.clear();
                    attrs.clear();
                    parse_state = ParseState::Text;
                } else {
                    temp_attr_key.push(char);
                }
            }
            ParseState::AttrValueStart => {
                if char == '"' {
                    parse_state = ParseState::AttrValue;
                }
            }
            ParseState::AttrValue => {
                if char == '"' {
                    attrs.insert(
                        temp_attr_key.trim().clone().to_string(),
                        temp_attr_value.clone(),
                    );
                    temp_attr_key.clear();
                    temp_attr_value.clear();
                    parse_state = ParseState::AttrKey;
                } else {
                    temp_attr_value.push(char);
                }
            }
        }
    }

    if !temp_text.is_empty() {
        tokens.push(Token::Text {
            content: temp_text.clone(),
        });
        temp_text.clear();
    }

    return tokens;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parsing_only_text() {
        let text = String::from("Demo random something");

        let parsed_text = parse(text);

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
        let token = parse("[test]".to_string());

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
        let tokens = parse("[test] hello".to_string());

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
        let tokens = parse("[test color=\"red\"]".to_string());

        assert_eq!(1, tokens.len());

        for token in tokens {
            match token {
                Token::AttributeTag { tag, attrs } => {
                    assert_eq!("test", tag);
                    assert_eq!(1, attrs.len());

                    assert_eq!("red", attrs.get("color").unwrap());
                }
                _ => {
                    panic!("Test failed: test_parsing_inline_tag_with_attribute");
                }
            }
        }
    }

    #[test]
    fn test_parsing_nested_attribute_tags() {
        let tokens = parse("[style color=\"red\"] [hello] [/style]".to_string());

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

                    assert_eq!("red", attrs.get("color").unwrap());

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
        let tokens = parse("[style color=\"red\"][row][text][/row][/style]".to_string());

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

                    assert_eq!("red", attrs.get("color").unwrap());

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
