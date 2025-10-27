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


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shortcode() {
        let mut shortcode = Shortcode::new();
        shortcode.add("test", |_, _| "Hello world".to_string());

        assert_eq!(shortcode.render("[test]"), "Hello world");
    }

    #[test]
    fn test_shortcode_with_content() {
        let mut shortcode = Shortcode::new();
        shortcode.add("test", |content, _| {
            format!("T {} T", content.unwrap())
        });
        assert_eq!(shortcode.render("[test]Hello world[/test]"), "T Hello world T");
    }

    #[test]
    fn test_shortcode_with_attr() {
        let mut shortcode = Shortcode::new();
        shortcode.add("test", |_, attrs| {
            format!("T {} T", attrs.get("name").unwrap().unwrap())
        });
        assert_eq!(shortcode.render("[test name=\"hello world\"]"), "T hello world T");
    }

    #[test]
    fn test_plain_text() {
        let shortcode = Shortcode::new();
        assert_eq!(shortcode.render("Hello world"), "Hello world");
    }

    #[test]
    fn test_multiple_shortcodes() {
        let mut shortcode = Shortcode::new();
        shortcode.add("test", |_, _| "Hello world".to_string());
        shortcode.add("test2", |_, _| "Hello world 2".to_string());
        assert_eq!(shortcode.render("[test] [test2]"), "Hello world Hello world 2");
    }

    #[test]
    fn test_multiple_shortcodes_with_content() {
        let mut shortcode = Shortcode::new();
        shortcode.add("test", |content, _| {
            format!("T {} T", content.unwrap())
        });
        shortcode.add("test2", |content, _| {
            format!("T {} T", content.unwrap())
        });
        assert_eq!(shortcode.render("[test]Hello world[/test] [test2]Hello world 2[/test2]"), "T Hello world T T Hello world 2 T");
    }

    #[test]
    fn test_multiple_shortcodes_with_attr() {
        let mut shortcode = Shortcode::new();
        shortcode.add("test", |_, attrs| {
            format!("T {} T", attrs.get("name").unwrap().unwrap())
        });
        shortcode.add("test2", |_, attrs| {
            format!("T {} T", attrs.get("name").unwrap().unwrap())
        });
        assert_eq!(shortcode.render("[test name=\"hello world\"] [test2 name=\"hello world 2\"]"), "T hello world T T hello world 2 T");
    }

    #[test]
    fn test_nested_shortcodes() {
        let mut shortcode = Shortcode::new();
        shortcode.add("test", |_, _| "Hello world".to_string());
        shortcode.add("test2", |content, _| {
            format!("T {} T", content.unwrap())
        });
        assert_eq!(shortcode.render("[test2][test][/test2]"), "T Hello world T");
    }
}