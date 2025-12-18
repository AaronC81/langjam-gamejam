use std::process::exit;

use include_dir::{Dir, include_dir};
use langjam_gamejam_lang::{BinaryOperator, Declaration, DisplayConfig, Expression, InputReport, Interpreter, Pixel, Statement, Tone, parse};
use raylib::prelude::*;

use crate::tone_player::TonePlayer;

mod tone_player;

const PIXEL_SIZE: i32 = 10;

const WINDOW_WIDTH: i32 = 640;
const WINDOW_HEIGHT: i32 = 480;

const GAME_FILES: Dir = include_dir!("$CARGO_MANIFEST_DIR/../game");

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(WINDOW_WIDTH, WINDOW_HEIGHT)
        .title("Hello, World")
        .build();
    rl.set_target_fps(60);

    let raylib_audio = RaylibAudio::init_audio_device().unwrap();
    let mut tone_player = TonePlayer::new(&raylib_audio);

    let mut declarations = vec![];
    for file in GAME_FILES.files() {
        match parse(file.contents_utf8().unwrap()) {
            Ok(decls) => declarations.extend(decls),
            Err(err) => {
                println!("Error loading `{}`: {}", file.path().to_string_lossy(), err);
                exit(1);
            }
        }
    }

    let mut interpreter = Interpreter::with_declarations(&declarations).unwrap();

    interpreter.update_display_config(DisplayConfig {
        width: (WINDOW_WIDTH / PIXEL_SIZE) as usize,
        height: (WINDOW_HEIGHT / PIXEL_SIZE) as usize,
    });

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

        let sounds = interpreter.execute_tick().unwrap();
        for sound in sounds {
            println!("Playing: {sound:?}");
            let Tone { note, duration } = sound;
            tone_player.play_sound(note, (duration * 1000.0) as usize);
        }

        let fps = rl.get_fps();

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

        d.draw_text(&fps.to_string(), 1, 1, 8, Color::BLACK);
    }
}
