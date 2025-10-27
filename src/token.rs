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
                format!("[{} {}]", name, self.get_attr_string().unwrap())
            }
        }
    }

    pub fn get_attr_string(&self) -> Option<String> {
        match self {
            Token::SelfCloseAttr(_, attrs) => Some(
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
                    .join(" "),
            ),
            _ => None,
        }
    }
}
