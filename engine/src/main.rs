use langjam_gamejam_lang::{BinaryOperator, Declaration, Expression, Interpreter, Statement, parse};
use raylib::prelude::*;

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(640, 480)
        .title("Hello, World")
        .build();
    rl.set_target_fps(60);

    let declarations = parse("
        entity FpsTest {
            declare @ticks;

            constructor {
                @ticks = 0;
            }

            tick {
                @ticks = @ticks + 1;
                echo @ticks;
            }
        }

        gameinit {
            spawn FpsTest;
        }
    ").unwrap();
    let mut interpreter = Interpreter::with_declarations(&declarations).unwrap();


    interpreter.execute_init().unwrap();
    while !rl.window_should_close() {
        interpreter.execute_tick().unwrap();

        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::WHITE);
        d.draw_text("Hello, world!", 12, 12, 20, Color::BLACK);
    }
}
