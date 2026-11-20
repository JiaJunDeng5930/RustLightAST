#![allow(dead_code)]

use rustlightast::{parse_rust_source, Block, Expr, Item, ParseError, Pattern, RustCodeGenerator};

pub fn parse_expression(source: &str) -> Result<Expr, ParseError> {
    let module = parse_rust_source(&format!("fn sample() {{ {source} }}"), "fixture")?;
    let Item::Function(function) = module.items.into_iter().next().unwrap() else {
        panic!("expected fixture function");
    };
    Ok(*function.body.expr.expect("expected fixture expression"))
}

pub fn parse_pattern(source: &str) -> Result<Pattern, ParseError> {
    let expression = parse_expression(&format!("match value {{ {source} => () }}"))?;
    let Expr::Match { arms, .. } = expression else {
        panic!("expected match expression");
    };
    Ok(arms.into_iter().next().unwrap().pattern)
}

pub fn print_block(block: &Block) -> String {
    let mut module = parse_rust_source("fn sample() {}", "fixture").unwrap();
    let Item::Function(function) = &mut module.items[0] else {
        panic!("expected fixture function");
    };
    function.body = block.clone();
    let source = RustCodeGenerator::new().generate_module_code(&module);
    source[source.find('{').unwrap()..=source.rfind('}').unwrap()].to_owned()
}

pub fn print_expression(expression: &Expr) -> String {
    let source = print_block(&Block {
        stmts: Vec::new(),
        expr: Some(Box::new(expression.clone())),
    });
    source[1..source.len() - 1].trim().to_owned()
}

pub fn print_item(item: &Item) -> String {
    let mut module = parse_rust_source("", "fixture").unwrap();
    module.items.push(item.clone());
    RustCodeGenerator::new().generate_module_code(&module)
}
