use crate::parser::Parser;
use crate::renderer::Renderer;
use std::borrow::Cow;
use std::collections::HashMap;

pub type ShortcodeFn = fn(Option<&str>, HashMap<&str, Option<&str>>) -> String;

#[derive(Debug)]
pub struct Shortcode<'a> {
    items: HashMap<&'a str, ShortcodeFn>,
}

impl<'a> Default for Shortcode<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Shortcode<'a> {
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
        }
    }

    pub fn add(&mut self, name: &'a str, func: ShortcodeFn) {
        self.items.insert(name, func);
    }

    pub fn has(&self, name: &str) -> bool {
        self.items.contains_key(name)
    }

    pub fn get(&self, name: &str) -> Option<&ShortcodeFn> {
        self.items.get(name)
    }

    pub fn render<'b>(&self, content: &'b str) -> Cow<'b, str> {
        let mut parser = Parser::new(content);
        let tokens = parser.parse();

        // Only one token and it's not a tag
        if tokens.len() == 1 && tokens[0].tag_name().is_none() {
            return Cow::Borrowed(content);
        }

        Cow::Owned(Renderer::new(tokens).render(&self.items))
    }
}
