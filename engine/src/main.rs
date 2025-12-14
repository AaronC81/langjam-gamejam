use langjam_gamejam_lang::{BinaryOperator, Declaration, Expression, Interpreter, Statement};
use raylib::prelude::*;

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(640, 480)
        .title("Hello, World")
        .build();
    rl.set_target_fps(60);

    let mut interpreter = Interpreter::new();

    interpreter.interpret_declaration(&Declaration::EntityDeclaration {
        name: "FpsTest".to_owned(),
        body: vec![
            Declaration::InstanceVarDeclaration { name: "ticks".to_owned() },
            Declaration::ConstructorDeclaration { body: vec![
                Statement::Assignment {
                    target: Expression::InstanceVarIdentifier("ticks".to_owned()),
                    value: Expression::NumberLiteral(0.0),
                },
            ] },
            Declaration::TickDeclaration {
                body: vec![
                    Statement::Assignment {
                        target: Expression::InstanceVarIdentifier("ticks".to_owned()),
                        value: Expression::BinaryOperation {
                            left: Box::new(Expression::InstanceVarIdentifier("ticks".to_owned())),
                            right: Box::new(Expression::NumberLiteral(1.0)),
                            operator: BinaryOperator::Add,
                        },
                    },
                    Statement::Expression(
                        Expression::Echo(
                            Box::new(Expression::InstanceVarIdentifier("ticks".to_owned())),
                        )
                    ),
                ],
            },
        ],
    }, None).unwrap();

    interpreter.interpret_declaration(&Declaration::GameInitDeclaration { body: vec![
        Statement::Expression(
            Expression::AddEntity { name: "FpsTest".to_owned() },
        ),
    ] }, None).unwrap();


    interpreter.execute_init().unwrap();
    while !rl.window_should_close() {
        interpreter.execute_tick().unwrap();

        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::WHITE);
        d.draw_text("Hello, world!", 12, 12, 20, Color::BLACK);
    }
}
