use std::collections::HashMap;

#[derive(Clone)]
pub enum Token {
    Text {
        content: String,
    },
    InlineTag {
        tag: String,
    },
    AttributeTag {
        tag: String,
        attrs: HashMap<String, String>,
    },
    NestedAttributeTag {
        tag: String,
        attrs: HashMap<String, String>,
        children: Vec<Token>,
    },
    NestedInlineTag {
        tag: String,
        children: Vec<Token>,
    },
}

impl Token {
    pub fn tag_name(&self) -> Option<String> {
        match self {
            Token::Text { .. } => None,
            Token::AttributeTag { tag, .. } => Some(tag.to_owned()),
            Token::InlineTag { tag } => Some(tag.to_owned()),
            Token::NestedInlineTag { tag, .. } => Some(tag.to_owned()),
            Token::NestedAttributeTag { tag, .. } => Some(tag.to_owned()),
        }
    }
}
