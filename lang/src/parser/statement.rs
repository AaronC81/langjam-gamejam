use nom::{IResult, Parser, branch::alt, bytes::complete::tag, character::complete::char, combinator::map};

use crate::{Statement, parser::{expression::expression, statement_body, ws0, ws1}};

fn if_statement(input: &str) -> IResult<&str, Statement> {
    map(
        (
            tag("if"),
            ws0,
            char('('),
            ws0,
            expression,
            ws0,
            char(')'),
            ws0,
            statement_body,
            // TODO: `else`
        ),
        |(_, _, _, _, condition, _, _, _, true_body)| Statement::IfConditional { condition, true_body, false_body: None }
    ).parse(input)
}

pub fn statement(input: &str) -> IResult<&str, Statement> {
    alt((
        if_statement,
        map((tag("return"), ws1, expression, ws0, tag(";")), |(_, _, e, _, _)| Statement::Return(e)),
        map(
            (expression, ws0, tag("="), ws0, expression, ws0, tag(";")),
            |(target, _, _, _, value, _, _)| Statement::Assignment { target, value },
        ),
        map((expression, ws0, tag(";")), |(e, _, _)| Statement::Expression(e)),
    )).parse(input)
}
