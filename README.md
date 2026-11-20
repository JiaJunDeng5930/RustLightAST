# RustLightAST

A lightweight Rust subset AST, source parser, and printer.

```toml
[dependencies]
rustlightast = { path = "../RustLightAST" }
```

```rust
use rustlightast::{parse_rust_source, RustCodeGenerator};

fn main() -> Result<(), rustlightast::ParseError> {
    let module = parse_rust_source("fn identity(x: u32) -> u32 { x }", "example")?;
    println!("{}", RustCodeGenerator::new().generate_module_code(&module));
    Ok(())
}
```

- `rustlight_ast` defines the syntax tree.
- `rustlight_parser` converts supported Rust source into that tree.
- `rustlight_print` renders the tree as Rust source.

`parse_rust_type_facts` extracts declarations and signatures for analysis,
with function bodies omitted. Unsupported parser syntax returns `ParseError`.

Consumers that receive a RustLight model can depend on the model package:

```toml
[dependencies]
rustlightast = { package = "rustlightast-model", path = "../RustLightAST/model" }
```

It provides the syntax tree and printer. The parser package re-exports
the same types and can pass them directly to consumers of the model package.

Run tests with `cargo test --workspace --all-targets --locked`.
