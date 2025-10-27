use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub enum Token<'a> {
    Text(&'a str),
    SelfClose(&'a str),
    SelfCloseAttr(&'a str, Vec<(&'a str, Option<&'a str>)>),
    CloseTag(&'a str),
}

impl<'a> Token<'a> {
    pub fn tag_name(&self) -> Option<&str> {
        match self {
            Token::SelfClose(name) => Some(name),
            Token::SelfCloseAttr(name, _) => Some(name),
            Token::CloseTag(name) => Some(name),
            Token::Text(_) => None,
        }
    }

    pub fn render_raw(&self) -> String {
        match self {
            Token::Text(text) => text.to_string(),
            Token::SelfClose(name) => format!("[{}]", name),
            Token::CloseTag(name) => format!("[/{}]", name),
            Token::SelfCloseAttr(name, attrs) => {
                format!("[{} {}]", name, Token::attrs_to_string(attrs))
            }
        }
    }

    pub fn attrs_to_string(attrs: &Vec<(&str, Option<&str>)>) -> String {
        attrs
            .iter()
            .map(|(name, value)| {
                if let Some(v) = value {
                    format!("{}=\"{}\"", name, v)
                } else {
                    name.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    pub fn get_attr_map(&self) -> HashMap<&str, Option<&str>> {
        match self {
            Token::SelfCloseAttr(_, attrs) => {
                attrs.iter().map(|(name, value)| (*name, *value)).collect()
            }
            _ => HashMap::new(),
        }
    }
}
