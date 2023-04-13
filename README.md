# WP shortcode parser
WordPress shortcode parser written in rust.

### Example
Here is a simple `[audio]` shortcode parse into html.

```rust
use wp_shortcode::Shortcode;

fn main() {
    let mut shortcode = Shortcode::new();

    shortcode.add("audio".to_string(), |_content, attrs| {
        let tag_attrs = attrs
            .unwrap()
            .iter()
            .map(|attr| {
                return format!("{}=\"{}\"", attr.0, attr.1);
            })
            .collect::<Vec<_>>()
            .join(" ");

        return format!("<audio {tag_attrs}></audio>");
    });

    let content = "This is a [audio loop=\"true\"] tag";

    dbg!(shortcode.render(content.to_string()));
    // Output: This is a <audio loop="true"></audio> tag
}
```