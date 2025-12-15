use nom::{IResult, Parser, branch::alt, bytes::complete::tag, character::complete::char, combinator::map, multi::many0, number::complete::double};

use crate::{BinaryOperator, Expression, parser::{identifier, instance_var_identifier, ws0, ws1}};

fn number(input: &str) -> IResult<&str, f64> {
    double(input)
}

fn echo_expression(input: &str) -> IResult<&str, Expression> {
    map(
        (tag("echo"), ws1, expression),
        |(_, _, e)| Expression::Echo(Box::new(e)),
    ).parse(input)
}

fn spawn_expression(input: &str) -> IResult<&str, Expression> {
    map(
        (tag("spawn"), ws1, identifier),
        |(_, _, name)| Expression::AddEntity { name },
    ).parse(input)
}

fn atom_expression(input: &str) -> IResult<&str, Expression> {
    alt((
        map(tag("null"), |_| Expression::NullLiteral),
        map(tag("this"), |_| Expression::ThisLiteral),

        echo_expression,
        spawn_expression,

        map(identifier, |id| Expression::Identifier(id)),
        map(instance_var_identifier, |id| Expression::InstanceVarIdentifier(id)),
        map(number, |n| Expression::NumberLiteral(n)),
    )).parse(input)
}

fn mul_div_expression(input: &str) -> IResult<&str, Expression> {
    let (input, mut expr) = atom_expression(input)?;

    let (input, ops) = many0((
        ws0,
        alt((char('*'), char('/'))),
        ws0,
        atom_expression,
    )).parse(input)?;
    for (_, op, _, right) in ops {
        let operator = match op {
            '*' => BinaryOperator::Multiply,
            '/' => BinaryOperator::Divide,
            _ => unreachable!(),
        };
        expr = Expression::BinaryOperation { left: Box::new(expr), right: Box::new(right), operator };
    }

    Ok((input, expr))
}

fn add_sub_expression(input: &str) -> IResult<&str, Expression> {
    let (input, mut expr) = mul_div_expression(input)?;

    let (input, ops) = many0((
        ws0,
        alt((char('+'), char('-'))),
        ws0,
        mul_div_expression,
    )).parse(input)?;
    for (_, op, _, right) in ops {
        let operator = match op {
            '+' => BinaryOperator::Add,
            '-' => BinaryOperator::Subtract,
            _ => unreachable!(),
        };
        expr = Expression::BinaryOperation { left: Box::new(expr), right: Box::new(right), operator };
    }

    Ok((input, expr))
}

pub fn expression(input: &str) -> IResult<&str, Expression> {
    // TODO: binop
    // TODO: call

    add_sub_expression(input)
}
