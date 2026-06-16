# Shortcode Parser

A small, experimental Rust crate that parses WordPress‑flavored shortcodes and lets you render them with your own handlers.

Status: experimental. The API may change. Feel free to use it as a local or Git dependency to experiment.

- Parses tags like `[gallery ids="1,2,3"]`
- Passes attributes to your handler as key/value pairs
- Lets you decide how each shortcode should render (return any `String`)

## Installation

This crate isn’t published on crates.io yet. Add it to your project either via a local path or directly from Git.

Using a local path:

```toml
[dependencies]
shortcode_parser = { path = "../path/to/shortcode-parser" }
```

Using Git (main branch):

```toml
[dependencies]
shortcode_parser = { git = "https://github.com/mehedimi/shortcode-parser", branch = "main" }
```

## Quick start

Register a handler for a tag and render text containing that shortcode.

```rust
use shortcode_parser::shortcode::Shortcode;

fn main() {
    let mut sc = Shortcode::new();

    // Register a handler for [audio]
    sc.add("audio", |_content, attrs| {
        // attrs is an Option of key/value pairs for the shortcode attributes
        let tag_attrs = attrs
            .iter()
            .map(|(k, v)| format!("{}=\"{}\"", k, v.unwrap()))
            .collect::<Vec<_>>()
            .join(" ");

        format!("<audio {}></audio>", tag_attrs)
    });

    let text = "This is a [audio class=\"audio\"] tag";
    let rendered = sc.render(text);

    println!("{}", rendered);
}
```

## Handlers: what you implement

- You register handlers by tag name with `Shortcode::add(name, handler)`.
- A handler receives two parameters: `(content, attrs)`
  - `content`: the inner text for enclosing shortcodes like `[note]…[/note]`, otherwise often empty/`None`
  - `attrs`: optional key/value attributes parsed from the shortcode
- Return a `String` from your handler — this is inserted into the output.

Example: enclosing shortcode with inner content

```rust
use shortcode_parser::shortcode::Shortcode;

fn main() {
    let mut sc = Shortcode::new();

    sc.add("note", |content, attrs| {
        let inner = content.unwrap_or_default();
        format!("<div class=\"{}\">{}</div>", attrs.get("class").unwrap().unwrap(), inner)
    });

    let input = "Please read [note class=\"warning\"]be careful[/note].";
    let output = sc.render(input);
    println!("{}", output);
}
```

## Supported shortcode shapes

- Self-closing: `[tag]` or `[tag key="val" key2="val2"]`
- Enclosing: `[tag]inner content[/tag]`

Nesting support, escaping rules, and edge cases may evolve while the crate is experimental.

## Design overview

Internally, the crate tokenizes the input, parses shortcodes, and delegates rendering to your registered handlers. Modules include a tokenizer, parser, and renderer, kept deliberately small and dependency‑light.

## Limitations and notes

- This is experimental; APIs may change without notice.
- Attribute parsing is simple and aims for WP‑style key/value pairs; malformed input may be parsed loosely.
- Unknown shortcodes are currently left as‑is unless you register a handler or choose to strip them in your own post‑processing.

## Roadmap

- More robust nesting and escaping behavior
- Configurable policy for unknown shortcodes (leave, strip, or pass through)
- Better error reporting and diagnostics
- Publication to crates.io once the API stabilizes

## Contributing

Issues and pull requests are welcome. If you plan a larger contribution, please open an issue first to discuss the design.

## Acknowledgements

Inspired by WordPress shortcodes and the many parser implementations across ecosystems.
