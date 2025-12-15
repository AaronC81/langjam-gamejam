use nom::{IResult, Parser, branch::alt, bytes::complete::tag, combinator::map};

use crate::{Statement, parser::{expression::expression, ws0, ws1}};

pub fn statement(input: &str) -> IResult<&str, Statement> {
    alt((
        map((tag("return"), ws1, expression, ws0, tag(";")), |(_, _, e, _, _)| Statement::Return(e)),
        map(
            (expression, ws0, tag("="), ws0, expression, ws0, tag(";")),
            |(target, _, _, _, value, _, _)| Statement::Assignment { target, value },
        ),
        map((expression, ws0, tag(";")), |(e, _, _)| Statement::Expression(e)),
    )).parse(input)
}
