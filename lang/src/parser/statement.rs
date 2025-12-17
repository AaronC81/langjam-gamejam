use nom::{IResult, Parser, branch::alt, bytes::complete::tag, character::complete::char, combinator::map};

use crate::{Expression, Statement, parser::{expression::expression, identifier, statement_body, ws0, ws1}};

fn parenthesised_expression(input: &str) -> IResult<&str, Expression> {
    map(
        (
            char('('),
            ws0,
            expression,
            ws0,
            char(')'),
        ),
        |(_, _, e, _, _)| e
    ).parse(input)
}

fn if_statement(input: &str) -> IResult<&str, Statement> {
    map(
        (
            tag("if"),
            ws0,
            parenthesised_expression,
            ws0,
            statement_body,
            // TODO: `else`
        ),
        |(_, _, condition, _, true_body)| Statement::IfConditional { condition, true_body, false_body: None }
    ).parse(input)
}

fn each_loop(input: &str) -> IResult<&str, Statement> {
    map(
        (
            tag("each"),
            ws1,
            identifier,
            ws1,
            tag("in"),
            ws0,
            parenthesised_expression,
            ws0,
            statement_body,
        ),
        |(_, _, variable, _, _, _, source, _, body)| Statement::EachLoop { variable, source, body }
    ).parse(input)
}

pub fn statement(input: &str) -> IResult<&str, Statement> {
    alt((
        if_statement,
        each_loop,
        map((tag("return"), ws1, expression, ws0, tag(";")), |(_, _, e, _, _)| Statement::Return(Some(e))),
        map((tag("return"), ws0, tag(";")), |_| Statement::Return(None)),
        map(
            (expression, ws0, tag("="), ws0, expression, ws0, tag(";")),
            |(target, _, _, _, value, _, _)| Statement::Assignment { target, value },
        ),
        map((expression, ws0, tag(";")), |(e, _, _)| Statement::Expression(e)),
    )).parse(input)
}
