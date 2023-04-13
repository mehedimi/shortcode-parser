use crate::tokenizer::token::Token;
use std::collections::HashMap;

fn render_raw_nested_child(
    children: &Vec<Token>,
    shortcodes: &HashMap<String, fn(Option<String>, Option<HashMap<String, String>>) -> String>,
) -> String {
    return children
        .iter()
        .map(|t| match t.tag_name() {
            Some(tag) => match shortcodes.get(tag.as_str()) {
                Some(callback) => t.render(callback.to_owned()),
                None => t.clone().render_raw(shortcodes),
            },
            None => t.clone().render_raw(shortcodes),
        })
        .collect::<Vec<String>>()
        .join("");
}

fn render_nested_child(
    children: &Vec<Token>,
    callback: fn(Option<String>, Option<HashMap<String, String>>) -> String,
) -> String {
    return children
        .iter()
        .map(|t| {
            return t.render(callback);
        })
        .collect::<Vec<String>>()
        .join("");
}

fn render_raw_attributes(attrs: &HashMap<String, String>) -> String {
    return attrs
        .iter()
        .map(|attr| format!("{}=\"{}\"", attr.0, attr.1))
        .collect::<Vec<_>>()
        .join(" ");
}

impl Token {
    pub fn render(
        &self,
        callback: fn(Option<String>, Option<HashMap<String, String>>) -> String,
    ) -> String {
        return match self {
            Token::Text { content } => content.to_string(),
            Token::AttributeTag { attrs, .. } => callback(None, Some(attrs.clone())),
            Token::InlineTag { .. } => return callback(None, None),
            Token::NestedAttributeTag { children, .. } => {
                return render_nested_child(children, callback);
            }
            Token::NestedInlineTag { children, .. } => {
                return render_nested_child(children, callback);
            }
        };
    }

    pub fn render_raw(
        self,
        items: &HashMap<String, fn(Option<String>, Option<HashMap<String, String>>) -> String>,
    ) -> String {
        match self {
            Token::Text { content } => content,
            Token::InlineTag { tag } => format!("[{}]", tag),
            Token::AttributeTag { tag, attrs } => {
                format!("[{} {}]", tag, render_raw_attributes(&attrs))
            }
            Token::NestedInlineTag { tag, children } => {
                return format!(
                    "[{}]{}[/{}]",
                    tag.clone(),
                    render_raw_nested_child(&children, items),
                    tag
                );
            }
            Token::NestedAttributeTag {
                tag,
                attrs,
                children,
            } => {
                return format!(
                    "[{} {}]{}[/{}]",
                    tag.clone(),
                    render_raw_attributes(&attrs),
                    render_raw_nested_child(&children, items),
                    tag
                );
            }
        }
    }
}
