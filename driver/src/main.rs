extern crate chip8_emu;
extern crate sdl2;
extern crate rand;

use std::env;
use std::fs::File;
use std::io::Read;
use std::sync::atomic::{AtomicBool, Ordering};

use chip8_emu::Chip8;

use rand::{Rng, XorShiftRng};

use sdl2::EventPump;
use sdl2::audio::{AudioCallback, AudioSpecDesired};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Renderer;

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: chip8_emu_driver rom");
        return;
    }

    let mut chip8 = init_chip8_with_rom(&args[1]);

    let sdl = sdl2::init().unwrap();
    let mut event_pump = sdl.event_pump().unwrap();

    let audio = sdl.audio().unwrap();
    let video = sdl.video().unwrap();
    let window = video.window("Chip 8 emu", 640, 320).build().unwrap();
    let mut renderer = window.renderer().accelerated().present_vsync().build().unwrap();

    // Because audio playback happens on a separate thread, we need to use a
    // thread-safe flag to tell the callback whether it should be playing
    // anything or not.
    let should_play_flag = AtomicBool::new(false);

    struct SquareWaveGenerator<'a> {
        frequency: f32,
        phase: f32,
        should_play_flag: &'a AtomicBool,
    }

    impl<'a> AudioCallback for SquareWaveGenerator<'a> {
        type Channel = i16;

        fn callback(&mut self, out: &mut [i16]) {
            for x in out.iter_mut() {
                let volume = if self.should_play_flag.load(Ordering::Acquire) { 3000 } else { 0 };
                *x = if self.phase <= 0.5 { volume } else { -volume };                
                self.phase = (self.phase + self.frequency) % 1.0;
            }
        }
    }

    let requested_spec = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(1),
        samples: None,
    };

    let audio_device = audio.open_playback(None, &requested_spec, |spec| {
            SquareWaveGenerator {
                frequency: 125.0 / spec.freq as f32,
                phase: 0.0,
                should_play_flag: &should_play_flag,
            }
        })
        .unwrap();
    audio_device.resume();

    loop {
        if let EventSignal::Quit = handle_events(&mut chip8, &mut event_pump) {
            // We got a quit signal, time to exit.
            return;
        }

        execute_for_frame(&mut chip8);
        should_play_flag.store(chip8.should_play_sound(), Ordering::Release);
        draw_emu_screen(&mut chip8, &mut renderer);
    }
}

fn init_chip8_with_rom(path: &String) -> Chip8<XorShiftRng> {
    let mut buffer = [0u8; 3584];
    let mut file = File::open(path).unwrap();
    file.read(&mut buffer[..]).unwrap();

    let mut chip8 = Chip8::new_and_init();
    chip8.load_rom(&buffer);
    return chip8;
}

enum EventSignal {
    Quit,
    DoNothing,
}

fn handle_events<R: Rng>(chip8: &mut Chip8<R>, event_pump: &mut EventPump) -> EventSignal {
    for event in event_pump.poll_iter() {
        match event {
            Event::Quit { .. } => {
                return EventSignal::Quit;
            }
            Event::KeyDown { keycode: Some(key), .. } => {
                pass_key_to_emu(chip8, key, true);
            }
            Event::KeyUp { keycode: Some(key), .. } => {
                pass_key_to_emu(chip8, key, false);
            }
            _ => {
                // Skip this event.
            }
        }
    }

    return EventSignal::DoNothing;
}

fn pass_key_to_emu<R: Rng>(chip8: &mut Chip8<R>, key: Keycode, is_pressed: bool) {
    match key {
        Keycode::Num1       => chip8.set_key_state(0x1, is_pressed),
        Keycode::Num2       => chip8.set_key_state(0x2, is_pressed),
        Keycode::Num3       => chip8.set_key_state(0x3, is_pressed),
        Keycode::Num4       => chip8.set_key_state(0xC, is_pressed),
        Keycode::Q          => chip8.set_key_state(0x4, is_pressed),
        Keycode::W          => chip8.set_key_state(0x5, is_pressed),
        Keycode::E          => chip8.set_key_state(0x6, is_pressed),
        Keycode::R          => chip8.set_key_state(0xD, is_pressed),
        Keycode::A          => chip8.set_key_state(0x7, is_pressed),
        Keycode::S          => chip8.set_key_state(0x8, is_pressed),
        Keycode::D          => chip8.set_key_state(0x9, is_pressed),
        Keycode::F          => chip8.set_key_state(0xE, is_pressed),
        Keycode::Z          => chip8.set_key_state(0xA, is_pressed),
        Keycode::X          => chip8.set_key_state(0x0, is_pressed),
        Keycode::C          => chip8.set_key_state(0xB, is_pressed),
        Keycode::V          => chip8.set_key_state(0xF, is_pressed),
        _                   => {},
    }
}

// Note: The execution rate and timers are frame-rate dependent.
// An improvement would be to measure elapsed time and execute based on that.

fn execute_for_frame<R: Rng>(chip8: &mut Chip8<R>) {
    // Execute 10 opcodes every frame
    for _ in 0..10 {
        chip8.execute_next_opcode();
    }
    // Timers should execute at 60Hz.
    chip8.update_timers();
}

fn draw_emu_screen<R: Rng>(chip8: &mut Chip8<R>, renderer: &mut Renderer) {
    // Note that we're not making use of the emu draw flag since we're always
    // drawing something on each frame.
    renderer.clear();
    let emu_screen = chip8.get_screen_ref();
    for y in 0..32 {
        for x in 0..64 {
            let is_active_cell = emu_screen[y][x];
            if is_active_cell {
                renderer.set_draw_color(Color::RGB(255, 255, 224));
            } else {
                renderer.set_draw_color(Color::RGB(0, 0, 0));
            }
            let rect = Rect::new(x as i32 * 10, y as i32 * 10, 10, 10);
            renderer.fill_rect(rect).unwrap();
        }
    }
    renderer.present();
}
