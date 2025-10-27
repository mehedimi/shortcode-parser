use crate::token::Token;

#[derive(Debug)]
pub enum Code<'a> {
    Inline(&'a Token<'a>),
    Nested(&'a Token<'a>, Vec<Code<'a>>),
}

impl<'a> Code<'a> {
    pub fn tag_name(&self) -> Option<&str> {
        match self {
            Code::Inline(token) | Code::Nested(token, ..) => token.tag_name(),
        }
    }

    pub fn render_raw(&self) -> String {
        match self {
            Code::Inline(token) => token.render_raw(),
            Code::Nested(token, children) => {
                let code = Code::Inline(&Token::CloseTag(token.tag_name().unwrap()));
                format!(
                    "{}{}{}",
                    token.render_raw(),
                    children
                        .iter()
                        .map(|code| code.render_raw())
                        .collect::<Vec<_>>()
                        .join(""),
                    code.render_raw()
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_raw_self_close_shortcode() {
        let code = Code::Inline(&Token::SelfClose("test"));

        assert_eq!(code.render_raw(), "[test]");
    }

    #[test]
    fn test_render_raw_self_close_shortcode_with_attr() {
        let code = Code::Inline(&Token::SelfCloseAttr("test", vec![("id", Some("123"))]));
        assert_eq!(code.render_raw(), "[test id=\"123\"]");
    }

    #[test]
    fn test_render_raw_self_close_shortcode_with_attrs() {
        let code = Code::Inline(&Token::SelfCloseAttr(
            "test",
            vec![("id", Some("123")), ("name", Some("hello world"))],
        ));
        assert_eq!(code.render_raw(), "[test id=\"123\" name=\"hello world\"]");
    }

    #[test]
    fn test_render_raw_enclosed_shortcode() {
        let code = Code::Inline(&Token::CloseTag("test"));
        assert_eq!(code.render_raw(), "[/test]");
    }

    #[test]
    fn test_render_raw_text() {
        let code = Code::Inline(&Token::Text("test"));

        assert_eq!(code.render_raw(), "test");
    }

    #[test]
    fn test_render_raw_nested_shortcode() {
        let code = Code::Nested(
            &Token::SelfClose("test"),
            vec![Code::Inline(&Token::Text("hello world"))],
        );

        assert_eq!(code.render_raw(), "[test]hello world[/test]");
    }

    #[test]
    fn test_render_raw_nested_shortcode_with_attr() {
        let code = Code::Nested(
            &Token::SelfCloseAttr("test", vec![("id", Some("123"))]),
            vec![Code::Inline(&Token::Text("hello world"))],
        );
        assert_eq!(code.render_raw(), "[test id=\"123\"]hello world[/test]");
    }

    #[test]
    fn test_render_raw_nested_shortcode_with_attrs() {
        let code = Code::Nested(
            &Token::SelfCloseAttr(
                "test",
                vec![("id", Some("123")), ("name", Some("hello world"))],
            ),
            vec![Code::Inline(&Token::Text("hello world"))],
        );
        assert_eq!(
            code.render_raw(),
            "[test id=\"123\" name=\"hello world\"]hello world[/test]"
        );
    }

    #[test]
    fn test_render_raw_nested_shortcode_with_attrs_and_text() {
        let code = Code::Nested(
            &Token::SelfCloseAttr(
                "test",
                vec![("id", Some("123")), ("name", Some("hello world"))],
            ),
            vec![
                Code::Inline(&Token::Text("hello world")),
                Code::Inline(&Token::Text("hello world")),
            ],
        );
        assert_eq!(
            code.render_raw(),
            "[test id=\"123\" name=\"hello world\"]hello worldhello world[/test]"
        );
    }

    #[test]
    fn test_render_raw_nested_shortcode_with_attrs_and_text_and_nested_shortcode() {
        let code = Code::Nested(
            &Token::SelfCloseAttr(
                "test",
                vec![("id", Some("123")), ("name", Some("hello world"))],
            ),
            vec![
                Code::Inline(&Token::Text("hello world")),
                Code::Inline(&Token::Text("hello world")),
                Code::Nested(
                    &Token::SelfClose("test"),
                    vec![Code::Inline(&Token::Text("hello world"))],
                ),
            ],
        );
        assert_eq!(code.render_raw(), "[test id=\"123\" name=\"hello world\"]hello worldhello world[test]hello world[/test][/test]");
    }
}
