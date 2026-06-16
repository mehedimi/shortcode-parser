use crate::code::Code;
use crate::shortcode::ShortcodeFn;
use crate::token::Token;

pub struct Renderer<'a> {
    items: Vec<Code<'a>>,
}

impl<'a> Renderer<'a> {
    pub fn new(tokens: &'a [Token<'a>]) -> Self {
        let mut items: Vec<Code<'a>> = vec![];

        for token in tokens {
            match token {
                 Token::CloseTag(name) => {
                    // Collect items popped while searching for matching opener.
                    let mut popped = vec![];
                    let mut matched: Option<Code<'a>> = None;

                    while let Some(code) = items.pop() {
                        if let Some(tag_name) = code.tag_name() {
                            if *name == tag_name {
                                matched = Some(code);
                                break;
                            }
                        }
                        popped.push(code);
                    }

                    if let Some(matched_code) = matched {
                        // Found matching opener — build nested node.
                        let mut children = vec![];
                        for code in popped.into_iter().rev() {
                            children.push(code);
                        }
                        // Extract the token from the matched code.
                        match matched_code {
                            Code::Nested(token, _) | Code::Inline(token) => {
                                items.push(Code::Nested(token, children));
                            }
                        }
                    } else {
                        // No matching opener — restore stack and render close tag raw.
                        for code in popped.into_iter().rev() {
                            items.push(code);
                        }
                        items.push(Code::Inline(token));
                    }
                }
                _ => items.push(Code::Inline(token)),
            }
        }

        Self { items }
    }

    pub fn render(&self, codes: &[(&str, ShortcodeFn)]) -> String {
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

        assert_eq!(renderer.render(&[]), "Hello world");
    }

    #[test]
    fn test_render_unmatched_close_tag() {
        let tokens = vec![
            Token::Text("before "),
            Token::CloseTag("unknown"),
            Token::Text(" after"),
        ];

        let renderer = Renderer::new(&tokens);
        assert_eq!(renderer.render(&[]), "before [/unknown] after");
    }

    #[test]
    fn test_render_unmatched_close_tag_with_handler() {
        let tokens = vec![
            Token::SelfClose("foo"),
            Token::Text(" "),
            Token::CloseTag("unknown"),
        ];

        let codes: &[(&str, ShortcodeFn)] = &[("foo", |_, _| "<foo/>".to_string())];

        let renderer = Renderer::new(&tokens);
        assert_eq!(renderer.render(codes), "<foo/> [/unknown]");
    }

    #[test]
    fn test_render_nested_unmatched_inner() {
        let tokens = vec![
            Token::SelfClose("outer"),
            Token::CloseTag("inner"),
        ];

        let codes: &[(&str, ShortcodeFn)] = &[("outer", |_, _| "<outer/>".to_string())];

        let renderer = Renderer::new(&tokens);
        assert_eq!(renderer.render(codes), "<outer/>[/inner]");
    }
}
