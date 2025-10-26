use std::borrow::Cow;

pub struct Shortcode;

impl Shortcode {
    pub fn new() -> Self {
        Self
    }

    pub fn parse(content: &'_ str) -> Cow<'_, str> {
        Cow::Borrowed(content)
    }
}
