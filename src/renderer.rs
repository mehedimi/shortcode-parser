use crate::code::Code;
use crate::shortcode::{Shortcode, ShortcodeFn};
use crate::token::Token;
use std::collections::HashMap;

pub struct Renderer<'a> {
    items: Vec<Code<'a>>,
}

impl<'a> Renderer<'a> {
    pub fn new(tokens: &'a Vec<Token<'a>>) -> Self {
        let mut items = vec![];

        for token in tokens {
            match token {
                Token::CloseTag(name) => {
                    let mut children = vec![];

                    while let Some(code) = items.pop() {
                        match code {
                            Code::Nested(token, ..) | Code::Inline(token) => {
                                if let Some(tag_name) = token.tag_name() {
                                    if *name == tag_name {
                                        children.reverse();

                                        items.push(Code::Nested(token, children));
                                        break;
                                    }
                                }

                                children.push(code);
                            }
                        }
                    }
                }
                _ => items.push(Code::Inline(token)),
            }
        }

        Self { items }
    }

    pub fn render(&self, codes: &HashMap<&str, ShortcodeFn>) -> String {
        self.items
            .iter()
            .map(|code| {
                if let Some(tag_name) = code.tag_name() {
                    if let Some(func) = codes.get(tag_name) {
                        // func(code.render_raw().as_str(), HashMap::new())
                        "".to_string()
                    } else {
                        return code.render_raw();
                    }
                } else {
                    return code.render_raw();
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_empty_content() {
        let tokens = vec![Token::Text("Hello world")];

        let renderer = Renderer::new(&tokens);

        assert_eq!(renderer.render(&HashMap::new()), "Hello world");
    }
}
