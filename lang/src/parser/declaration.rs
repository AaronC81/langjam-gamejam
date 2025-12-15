use nom::{IResult, Parser, branch::alt, bytes::complete::tag, combinator::map, multi::{many0, separated_list1}};

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

pub fn declaration(input: &str) -> IResult<&str, Declaration> {
    alt((
        map((tag("entity"), ws1, identifier, ws0, declaration_body), |(_, _, name, _, body)| Declaration::EntityDeclaration { name, body }),
        map((tag("constructor"), ws0, statement_body), |(_, _, body)| Declaration::ConstructorDeclaration { body }),
        map((tag("tick"), ws0, statement_body), |(_, _, body)| Declaration::TickDeclaration { body }),
        map((tag("draw"), ws0, statement_body), |(_, _, body)| Declaration::DrawDeclaration { body }),
        instance_var_declaration,
        // TODO: function
    )).parse(input)
}
