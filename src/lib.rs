use std::collections::HashMap;

mod tokenizer;

pub struct Shortcode {
    items: HashMap<String, fn(Option<String>, Option<HashMap<String, Option<String>>>) -> String>,
}

impl Shortcode {
    pub fn new() -> Shortcode {
        return Shortcode {
            items: HashMap::new(),
        };
    }

    pub fn add(
        &mut self,
        name: &str,
        callback: fn(Option<String>, Option<HashMap<String, Option<String>>>) -> String,
    ) -> &Self {
        self.items.insert(name.to_string(), callback);
        return self;
    }

    pub fn has(&self, name: &str) -> bool {
        return self.items.contains_key(name);
    }

    pub fn render(&self, content: String) -> String {
        return tokenizer::Parser::new().parse(&content)
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

        s.add("home", |_content, _attrs| {
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

        s.add("world", |_content, _attrs| {
            return "planet".to_string();
        });

        let content = s.render("hello [world]".to_string());

        assert_eq!("hello planet", content);
    }

    #[test]
    fn it_can_render_inline_attribute_tag() {
        let mut s = Shortcode::new();

        s.add("world", |_content, attrs| {
            return attrs.unwrap().get("r").unwrap().clone().unwrap().to_owned();
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

        s.add("u", |_c, _attrs| {
            return "U".to_string();
        });

        let content = s.render("hello [r][u][/r]".to_string());

        assert_eq!("hello [r]U[/r]", content);
    }

    #[test]
    fn it_can_render_nested_attributes_tag() {
        let mut s = Shortcode::new();

        s.add("u", |_c, attrs| {
            return "U".repeat(attrs.unwrap().get("repeat").unwrap().clone().unwrap().parse().unwrap());
        });

        let content = s.render("hello [r f=\"true\"][u repeat=\"6\"][/r]".to_string());

        assert_eq!("hello [r f=\"true\"]UUUUUU[/r]", content);
    }

    #[test]
    fn it_can_accept_html_code_as_attribute() {
        let mut s = Shortcode::new();

        s.add("html", |_c, attrs| {
            return attrs.unwrap().get("code").unwrap().clone().unwrap().to_string()
        });

        let content = s.render("hello [html code='<div class=\"something\"></div>']".to_string());

        assert_eq!("hello <div class=\"something\"></div>", content);
    }

    #[test]
    fn it_can_handle_inline_attr() {
        let mut s = Shortcode::new();

        s.add("video", |_c, attrs| {
            let src = match attrs.clone().unwrap().get("src") {
                Some(src) => src.clone().unwrap(),
                None => "default.mp4".to_string()
            };

            let loop_value = match attrs.clone().unwrap().get("loop") {
                Some(..) => " loop",
                None => ""
            };

            return format!("<video src=\"{}\"{}></video>", src, loop_value)
        });

        let content = s.render("hello [video loop] [video src='custom.mp4']".to_string());

        assert_eq!("hello <video src=\"default.mp4\" loop></video> <video src=\"default.mp4\"></video>", content);
    }
}
