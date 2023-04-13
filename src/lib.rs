use std::collections::HashMap;

mod tokenizer;

pub struct Shortcode {
    items: HashMap<String, fn(Option<String>, Option<HashMap<String, String>>) -> String>,
}

impl Shortcode {
    pub fn new() -> Shortcode {
        return Shortcode {
            items: HashMap::new(),
        };
    }

    fn add(
        &mut self,
        name: String,
        callback: fn(Option<String>, Option<HashMap<String, String>>) -> String,
    ) -> &Self {
        self.items.insert(name, callback);
        return self;
    }

    fn has(&self, name: &str) -> bool {
        return self.items.contains_key(name);
    }

    fn render(&self, content: String) -> String {
        return tokenizer::parse(content)
            .iter()
            .map(|token| match token.tag_name() {
                Some(tag) => {
                    return match self.items.get(tag.as_str()) {
                        Some(callback) => token.render(callback.to_owned()),
                        None => token.clone().render_raw(&self.items),
                    }
                }
                None => token.clone().render_raw(&self.items),
            })
            .collect::<Vec<String>>()
            .join("");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_can_register_a_shortcode() {
        let mut s = Shortcode::new();

        s.add("home".to_string(), |_content, _attrs| {
            return "home shortcode".to_string();
        });

        assert_eq!(true, s.has("home"));
        assert_eq!(false, s.has("nothing"));
    }

    #[test]
    fn it_can_render_plain_text() {
        let s = Shortcode::new();

        let content = s.render("plain text".to_string());

        assert_eq!("plain text", content);
    }

    #[test]
    fn it_can_render_inline_tag() {
        let mut s = Shortcode::new();

        s.add("world".to_string(), |_content, _attrs| {
            return "planet".to_string();
        });

        let content = s.render("hello [world]".to_string());

        assert_eq!("hello planet", content);
    }

    #[test]
    fn it_can_render_inline_attribute_tag() {
        let mut s = Shortcode::new();

        s.add("world".to_string(), |_content, attrs| {
            return attrs.unwrap().get("r").unwrap().to_owned();
        });

        let content = s.render("hello [world r=\"sun\"]".to_string());

        assert_eq!("hello sun", content);
    }

    #[test]
    fn it_can_render_unknown_inline_attribute_tag() {
        let s = Shortcode::new();

        let content = s.render("hello [world r=\"sun\"]".to_string());

        assert_eq!("hello [world r=\"sun\"]", content);
    }

    #[test]
    fn it_can_render_nested_inline_tag() {
        let mut s = Shortcode::new();

        s.add("u".to_string(), |_c, _attrs| {
            return "U".to_string();
        });

        let content = s.render("hello [r][u][/r]".to_string());

        assert_eq!("hello [r]U[/r]", content);
    }

    #[test]
    fn it_can_render_nested_attributes_tag() {
        let mut s = Shortcode::new();

        s.add("u".to_string(), |_c, attrs| {
            return "U".repeat(attrs.unwrap().get("repeat").unwrap().parse().unwrap());
        });

        let content = s.render("hello [r f=\"true\"][u repeat=\"6\"][/r]".to_string());

        assert_eq!("hello [r f=\"true\"]UUUUUU[/r]", content);
    }
}
