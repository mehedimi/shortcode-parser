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

#[cfg(test)]
mod tests {
    use super::*;

    // Helper fn items for tests (ShortcodeFn is a function pointer, not a closure).
    fn handler_empty(_: Option<&str>, _: ShortcodeAttrs) -> String {
        String::new()
    }

    fn handler_inner(_: Option<&str>, _: ShortcodeAttrs) -> String {
        "<inner/>".to_string()
    }

    fn handler_outer(content: Option<&str>, _: ShortcodeAttrs) -> String {
        format!("<outer>{}</outer>", content.unwrap_or(""))
    }

    fn handler_a(content: Option<&str>, _: ShortcodeAttrs) -> String {
        format!("<a>{}</a>", content.unwrap_or(""))
    }

    fn handler_b(content: Option<&str>, _: ShortcodeAttrs) -> String {
        format!("<b>{}</b>", content.unwrap_or(""))
    }

    fn handler_c(content: Option<&str>, _: ShortcodeAttrs) -> String {
        format!("<c>{}</c>", content.unwrap_or(""))
    }

    fn handler_tag(content: Option<&str>, _: ShortcodeAttrs) -> String {
        format!("<tag>{}</tag>", content.unwrap_or(""))
    }

    fn handler_foo(_: Option<&str>, _: ShortcodeAttrs) -> String {
        "BAR".to_string()
    }

    fn handler_foo_content(content: Option<&str>, _: ShortcodeAttrs) -> String {
        format!("FOO({})", content.unwrap_or(""))
    }

    fn handler_video_id(_: Option<&str>, attrs: ShortcodeAttrs) -> String {
        format!("<video id=\"{}\"/>", attrs.get("id").unwrap_or("none"))
    }

    fn handler_video_autoplay(_: Option<&str>, attrs: ShortcodeAttrs) -> String {
        if attrs.get("autoplay").is_some() {
            "<video autoplay/>".to_string()
        } else {
            "<video/>".to_string()
        }
    }

    fn handler_video_width(_: Option<&str>, attrs: ShortcodeAttrs) -> String {
        format!("<video width=\"{}\"/>", attrs.get("width").unwrap_or("auto"))
    }

    fn handler_video_url(_: Option<&str>, attrs: ShortcodeAttrs) -> String {
        format!("<video url=\"{}\"/>", attrs.get("url").unwrap_or(""))
    }

    fn handler_video_src(_: Option<&str>, attrs: ShortcodeAttrs) -> String {
        format!("<video src=\"{}\"/>", attrs.get("src").unwrap_or(""))
    }

    fn handler_video_html(_: Option<&str>, attrs: ShortcodeAttrs) -> String {
        format!("<video>{}</video>", attrs.get("html").unwrap_or(""))
    }

    fn handler_greet(_: Option<&str>, attrs: ShortcodeAttrs) -> String {
        format!("<greet>Hello {}!</greet>", attrs.get("name").unwrap_or("world"))
    }

    fn handler_emoji(content: Option<&str>, _: ShortcodeAttrs) -> String {
        format!("<emoji>{}</emoji>", content.unwrap_or(""))
    }

    fn handler_x(content: Option<&str>, _: ShortcodeAttrs) -> String {
        format!("<x>{}</x>", content.unwrap_or(""))
    }

    fn handler_y(content: Option<&str>, _: ShortcodeAttrs) -> String {
        format!("<y>{}</y>", content.unwrap_or(""))
    }

    fn handler_z(_: Option<&str>, _: ShortcodeAttrs) -> String {
        "Z".to_string()
    }

    fn handler_outer_inner(content: Option<&str>, _: ShortcodeAttrs) -> String {
        let c = content.unwrap_or("");
        format!("outer({})-inner({})", c, "handled")
    }

    fn handler_inner_handled(_: Option<&str>, _: ShortcodeAttrs) -> String {
        "inner_handled".to_string()
    }

    fn handler_x_nested(content: Option<&str>, _: ShortcodeAttrs) -> String {
        format!("x({})", content.unwrap_or(""))
    }

    fn handler_y_nested(content: Option<&str>, _: ShortcodeAttrs) -> String {
        format!("y({})", content.unwrap_or(""))
    }

    fn handler_len(_: Option<&str>, attrs: ShortcodeAttrs) -> String {
        format!("len={}", attrs.get("id").map(|s| s.len()).unwrap_or(0))
    }

    fn handler_val(_: Option<&str>, attrs: ShortcodeAttrs) -> String {
        format!("VAL={}", attrs.get("").unwrap_or("none"))
    }

    fn handler_flags(_: Option<&str>, attrs: ShortcodeAttrs) -> String {
        let mut flags = vec![];
        for (k, _) in attrs.iter() {
            flags.push(k);
        }
        format!("flags={}", flags.iter().map(|s| **s).collect::<Vec<_>>().join(","))
    }

    fn handler_id(_: Option<&str>, attrs: ShortcodeAttrs) -> String {
        format!("ID={}", attrs.get("id").unwrap_or("none"))
    }

    fn handler_video_id_autoplay(_: Option<&str>, attrs: ShortcodeAttrs) -> String {
        let id = attrs.get("id").unwrap_or("unknown");
        let autoplay = if attrs.get("autoplay").is_some() { " autoplay" } else { "" };
        format!("<video id=\"{}\"{} />", id, autoplay)
    }

    fn handler_a_interleaved(content: Option<&str>, _: ShortcodeAttrs) -> String {
        format!("<a>{}</a>", content.unwrap_or(""))
    }

    fn handler_b_interleaved(content: Option<&str>, _: ShortcodeAttrs) -> String {
        format!("<b>{}</b>", content.unwrap_or(""))
    }

    fn handler_c_interleaved(content: Option<&str>, _: ShortcodeAttrs) -> String {
        format!("<c>{}</c>", content.unwrap_or(""))
    }

    #[test]
    fn test_tag_name_inline_self_close() {
        let token = Token::SelfClose("video");
        let code = Code::Inline(&token);
        assert_eq!(code.tag_name(), Some("video"));
    }

    #[test]
    fn test_tag_name_inline_self_close_attr() {
        let token = Token::SelfCloseAttr("video", vec![("id", Some("123"))]);
        let code = Code::Inline(&token);
        assert_eq!(code.tag_name(), Some("video"));
    }

    #[test]
    fn test_tag_name_inline_close_tag() {
        let token = Token::CloseTag("video");
        let code = Code::Inline(&token);
        assert_eq!(code.tag_name(), Some("video"));
    }

    #[test]
    fn test_tag_name_inline_text() {
        let token = Token::Text("hello");
        let code = Code::Inline(&token);
        assert_eq!(code.tag_name(), None);
    }

    #[test]
    fn test_tag_name_nested_self_close() {
        let token = Token::SelfClose("outer");
        let code = Code::Nested(&token, vec![]);
        assert_eq!(code.tag_name(), Some("outer"));
    }

    #[test]
    fn test_tag_name_nested_self_close_attr() {
        let token = Token::SelfCloseAttr("outer", vec![("x", Some("1"))]);
        let code = Code::Nested(&token, vec![]);
        assert_eq!(code.tag_name(), Some("outer"));
    }

    #[test]
    fn test_tag_name_nested_close_tag() {
        let token = Token::CloseTag("outer");
        let code = Code::Nested(&token, vec![]);
        assert_eq!(code.tag_name(), Some("outer"));
    }

    #[test]
    fn test_tag_name_nested_text() {
        let token = Token::Text("hello");
        let code = Code::Nested(&token, vec![]);
        assert_eq!(code.tag_name(), None);
    }

    #[test]
    fn test_render_inline_unknown_handler() {
        let token = Token::SelfClose("unknown");
        let code = Code::Inline(&token);
        assert_eq!(code.render(&[]), "[unknown]");
    }

    #[test]
    fn test_render_inline_known_handler() {
        let token = Token::SelfClose("foo");
        let code = Code::Inline(&token);
        let handlers: &[(&str, ShortcodeFn)] = &[("foo", handler_foo)];
        assert_eq!(code.render(handlers), "BAR");
    }

    #[test]
    fn test_render_inline_text() {
        let token = Token::Text("hello world");
        let code = Code::Inline(&token);
        assert_eq!(code.render(&[]), "hello world");
    }

    #[test]
    fn test_render_inline_close_tag_no_handler() {
        let token = Token::CloseTag("foo");
        let code = Code::Inline(&token);
        assert_eq!(code.render(&[]), "[/foo]");
    }

    #[test]
    fn test_render_inline_self_close_attr_no_handler() {
        let token = Token::SelfCloseAttr("video", vec![("id", Some("123"))]);
        let code = Code::Inline(&token);
        assert_eq!(code.render(&[]), r#"[video id="123"]"#);
    }

    #[test]
    fn test_render_inline_self_close_flag_attr_no_handler() {
        let token = Token::SelfCloseAttr("video", vec![("autoplay", None)]);
        let code = Code::Inline(&token);
        assert_eq!(code.render(&[]), "[video autoplay]");
    }

    #[test]
    fn test_render_inline_unknown_handler_with_attrs() {
        let token = Token::SelfCloseAttr("unknown", vec![("id", Some("123"))]);
        let code = Code::Inline(&token);
        assert_eq!(code.render(&[]), r#"[unknown id="123"]"#);
    }

    #[test]
    fn test_render_nested_unknown_handler() {
        let token = Token::SelfClose("unknown");
        let children = vec![Code::Inline(&Token::Text(" content "))];
        let code = Code::Nested(&token, children);
        assert_eq!(code.render(&[]), "[unknown] content [/unknown]");
    }

    #[test]
    fn test_render_nested_known_handler_no_content() {
        let token = Token::SelfClose("foo");
        let children: Vec<Code> = vec![];
        let code = Code::Nested(&token, children);
        let handlers: &[(&str, ShortcodeFn)] = &[("foo", handler_foo_content)];
        assert_eq!(code.render(handlers), "FOO()");
    }

    #[test]
    fn test_render_nested_known_handler_with_content() {
        let token = Token::SelfClose("outer");
        let inner_token = Token::SelfClose("inner");
        let inner_code = Code::Inline(&inner_token);
        let children = vec![inner_code];
        let code = Code::Nested(&token, children);
        let handlers: &[(&str, ShortcodeFn)] = &[("outer", handler_outer), ("inner", handler_inner)];
        assert_eq!(code.render(handlers), "<outer><inner/></outer>");
    }

    #[test]
    fn test_render_nested_known_handler_with_text_content() {
        let token = Token::SelfClose("outer");
        let text_token = Token::Text("hello");
        let text_code = Code::Inline(&text_token);
        let children = vec![text_code];
        let code = Code::Nested(&token, children);
        let handlers: &[(&str, ShortcodeFn)] = &[("outer", handler_outer)];
        assert_eq!(code.render(handlers), "<outer>hello</outer>");
    }

    #[test]
    fn test_render_nested_unknown_handler_with_text() {
        let token = Token::SelfClose("outer");
        let text_token = Token::Text("hello");
        let text_code = Code::Inline(&text_token);
        let children = vec![text_code];
        let code = Code::Nested(&token, children);
        assert_eq!(code.render(&[]), "[outer]hello[/outer]");
    }

    #[test]
    fn test_render_nested_deeply() {
        let token_c = Token::SelfClose("c");
        let code_c = Code::Inline(&token_c);
        let token_b = Token::SelfClose("b");
        let code_b = Code::Nested(&token_b, vec![code_c]);
        let token_a = Token::SelfClose("a");
        let code_a = Code::Nested(&token_a, vec![code_b]);
        let handlers: &[(&str, ShortcodeFn)] = &[
            ("a", handler_a),
            ("b", handler_b),
            ("c", handler_c),
        ];
        assert_eq!(code_a.render(handlers), "<a><b><c></c></b></a>");
    }

    #[test]
    fn test_render_nested_deeply_with_content() {
        let text_token = Token::Text(" hello ");
        let text_code = Code::Inline(&text_token);
        let token_c = Token::SelfClose("c");
        let code_c = Code::Nested(&token_c, vec![text_code]);
        let token_b = Token::SelfClose("b");
        let code_b = Code::Nested(&token_b, vec![code_c]);
        let token_a = Token::SelfClose("a");
        let code_a = Code::Nested(&token_a, vec![code_b]);
        let handlers: &[(&str, ShortcodeFn)] = &[
            ("a", handler_a),
            ("b", handler_b),
            ("c", handler_c),
        ];
        assert_eq!(code_a.render(handlers), "<a><b><c> hello </c></b></a>");
    }

    #[test]
    fn test_render_nested_multiple_children() {
        let text1 = Code::Inline(&Token::Text(" a "));
        let text2 = Code::Inline(&Token::Text(" b "));
        let token = Token::SelfClose("outer");
        let code = Code::Nested(&token, vec![text1, text2]);
        let handlers: &[(&str, ShortcodeFn)] = &[("outer", handler_outer)];
        assert_eq!(code.render(handlers), "<outer> a  b </outer>");
    }

    #[test]
    fn test_render_nested_handler_with_attrs() {
        let token = Token::SelfCloseAttr("video", vec![("id", Some("123"))]);
        let children: Vec<Code> = vec![];
        let code = Code::Nested(&token, children);
        let handlers: &[(&str, ShortcodeFn)] = &[("video", handler_video_id)];
        assert_eq!(code.render(handlers), r#"<video id="123"/>"#);
    }

    #[test]
    fn test_render_nested_handler_with_flag_attr() {
        let token = Token::SelfCloseAttr("video", vec![("autoplay", Some(""))]);
        let children: Vec<Code> = vec![];
        let code = Code::Nested(&token, children);
        let handlers: &[(&str, ShortcodeFn)] = &[("video", handler_video_autoplay)];
        assert_eq!(code.render(handlers), "<video autoplay/>");
    }

    #[test]
    fn test_render_nested_handler_with_missing_attr() {
        let token = Token::SelfCloseAttr("video", vec![("id", Some("123"))]);
        let children: Vec<Code> = vec![];
        let code = Code::Nested(&token, children);
        let handlers: &[(&str, ShortcodeFn)] = &[("video", handler_video_width)];
        assert_eq!(code.render(handlers), r#"<video width="auto"/>"#);
    }

    #[test]
    fn test_render_nested_with_nested_unknown() {
        let inner_token = Token::SelfClose("inner");
        let inner_code = Code::Inline(&inner_token);
        let token = Token::SelfClose("outer");
        let code = Code::Nested(&token, vec![inner_code]);
        let handlers: &[(&str, ShortcodeFn)] = &[("outer", handler_outer)];
        assert_eq!(code.render(handlers), "<outer>[inner]</outer>");
    }

    #[test]
    fn test_render_nested_with_handler_and_unknown_child() {
        let inner_token = Token::SelfClose("inner");
        let inner_code = Code::Inline(&inner_token);
        let text_token = Token::Text(" hello ");
        let text_code = Code::Inline(&text_token);
        let token = Token::SelfClose("outer");
        let code = Code::Nested(&token, vec![inner_code, text_code]);
        let handlers: &[(&str, ShortcodeFn)] = &[("outer", handler_outer)];
        assert_eq!(code.render(handlers), "<outer>[inner] hello </outer>");
    }

    #[test]
    fn test_render_nested_empty_name() {
        let token = Token::SelfClose("");
        let children: Vec<Code> = vec![];
        let code = Code::Nested(&token, children);
        let handlers: &[(&str, ShortcodeFn)] = &[("", handler_foo)];
        assert_eq!(code.render(handlers), "BAR");
    }

    #[test]
    fn test_render_nested_empty_name_no_handler() {
        let token = Token::SelfClose("");
        let code = Code::Inline(&token);
        assert_eq!(code.render(&[]), "[]");
    }

    #[test]
    fn test_render_nested_space_name() {
        let token = Token::SelfClose(" ");
        let children: Vec<Code> = vec![];
        let code = Code::Nested(&token, children);
        let handlers: &[(&str, ShortcodeFn)] = &[(" ", handler_foo)];
        assert_eq!(code.render(handlers), "BAR");
    }

    #[test]
    fn test_render_nested_space_name_no_handler() {
        let token = Token::SelfClose(" ");
        let code = Code::Inline(&token);
        assert_eq!(code.render(&[]), "[ ]");
    }

    #[test]
    fn test_render_nested_unicode_attr() {
        let token = Token::SelfCloseAttr("greet", vec![("name", Some("世界"))]);
        let children: Vec<Code> = vec![];
        let code = Code::Nested(&token, children);
        let handlers: &[(&str, ShortcodeFn)] = &[("greet", handler_greet)];
        assert_eq!(code.render(handlers), "<greet>Hello 世界!</greet>");
    }

    #[test]
    fn test_render_nested_unicode_content() {
        let text_token = Token::Text("世界!");
        let text_code = Code::Inline(&text_token);
        let token = Token::SelfClose("emoji");
        let code = Code::Nested(&token, vec![text_code]);
        let handlers: &[(&str, ShortcodeFn)] = &[("emoji", handler_emoji)];
        assert_eq!(code.render(handlers), "<emoji>世界!</emoji>");
    }

    #[test]
    fn test_render_nested_equals_in_value() {
        let token = Token::SelfCloseAttr("video", vec![("url", Some("a=b&c=d"))]);
        let children: Vec<Code> = vec![];
        let code = Code::Nested(&token, children);
        let handlers: &[(&str, ShortcodeFn)] = &[("video", handler_video_url)];
        assert_eq!(code.render(handlers), r#"<video url="a=b&c=d"/>"#);
    }

    #[test]
    fn test_render_nested_duplicate_attrs() {
        let token = Token::SelfCloseAttr("video", vec![("id", Some("1")), ("id", Some("2"))]);
        let children: Vec<Code> = vec![];
        let code = Code::Nested(&token, children);
        let handlers: &[(&str, ShortcodeFn)] = &[("video", handler_video_id)];
        assert_eq!(code.render(handlers), r#"<video id="1"/>"#);
    }

    #[test]
    fn test_render_nested_html_in_attr() {
        let token = Token::SelfCloseAttr("video", vec![("html", Some("<div>"))]);
        let children: Vec<Code> = vec![];
        let code = Code::Nested(&token, children);
        let handlers: &[(&str, ShortcodeFn)] = &[("video", handler_video_html)];
        assert_eq!(code.render(handlers), "<video><div></video>");
    }

    #[test]
    fn test_render_nested_empty_attr_value() {
        let token = Token::SelfCloseAttr("video", vec![("id", Some(""))]);
        let children: Vec<Code> = vec![];
        let code = Code::Nested(&token, children);
        let handlers: &[(&str, ShortcodeFn)] = &[("video", handler_video_id)];
        assert_eq!(code.render(handlers), r#"<video id=""/>"#);
    }

    #[test]
    fn test_render_nested_newlines_in_content() {
        let text_token = Token::Text("\nhello\n");
        let text_code = Code::Inline(&text_token);
        let token = Token::SelfClose("tag");
        let code = Code::Nested(&token, vec![text_code]);
        let handlers: &[(&str, ShortcodeFn)] = &[("tag", handler_tag)];
        assert_eq!(code.render(handlers), "<tag>\nhello\n</tag>");
    }

    #[test]
    fn test_render_nested_tabs_in_content() {
        let text_token = Token::Text("\thello\t");
        let text_code = Code::Inline(&text_token);
        let token = Token::SelfClose("tag");
        let code = Code::Nested(&token, vec![text_code]);
        let handlers: &[(&str, ShortcodeFn)] = &[("tag", handler_tag)];
        assert_eq!(code.render(handlers), "<tag>\thello\t</tag>");
    }

    #[test]
    fn test_render_nested_whitespace_content() {
        let text_token = Token::Text("   ");
        let text_code = Code::Inline(&text_token);
        let token = Token::SelfClose("tag");
        let code = Code::Nested(&token, vec![text_code]);
        let handlers: &[(&str, ShortcodeFn)] = &[("tag", handler_tag)];
        assert_eq!(code.render(handlers), "<tag>   </tag>");
    }

    #[test]
    fn test_render_nested_bracket_in_text() {
        let text_token = Token::Text("a > b");
        let text_code = Code::Inline(&text_token);
        let token = Token::SelfClose("tag");
        let code = Code::Nested(&token, vec![text_code]);
        let handlers: &[(&str, ShortcodeFn)] = &[("tag", handler_tag)];
        assert_eq!(code.render(handlers), "<tag>a > b</tag>");
    }

    #[test]
    fn test_render_nested_close_tag_in_children() {
        let close_token = Token::CloseTag("inner");
        let close_code = Code::Inline(&close_token);
        let token = Token::SelfClose("outer");
        let code = Code::Nested(&token, vec![close_code]);
        let handlers: &[(&str, ShortcodeFn)] = &[("outer", handler_outer)];
        assert_eq!(code.render(handlers), "<outer>[/inner]</outer>");
    }

    #[test]
    fn test_render_nested_multiple_nested_children() {
        let inner1_token = Token::SelfClose("a");
        let inner1_code = Code::Inline(&inner1_token);
        let inner2_token = Token::SelfClose("b");
        let inner2_code = Code::Inline(&inner2_token);
        let token = Token::SelfClose("outer");
        let code = Code::Nested(&token, vec![inner1_code, inner2_code]);
        let handlers: &[(&str, ShortcodeFn)] = &[
            ("outer", handler_outer),
            ("a", handler_foo),
            ("b", handler_foo),
        ];
        assert_eq!(code.render(handlers), "<outer>BARBAR</outer>");
    }

    #[test]
    fn test_render_nested_deeply_with_multiple_children() {
        let text1 = Code::Inline(&Token::Text(" x "));
        let text2 = Code::Inline(&Token::Text(" y "));
        let inner_token = Token::SelfClose("b");
        let inner_code = Code::Nested(&inner_token, vec![text1, text2]);
        let text3 = Code::Inline(&Token::Text(" z "));
        let outer_token = Token::SelfClose("a");
        let outer_code = Code::Nested(&outer_token, vec![inner_code, text3]);
        let handlers: &[(&str, ShortcodeFn)] = &[
            ("a", handler_a),
            ("b", handler_b),
        ];
        assert_eq!(outer_code.render(handlers), "<a><b> x  y </b> z </a>");
    }

    #[test]
    fn test_render_nested_self_close_with_attr_no_handler() {
        let token = Token::SelfCloseAttr("video", vec![("id", Some("123")), ("autoplay", None)]);
        let code = Code::Inline(&token);
        assert_eq!(code.render(&[]), r#"[video id="123" autoplay]"#);
    }

    #[test]
    fn test_render_nested_self_close_with_attr_handler() {
        let token = Token::SelfCloseAttr("video", vec![("id", Some("123")), ("autoplay", Some(""))]);
        let children: Vec<Code> = vec![];
        let code = Code::Nested(&token, children);
        let handlers: &[(&str, ShortcodeFn)] = &[("video", handler_video_id_autoplay)];
        assert_eq!(code.render(handlers), r#"<video id="123" autoplay />"#);
    }

    #[test]
    fn test_render_nested_with_handler_and_unknown_nested() {
        let inner_token = Token::SelfClose("inner");
        let inner_code = Code::Inline(&inner_token);
        let text_token = Token::Text(" hello ");
        let text_code = Code::Inline(&text_token);
        let token = Token::SelfClose("outer");
        let code = Code::Nested(&token, vec![inner_code, text_code]);
        let handlers: &[(&str, ShortcodeFn)] = &[("outer", handler_outer)];
        assert_eq!(code.render(handlers), "<outer>[inner] hello </outer>");
    }

    #[test]
    fn test_render_nested_deeply_nested_unknown() {
        let innermost = Token::SelfClose("c");
        let innermost_code = Code::Inline(&innermost);
        let middle = Token::SelfClose("b");
        let middle_code = Code::Inline(&middle);
        let outer = Token::SelfClose("a");
        let outer_code = Code::Nested(&outer, vec![middle_code, innermost_code]);
        let handlers: &[(&str, ShortcodeFn)] = &[("a", handler_a)];
        assert_eq!(outer_code.render(handlers), "<a>[b][c]</a>");
    }

    #[test]
    fn test_render_inline_empty_name() {
        let token = Token::SelfClose("");
        let code = Code::Inline(&token);
        assert_eq!(code.tag_name(), Some(""));
    }

    #[test]
    fn test_render_inline_space_name() {
        let token = Token::SelfClose(" ");
        let code = Code::Inline(&token);
        assert_eq!(code.tag_name(), Some(" "));
    }

    #[test]
    fn test_render_inline_hyphen_name() {
        let token = Token::SelfClose("my-short-code");
        let code = Code::Inline(&token);
        assert_eq!(code.tag_name(), Some("my-short-code"));
    }

    #[test]
    fn test_render_inline_underscore_name() {
        let token = Token::SelfClose("my_shortcode");
        let code = Code::Inline(&token);
        assert_eq!(code.tag_name(), Some("my_shortcode"));
    }

    #[test]
    fn test_render_inline_number_name() {
        let token = Token::SelfClose("shortcode123");
        let code = Code::Inline(&token);
        assert_eq!(code.tag_name(), Some("shortcode123"));
    }

    #[test]
    fn test_render_nested_hyphen_name() {
        let token = Token::SelfClose("my-short-code");
        let children: Vec<Code> = vec![];
        let code = Code::Nested(&token, children);
        let handlers: &[(&str, ShortcodeFn)] = &[("my-short-code", handler_foo)];
        assert_eq!(code.render(handlers), "BAR");
    }

    #[test]
    fn test_render_nested_underscore_name() {
        let token = Token::SelfClose("my_shortcode");
        let children: Vec<Code> = vec![];
        let code = Code::Nested(&token, children);
        let handlers: &[(&str, ShortcodeFn)] = &[("my_shortcode", handler_foo)];
        assert_eq!(code.render(handlers), "BAR");
    }

    #[test]
    fn test_render_nested_number_name() {
        let token = Token::SelfClose("shortcode123");
        let children: Vec<Code> = vec![];
        let code = Code::Nested(&token, children);
        let handlers: &[(&str, ShortcodeFn)] = &[("shortcode123", handler_foo)];
        assert_eq!(code.render(handlers), "BAR");
    }

    #[test]
    fn test_render_nested_single_quote_attr() {
        let token = Token::SelfCloseAttr("video", vec![("id", Some("123"))]);
        let children: Vec<Code> = vec![];
        let code = Code::Nested(&token, children);
        let handlers: &[(&str, ShortcodeFn)] = &[("video", handler_video_id)];
        assert_eq!(code.render(handlers), r#"<video id="123"/>"#);
    }

    #[test]
    fn test_render_nested_ampersand_in_value() {
        let token = Token::SelfCloseAttr("video", vec![("src", Some("a&b"))]);
        let children: Vec<Code> = vec![];
        let code = Code::Nested(&token, children);
        let handlers: &[(&str, ShortcodeFn)] = &[("video", handler_video_src)];
        assert_eq!(code.render(handlers), r#"<video src="a&b"/>"#);
    }

    #[test]
    fn test_render_nested_empty_attr_key() {
        let token = Token::SelfCloseAttr("video", vec![("", Some("value"))]);
        let children: Vec<Code> = vec![];
        let code = Code::Nested(&token, children);
        let handlers: &[(&str, ShortcodeFn)] = &[("video", handler_val)];
        assert_eq!(code.render(handlers), "VAL=value");
    }

    #[test]
    fn test_render_nested_only_flag_attrs() {
        let token = Token::SelfCloseAttr("video", vec![("full", None), ("autoplay", None), ("loop", None)]);
        let children: Vec<Code> = vec![];
        let code = Code::Nested(&token, children);
        let handlers: &[(&str, ShortcodeFn)] = &[("video", handler_flags)];
        assert_eq!(code.render(handlers), "flags=full,autoplay,loop");
    }

    #[test]
    fn test_render_nested_mixed_spaces() {
        let token = Token::SelfCloseAttr("video", vec![("id", Some("123"))]);
        let children: Vec<Code> = vec![];
        let code = Code::Nested(&token, children);
        let handlers: &[(&str, ShortcodeFn)] = &[("video", handler_id)];
        assert_eq!(code.render(handlers), "ID=123");
    }

    #[test]
    fn test_render_nested_case_sensitive() {
        let token = Token::SelfClose("FOO");
        let code = Code::Inline(&token);
        let handlers: &[(&str, ShortcodeFn)] = &[("foo", handler_foo)];
        assert_eq!(code.render(handlers), "[FOO]");
    }

    #[test]
    fn test_render_nested_empty_children() {
        let token = Token::SelfClose("tag");
        let code = Code::Nested(&token, vec![]);
        let handlers: &[(&str, ShortcodeFn)] = &[("tag", handler_tag)];
        assert_eq!(code.render(handlers), "<tag></tag>");
    }

    #[test]
    fn test_render_nested_many_children() {
        let strs: Vec<String> = (0..10)
            .map(|i| format!("{} ", i))
            .collect();
        let tokens: Vec<Token> = strs.iter().map(|s| Token::Text(s.as_str())).collect();
        let children: Vec<Code> = tokens.iter().map(|t| Code::Inline(t)).collect();
        let token = Token::SelfClose("tag");
        let code = Code::Nested(&token, children);
        let handlers: &[(&str, ShortcodeFn)] = &[("tag", handler_tag)];
        assert_eq!(code.render(handlers), "<tag>0 1 2 3 4 5 6 7 8 9 </tag>");
    }

    #[test]
    fn test_render_nested_deeply_interleaved() {
        let text3 = Code::Inline(&Token::Text(" c "));
        let inner_c = Token::SelfClose("c");
        let code_c = Code::Nested(&inner_c, vec![text3]);
        let text2 = Code::Inline(&Token::Text(" b "));
        let inner_b = Token::SelfClose("b");
        let code_b = Code::Nested(&inner_b, vec![code_c, text2]);
        let text1 = Code::Inline(&Token::Text(" a "));
        let inner_a = Token::SelfClose("a");
        let code_a = Code::Nested(&inner_a, vec![code_b, text1]);
        let handlers: &[(&str, ShortcodeFn)] = &[
            ("a", handler_a_interleaved),
            ("b", handler_b_interleaved),
            ("c", handler_c_interleaved),
        ];
        assert_eq!(code_a.render(handlers), "<a><b><c> c </c> b </b> a </a>");
    }

    #[test]
    fn test_render_nested_large_attr_value() {
        let long_val = "x".repeat(1000);
        let token = Token::SelfCloseAttr("video", vec![("id", Some(&long_val))]);
        let children: Vec<Code> = vec![];
        let code = Code::Nested(&token, children);
        let handlers: &[(&str, ShortcodeFn)] = &[("video", handler_len)];
        assert_eq!(code.render(handlers), "len=1000");
    }

    #[test]
    fn test_render_nested_unknown_deeply() {
        let z = Token::SelfClose("z");
        let z_code = Code::Inline(&z);
        let y = Token::SelfClose("y");
        let y_code = Code::Inline(&y);
        let x = Token::SelfClose("x");
        let x_code = Code::Inline(&x);
        assert_eq!(x_code.render(&[]), "[x]");
        assert_eq!(y_code.render(&[]), "[y]");
        assert_eq!(z_code.render(&[]), "[z]");
    }

    #[test]
    fn test_render_nested_mixed_known_unknown() {
        let unknown = Token::SelfClose("z");
        let unknown_code = Code::Inline(&unknown);
        let known_token = Token::SelfClose("y");
        let known_code = Code::Nested(&known_token, vec![unknown_code]);
        let outer_token = Token::SelfClose("x");
        let outer_code = Code::Nested(&outer_token, vec![known_code]);
        let handlers: &[(&str, ShortcodeFn)] = &[
            ("x", handler_x),
            ("y", handler_y),
        ];
        assert_eq!(outer_code.render(handlers), "<x><y>[z]</y></x>");
    }

    #[test]
    fn test_render_nested_handler_receives_rendered_children() {
        let inner_token = Token::SelfClose("inner");
        let inner_code = Code::Inline(&inner_token);
        let token = Token::SelfClose("outer");
        let code = Code::Nested(&token, vec![inner_code]);
        let handlers: &[(&str, ShortcodeFn)] = &[
            ("outer", handler_outer_inner),
            ("inner", handler_inner_handled),
        ];
        assert_eq!(code.render(handlers), "outer(inner_handled)-inner(handled)");
    }

    #[test]
    fn test_render_nested_handler_receives_nested_rendered() {
        let innermost = Token::SelfClose("z");
        let innermost_code = Code::Inline(&innermost);
        let middle_token = Token::SelfClose("y");
        let middle_code = Code::Nested(&middle_token, vec![innermost_code]);
        let outer_token = Token::SelfClose("x");
        let outer_code = Code::Nested(&outer_token, vec![middle_code]);
        let handlers: &[(&str, ShortcodeFn)] = &[
            ("x", handler_x_nested),
            ("y", handler_y_nested),
            ("z", handler_z),
        ];
        assert_eq!(outer_code.render(handlers), "x(y(Z))");
    }

    #[test]
    fn test_render_nested_empty_string_handler() {
        let token = Token::SelfClose("tag");
        let children: Vec<Code> = vec![];
        let code = Code::Nested(&token, children);
        let handlers: &[(&str, ShortcodeFn)] = &[("tag", handler_empty)];
        assert_eq!(code.render(handlers), "");
    }

    #[test]
    fn test_render_nested_handler_with_special_chars() {
        let text_token = Token::Text("<>&\"'");
        let text_code = Code::Inline(&text_token);
        let token = Token::SelfClose("tag");
        let code = Code::Nested(&token, vec![text_code]);
        let handlers: &[(&str, ShortcodeFn)] = &[("tag", handler_tag)];
        assert_eq!(code.render(handlers), "<tag><>&\"'</tag>");
    }
}
