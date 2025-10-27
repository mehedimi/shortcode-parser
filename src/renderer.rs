use crate::code::Code;
use crate::shortcode::ShortcodeFn;
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
        self.items.iter().map(|code| code.render(codes)).collect()
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
