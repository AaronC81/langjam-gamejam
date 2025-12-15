use crate::{BinaryOperator, Declaration, Expression, Interpreter, Object, Statement};

#[test]
fn test_basic_interpreter() {
    let mut interpreter = Interpreter::new();

    interpreter.interpret_declaration(&Declaration::EntityDeclaration {
        name: "Player".to_owned(),
        body: vec![
            Declaration::InstanceVarDeclaration { name: "score".to_owned() },
            Declaration::ConstructorDeclaration { body: vec![
                Statement::Assignment {
                    target: Expression::InstanceVarIdentifier("score".to_owned()),
                    value: Expression::NumberLiteral(0.0),
                },
            ] },
            Declaration::FunctionDeclaration {
                name: "complete_objective".to_owned(),
                parameters: vec![],
                body: vec![
                    Statement::Assignment {
                        target: Expression::InstanceVarIdentifier("score".to_owned()),
                        value: Expression::BinaryOperation {
                            left: Box::new(Expression::InstanceVarIdentifier("score".to_owned())),
                            right: Box::new(Expression::NumberLiteral(1.0)),
                            operator: BinaryOperator::Add,
                        },
                    }
                ],
            },
        ],
    }, None).unwrap();

    interpreter.interpret_declaration(&Declaration::ConstructorDeclaration { body: vec![
        Statement::Assignment {
            target: Expression::Identifier("plyr".to_owned()),
            value: Expression::AddEntity { name: "Player".to_owned() },
        },
        Statement::Expression(
            Expression::FunctionCall {
                target: Box::new(Expression::Identifier("plyr".to_owned())),
                name: "complete_objective".to_owned(),
                arguments: vec![],
            },
        ),
        Statement::Expression(
            Expression::FunctionCall {
                target: Box::new(Expression::Identifier("plyr".to_owned())),
                name: "complete_objective".to_owned(),
                arguments: vec![],
            },
        ),
    ] }, None).unwrap();

    interpreter.execute_init().unwrap();

    let entities = interpreter.entities().collect::<Vec<_>>();
    assert_eq!(entities.len(), 1);

    let player = entities[0];
    assert_eq!(player.ivars.len(), 1);
    assert_eq!(player.ivars["score"], Object::Number(2.0));
}
