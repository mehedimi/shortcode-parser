# AGENTS.md

## Project

`shortcode_parser` — zero-dependency Rust 2021 crate that parses WordPress-flavored shortcodes and renders them via user-registered handlers. Not published to crates.io.

## Commands

```
cargo build          # compile
cargo test           # run all inline tests
cargo test <name>    # run a single test, e.g. cargo test test_parse_self_close
```

No lint/typecheck/formatter config exists. `cargo clippy` and `cargo fmt` work but are not enforced.

## Architecture

Only `shortcode` module is public. Everything else is internal implementation.

```
src/lib.rs          — re-exports `pub mod shortcode`
src/shortcode.rs    — Shortcode registry (HashMap<&str, ShortcodeFn>), public API entry point
src/parser.rs       — Char-by-char tokenizer → Vec<Token>
src/token.rs        — Token enum: Text, SelfClose, SelfCloseAttr, CloseTag
src/renderer.rs     — Builds Code tree from tokens, resolves nesting via stack, delegates to handlers
src/code.rs         — Code enum (Inline / Nested), renders each token via handler lookup
```

Flow: `Shortcode::render()` → `Parser::parse()` → `Renderer::new()` → `Renderer::render()` → handler functions.

## Gotchas

- `Renderer::render()` at `src/renderer.rs:44` contains an unreleased `dbg!(&self.items)` — it prints to stderr on every render. Remove before shipping.
- `Parser::parse()` returns `&Vec<Token>` borrowing from `self`. The caller must keep the `Parser` alive. Currently fine because `Parser` is created inline in `Shortcode::render()`.
- `.gitignore` excludes `Cargo.lock`. Unusual for a library but intentional.
- Unknown shortcodes are left as-is in output (rendered raw). No built-in strip/ignore policy.
