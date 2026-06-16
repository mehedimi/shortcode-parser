use crate::attrs::ShortcodeAttrs;
use crate::shortcode::ShortcodeFn;
use crate::token::Token;

#[derive(Debug)]
pub enum Code<'a> {
    Inline(&'a Token<'a>),
    Nested(&'a Token<'a>, Vec<Code<'a>>),
}

impl<'a> Code<'a> {
    /// Returns the tag name of this code node, if it represents a shortcode tag.
    pub(crate) fn tag_name(&self) -> Option<&str> {
        match self {
            Code::Inline(token) | Code::Nested(token, _) => token.tag_name(),
        }
    }

    fn lookup_handler<'b>(
        shortcodes: &'b [(&str, ShortcodeFn)],
        code_name: &str,
    ) -> Option<&'b ShortcodeFn> {
        shortcodes
            .iter()
            .find(|(name, _)| *name == code_name)
            .map(|(_, f)| f)
    }

    pub fn render(&self, shortcodes: &[(&str, ShortcodeFn)]) -> String {
        match self {
            Code::Inline(token) => {
                if let Some(code_name) = token.tag_name() {
                    if let Some(code_fn) = Self::lookup_handler(shortcodes, code_name) {
                        code_fn(None, ShortcodeAttrs::new(token.attrs_slice()))
                    } else {
                        token.render_raw().into_owned()
                    }
                } else {
                    token.render_raw().into_owned()
                }
            }
            Code::Nested(token, children) => {
                if let Some(code_name) = token.tag_name() {
                    let rendered_children = children
                        .iter()
                        .map(|code| code.render(shortcodes))
                        .collect::<String>();

                    if let Some(code_fn) = Self::lookup_handler(shortcodes, code_name) {
                        code_fn(
                            Some(rendered_children.as_str()),
                            ShortcodeAttrs::new(token.attrs_slice()),
                        )
                    } else {
                        format!(
                            "{}{}{}",
                            token.render_raw(),
                            rendered_children,
                            Token::CloseTag(code_name).render_raw(),
                        )
                    }
                } else {
                    token.render_raw().into_owned()
                }
            }
        }
    }
}
