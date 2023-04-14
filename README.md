# WP Shortcode Parser
WordPress shortcode parser written in rust.
> NB: Currently this crate is not for production use. 
> If you need to experiment with this, feel free use as local crate.

### Installation
```toml
[dependencies]
wp_shortcode = {path = "PATH_OF_THE_CRATE"}
```

### Example
Here is a simple `[audio]` shortcode parsed into html.

```rust
use wp_shortcode::Shortcode;

fn main() {
    let mut shortcode = Shortcode::new();

    shortcode.add("audio", |_content, attrs| {
        let tag_attrs = attrs
            .unwrap()
            .iter()
            .map(|attr| {
                return format!("{}=\"{}\"", attr.0, attr.1);
            })
            .collect::<Vec<_>>()
            .join(" ");

        return format!("<audio {}></audio>", tag_attrs);
    });

    let content = "This is a [audio class=\"audio\"] tag";

    dbg!(shortcode.render(content.to_string()));
    // Output: This is a <audio class="audio"></audio> tag
}
```
