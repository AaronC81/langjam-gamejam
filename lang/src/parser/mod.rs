use std::error::Error;

use nom::{IResult, Parser, bytes::complete::{tag, take_while, take_while1}, character::complete::satisfy, combinator::map, multi::many0};

use crate::{Declaration, Statement};

mod expression;
mod statement;
mod declaration;

fn ws1(input: &str) -> IResult<&str, &str> {
    take_while1(char::is_whitespace)(input)
}

fn ws0(input: &str) -> IResult<&str, &str> {
    take_while(char::is_whitespace)(input)
}

fn identifier(input: &str) -> IResult<&str, String> {
    fn is_first_identifier_character(c: char) -> bool {
        c.is_alphabetic() || c == '_'
    }

    fn is_identifier_character(c: char) -> bool {
        c.is_alphanumeric() || c == '_'
    }

    let (input, first) = satisfy(is_first_identifier_character)(input)?;
    let (input, rest) = take_while(is_identifier_character)(input)?;
    
    let id = format!("{first}{rest}");
    Ok((input, id))
}

fn instance_var_identifier(input: &str) -> IResult<&str, String> {
    let (input, _) = tag("@")(input)?;
    identifier(input)
}

fn braced_body<'a, T>(inner: impl Fn(&str) -> IResult<&str, T>) -> impl Parser<&'a str, Output = Vec<T>, Error = nom::error::Error<&'a str>> {
    map(
        (
            ws0,
            tag("{"),
            many0(
                map((ws0, inner, ws0), |(_, i, _)| i),
            ),
            tag("}"),
            ws0,
        ),
        |(_, _, s, _, _)| s,
    )
}

fn statement_body(input: &str) -> IResult<&str, Vec<Statement>> {
    braced_body(statement::statement).parse(input)
}

fn declaration_body(input: &str) -> IResult<&str, Vec<Declaration>> {
    braced_body(declaration::declaration).parse(input)
}

pub fn parse(input: &str) -> Result<Vec<Declaration>, Box<dyn Error + '_>> {
    let (remaining, declarations) =
        many0(
            map((ws0, declaration::declaration, ws0), |(_, d, _)| d),
        ).parse(input)?;

    if !remaining.is_empty() {
        return Err("parse error - not all input consumed".into());
    }

    Ok(declarations)
}
