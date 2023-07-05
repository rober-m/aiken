use chumsky::prelude::*;

use crate::{
    ast,
    parser::{error::ParseError, token::Token},
};

pub fn parser() -> impl Parser<Token, ast::UntypedDefinition, Error = ParseError> {
    let unqualified_import = choice((
        select! {Token::Name { name } => name}.then(
            just(Token::As)
                .ignore_then(select! {Token::Name { name } => name})
                .or_not(),
        ),
        select! {Token::UpName { name } => name}.then(
            just(Token::As)
                .ignore_then(select! {Token::UpName { name } => name})
                .or_not(),
        ),
    ))
    .map_with_span(|(name, as_name), span| ast::UnqualifiedImport {
        name,
        location: span,
        as_name,
        layer: Default::default(),
    });

    let unqualified_imports = just(Token::Dot)
        .ignore_then(
            unqualified_import
                .separated_by(just(Token::Comma))
                .allow_trailing()
                .delimited_by(just(Token::LeftBrace), just(Token::RightBrace)),
        )
        .or_not();

    let as_name = just(Token::As)
        .ignore_then(select! {Token::Name { name } => name})
        .or_not();

    let module_path = select! {Token::Name { name } => name}
        .separated_by(just(Token::Slash))
        .then(unqualified_imports)
        .then(as_name);

    just(Token::Use).ignore_then(module_path).map_with_span(
        |((module, unqualified), as_name), span| {
            ast::UntypedDefinition::Use(ast::Use {
                module,
                as_name,
                unqualified: unqualified.unwrap_or_default(),
                package: (),
                location: span,
            })
        },
    )
}

#[cfg(test)]
mod tests {
    use crate::assert_definition;

    #[test]
    fn import_basic() {
        assert_definition!("use aiken/list");
    }

    #[test]
    fn import_unqualified() {
        assert_definition!(
            r#"
            use std/address.{Address as A, thing as w}
            "#
        );
    }

    #[test]
    fn import_alias() {
        assert_definition!("use aiken/list as foo");
    }
}
