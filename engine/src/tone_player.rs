//! This uses a tonne of unsafe because I can't figure out a way to make raylib-rs instantiate
//! a playable sound from a manually-constructed set of samples. It's easy in the original C library
//! and therefore also easy in the direct (unsafe) FFI

use std::{collections::HashMap, f64::consts::PI, ffi::c_void};

use langjam_gamejam_lang::Note;
use raylib::{audio::RaylibAudio, ffi};

pub struct TonePlayer<'a> {
    // Not actually used due to unsafe schenanigans, but proves you've at least initialised audio
    raylib_audio: &'a RaylibAudio,

    sounds: HashMap<(Note, usize), ffi::Sound>,
}

const SAMPLE_RATE: u32 = 44100;

impl<'a> TonePlayer<'a> {
    pub fn new(raylib_audio: &'a RaylibAudio) -> Self {
        Self {
            raylib_audio,
            sounds: HashMap::new(),
        }
    }

    pub fn play_sound(&mut self, note: Note, duration_millis: usize) {
        let sound = self.make_sound(note, duration_millis);
        unsafe { ffi::PlaySound(sound); }
    }

    fn make_sound(&mut self, note: Note, duration_millis: usize) -> ffi::Sound {
        // Cache waves to:
        //   - Avoid recalculation for sounds which have been played before
        //   - "Solve" lifetime issues by making them effectively static
        if let Some(sound) = self.sounds.get(&(note, duration_millis)) {
            return sound.clone();
        }

        let frequency = note.frequency();
        let duration = (duration_millis as f64) / 1000.0;
        let num_samples = (SAMPLE_RATE as f64 * duration) as usize;

        // Without a fade, there's a sharp "click" at the beginning of some notes - I'm not enough
        // of an audio person to understand why!
        let fade_samples = (SAMPLE_RATE as f64 * 0.005) as usize;

        // Claude special :(
        let mut samples: Vec<i16> = vec![0; num_samples];
        for i in 0..num_samples {
            let t = i as f64 / SAMPLE_RATE as f64;
            let sample = (2.0 * PI * frequency * t).sin();
            
            let envelope = if i < fade_samples {
                // Fade in
                i as f64 / fade_samples as f64
            } else if i > num_samples - fade_samples {
                // Fade out
                (num_samples - i) as f64 / fade_samples as f64
            } else {
                1.0
            };
            samples[i] = (sample * envelope * i16::MAX as f64) as i16;
        }
        
        // `sounds` hash ensures we don't leak any more memory than we need to
        let data = samples.leak().as_mut_ptr() as *mut c_void;
        let wave = raylib::ffi::Wave {
            frameCount: num_samples as u32,
            sampleRate: SAMPLE_RATE,
            sampleSize: 16,
            channels: 1,
            data,
        };

        let sound = unsafe { ffi::LoadSoundFromWave(wave) };
        self.sounds.insert((note, duration_millis), sound.clone());

        sound
    }
}
