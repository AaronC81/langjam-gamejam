use nom::{IResult, Parser, branch::alt, bytes::complete::tag, character::complete::char, combinator::map, multi::{many0, separated_list0, separated_list1}};

use crate::{Declaration, Statement, parser::{declaration_body, identifier, instance_var_identifier, statement::statement, statement_body, ws0, ws1}};

fn instance_var_declaration(input: &str) -> IResult<&str, Declaration> {
    map(
        (
            tag("var"),
            ws1,
            separated_list1((ws0, tag(","), ws0), instance_var_identifier),
            tag(";"),
        ),
        |(_, _, names, _)| Declaration::InstanceVarDeclaration { names },
    ).parse(input)
}

fn function_declaration(input: &str) -> IResult<&str, Declaration> {
    map(
        (
            tag("func"),
            ws1,
            identifier,
            ws0,
            char('('),
            separated_list0((ws0, char(','), ws0), identifier),
            char(')'),
            ws0,
            statement_body,
        ),
        |(_, _, name, _, _, parameters, _, _, body)| Declaration::FunctionDeclaration { name, parameters, body }
    ).parse(input)
}

pub fn declaration(input: &str) -> IResult<&str, Declaration> {
    alt((
        map((tag("entity"), ws1, identifier, ws0, declaration_body), |(_, _, name, _, body)| Declaration::EntityDeclaration { name, body }),
        map((tag("constructor"), ws0, statement_body), |(_, _, body)| Declaration::ConstructorDeclaration { body }),
        map((tag("tick"), ws0, statement_body), |(_, _, body)| Declaration::TickDeclaration { body }),
        map((tag("draw"), ws0, statement_body), |(_, _, body)| Declaration::DrawDeclaration { body }),
        instance_var_declaration,
        function_declaration,
    )).parse(input)
}
