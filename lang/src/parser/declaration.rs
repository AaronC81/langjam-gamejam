use nom::{IResult, Parser, branch::alt, bytes::complete::tag, combinator::map, multi::many0};

use crate::{Declaration, Statement, parser::{identifier, instance_var_identifier, statement::statement, ws0, ws1}};

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
    braced_body(statement).parse(input)
}

fn declaration_body(input: &str) -> IResult<&str, Vec<Declaration>> {
    braced_body(declaration).parse(input)
}

pub fn declaration(input: &str) -> IResult<&str, Declaration> {
    alt((
        map((tag("entity"), ws1, identifier, ws0, declaration_body), |(_, _, name, _, body)| Declaration::EntityDeclaration { name, body }),
        map((tag("constructor"), ws0, statement_body), |(_, _, body)| Declaration::ConstructorDeclaration { body }),
        map((tag("tick"), ws0, statement_body), |(_, _, body)| Declaration::TickDeclaration { body }),
        map((tag("draw"), ws0, statement_body), |(_, _, body)| Declaration::DrawDeclaration { body }),
        map((tag("declare"), ws1, instance_var_identifier, ws0, tag(";")), |(_, _, name, _, _)| Declaration::InstanceVarDeclaration { name }),
        // TODO: function
    )).parse(input)
}
