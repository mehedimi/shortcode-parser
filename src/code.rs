use crate::shortcode::ShortcodeFn;
use crate::token::Token;
use std::collections::HashMap;

#[derive(Debug)]
pub enum Code<'a> {
    Inline(&'a Token<'a>),
    Nested(&'a Token<'a>, Vec<Code<'a>>),
}

impl<'a> Code<'a> {


    pub fn render_vec(
        &self,
        shortcodes: &HashMap<&str, ShortcodeFn>,
        children: &Vec<Code>,
    ) -> String {
        children
            .iter()
            .map(|code| code.render(shortcodes))
            .collect()
    }

    pub fn render(&self, shortcodes: &HashMap<&str, ShortcodeFn>) -> String {
        match self {
            Code::Inline(token) => {
                if let Some(code_name) = token.tag_name() {
                    if let Some(code_fn) = shortcodes.get(code_name) {
                        code_fn(None, token.get_attr_map())
                    } else {
                        token.render_raw()
                    }
                } else {
                    token.render_raw()
                }
            }
            Code::Nested(token, children) => {
                if let Some(code_name) = token.tag_name() {
                    if let Some(code_fn) = shortcodes.get(code_name) {
                        code_fn(Some(self.render_vec(shortcodes, children).as_str()), token.get_attr_map())
                    } else {
                        format!(
                            "{}{}{}",
                            token.render_raw(),
                            self.render_vec(shortcodes, children),
                            Token::CloseTag(code_name).render_raw()
                        )
                    }
                } else {
                    token.render_raw()
                }
            }
        }
    }
}
