use nom::{IResult, Parser, branch::alt, bytes::complete::{tag, take_while1}, character::complete::char, combinator::map, error::make_error, multi::{many0, many1, separated_list0}, number::complete::double};

use crate::{BinaryOperator, Expression, Pixel, Sprite, parser::{identifier, instance_var_identifier, ws0, ws1}};

fn number(input: &str) -> IResult<&str, f64> {
    double(input)
}

fn sprite_expression(input: &str) -> IResult<&str, Expression> {
    fn sprite_pixel(input: &str) -> IResult<&str, Pixel> {
        alt((
            map(char('#'), |_| Pixel::Set),
            map(char('.'), |_| Pixel::Clear),
        )).parse(input)
    }
    
    fn sprite_pixel_row(input: &str) -> IResult<&str, Vec<Pixel>> {
        many1(sprite_pixel).parse(input)
    }

    fn sprite(input: &str) -> IResult<&str, Sprite> {
        let (input, rows) = separated_list0(ws1, sprite_pixel_row).parse(input)?;

        match rows.as_slice() {
            [] => Ok((input, Sprite {
                width: 0,
                height: 0,
                pixels: vec![]
            })),

            [only] => Ok((input, Sprite {
                width: only.len(),
                height: 1,
                pixels: only.clone()
            })),

            [first, rest@..] => {
                // Validate that all rows are the same size
                for row in rest {
                    if row.len() != first.len() {
                        // TODO: better error
                        panic!("sprite has inconsistent row lengths")
                    }
                }

                // Concatenate all pixels
                let mut all_pixels = first.clone();
                for row in rest {
                    all_pixels.extend_from_slice(row);
                }

                Ok((input, Sprite {
                    width: first.len(),
                    height: rest.len() + 1,
                    pixels: all_pixels,
                }))
            },
        }
    }

    map(
        (tag("sprite"), ws0, tag("{"), ws0, sprite, ws0, tag("}")),
        |(_, _, _, _, sprite, _, _)| Expression::SpriteLiteral(sprite)
    ).parse(input)
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
        |(_, _, name)| Expression::SpawnEntity { name },
    ).parse(input)
}

fn destroy_expression(input: &str) -> IResult<&str, Expression> {
    map(
        (tag("destroy"), ws1, expression),
        |(_, _, expr)| Expression::DestroyEntity(Box::new(expr)),
    ).parse(input)
}

fn array_expression(input: &str) -> IResult<&str, Expression> {
    map(
        (
            char('['),
            ws0,
            separated_list0((ws0, char(','), ws0), expression),
            ws0,
            char(']'),
        ),
        |(_, _, items, _, _)| Expression::ArrayLiteral(items),
    ).parse(input)
}

fn atom_expression(input: &str) -> IResult<&str, Expression> {
    alt((
        map(tag("null"), |_| Expression::NullLiteral),
        map(tag("this"), |_| Expression::ThisLiteral),
        map(tag("true"), |_| Expression::BooleanLiteral(true)),
        map(tag("false"), |_| Expression::BooleanLiteral(false)),

        sprite_expression,
        array_expression,

        map(identifier, |id| Expression::Identifier(id)),
        map(instance_var_identifier, |id| Expression::InstanceVarIdentifier(id)),
        map(number, |n| Expression::NumberLiteral(n)),
    )).parse(input)
}

fn call_expression(input: &str) -> IResult<&str, Expression> {
    let (input, mut expr) = atom_expression(input)?;

    let (input, calls) = many0(
        map(
            (
                ws0,
                char('.'),
                ws0,
                identifier,
                ws0,
                char('('),
                separated_list0(
                    char(','),
                    map((ws0, expression, ws0), |(_, e, _)| e),
                ),
                char(')'),
            ),
            |(_, _, _, name, _, _, arguments, _)| (name, arguments),
        )
    ).parse(input)?;
    for (name, arguments) in calls {
        expr = Expression::FunctionCall {
            target: Box::new(expr),
            name,
            arguments,
        }
    }

    Ok((input, expr))
}

fn prefix_expression(input: &str) -> IResult<&str, Expression> {
    alt((
        echo_expression,
        spawn_expression,
        destroy_expression,
        call_expression,
    )).parse(input)
}

fn mul_div_expression(input: &str) -> IResult<&str, Expression> {
    let (input, mut expr) = prefix_expression(input)?;

    let (input, ops) = many0((
        ws0,
        alt((char('*'), char('/'))),
        ws0,
        prefix_expression,
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

fn cmp_expression(input: &str) -> IResult<&str, Expression> {
    let (input, mut expr) = add_sub_expression(input)?;

    let (input, ops) = many0((
        ws0,
        alt((tag("=="), tag("!="), tag("<"), tag(">"), tag("<="), tag(">="))),
        ws0,
        add_sub_expression,
    )).parse(input)?;
    for (_, op, _, right) in ops {
        let operator = match op {
            "==" => BinaryOperator::Equals,
            "!=" => BinaryOperator::NotEquals,
            "<" => BinaryOperator::LessThan,
            ">" => BinaryOperator::GreaterThan,
            "<=" => BinaryOperator::LessThanOrEquals,
            ">=" => BinaryOperator::GreaterThanOrEquals,
            _ => unreachable!(),
        };
        expr = Expression::BinaryOperation { left: Box::new(expr), right: Box::new(right), operator };
    }

    Ok((input, expr))
}

pub fn expression(input: &str) -> IResult<&str, Expression> {
    // TODO: binop
    // TODO: call

    cmp_expression(input)
}
