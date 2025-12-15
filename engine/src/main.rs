use langjam_gamejam_lang::{BinaryOperator, Declaration, Expression, InputReport, Interpreter, Pixel, Statement, parse};
use raylib::prelude::*;

const PIXEL_SIZE: i32 = 10;

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(640, 480)
        .title("Hello, World")
        .build();
    rl.set_target_fps(60);

    let declarations = parse("
        entity FpsTest {
            var @ticks;

            constructor {
                @ticks = 0;
            }

            tick {
                @ticks = @ticks + 1;
                echo @ticks;
            }
        }
        
        entity Smile {
            var @x, @y, @move_cooldown;

            constructor {
                @x = 2;
                @y = 2;
                @move_cooldown = 0;
            }

            tick {
                if (@move_cooldown > 0) {
                    @move_cooldown = @move_cooldown - 1;
                }

                if (Input.down_pressed()) {
                    if (@move_cooldown == 0) {
                        @y = @y + 1;
                        @move_cooldown = 3;
                    }
                }

                if (Input.up_pressed()) {
                    if (@move_cooldown == 0) {
                        @y = @y - 1;
                        @move_cooldown = 3;
                    }
                }
            }

            draw {
                return sprite {
                    .#.#.
                    .....
                    #...#
                    .###.
                };
            }
        }

        constructor {
            spawn FpsTest;
            spawn Smile;
        }
    ").unwrap();
    let mut interpreter = Interpreter::with_declarations(&declarations).unwrap();

    interpreter.execute_init().unwrap();
    while !rl.window_should_close() {
        interpreter.update_input_report(InputReport {
            up: rl.is_key_down(KeyboardKey::KEY_UP),
            down: rl.is_key_down(KeyboardKey::KEY_DOWN),
            left: rl.is_key_down(KeyboardKey::KEY_LEFT),
            right: rl.is_key_down(KeyboardKey::KEY_RIGHT),

            x: rl.is_key_down(KeyboardKey::KEY_X),
            z: rl.is_key_down(KeyboardKey::KEY_Z),
        });
        interpreter.execute_tick().unwrap();

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::WHITE);

        for draw_op in interpreter.execute_draw().unwrap() {
            let base_x = draw_op.x as i32 * PIXEL_SIZE;
            let base_y = draw_op.y as i32 * PIXEL_SIZE;
            
            for dx in 0..draw_op.sprite.width {
                for dy in 0..draw_op.sprite.height {
                    if draw_op.sprite.pixels[dy * draw_op.sprite.width + dx] == Pixel::Set {
                        let canvas_x = base_x + dx as i32 * PIXEL_SIZE;
                        let canvas_y = base_y + dy as i32 * PIXEL_SIZE;
            
                        d.draw_rectangle(canvas_x, canvas_y, PIXEL_SIZE, PIXEL_SIZE, Color::BLACK);
                    }
                }
            }
        }
    }
}
