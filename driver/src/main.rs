extern crate chip8_emu;
extern crate sdl2;

use std::env;
use std::fs::File;
use std::io::Read;

use chip8_emu::Chip8;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;

fn main() {
	let mut buffer = [0u8; 3584];
    let args: Vec<_> = env::args().collect();
    if args.len() < 2 {
    	println!("Usage: chip8_emu_driver rom");
    	return;
    }

    let mut file = File::open(&args[1]).unwrap();
    file.read(&mut buffer[..]).unwrap();    

    let mut chip8 = Chip8::new_and_init();
    chip8.load_rom(&buffer);

	let sdl = sdl2::init().unwrap();	
	let mut event_pump = sdl.event_pump().unwrap();

	let video = sdl.video().unwrap();
	let window = video.window("Chip 8 emu", 640, 320).build().unwrap();
	let mut renderer = window.renderer().accelerated().present_vsync().build().unwrap();	
	
	// TODO this burns up 100% of CPU
	// TODO update timers by using elapsed time
	loop {

		// TODO process events
		for event in event_pump.poll_iter() {			
			match event {
				Event::Quit { .. } => {
					// Should tear down everything
					return;
				},
				Event::KeyDown { keycode: Some(key), .. } => {
					match key {
						Keycode::Num1	 	=> chip8.set_key_state(0x1, true),
						Keycode::Num2	 	=> chip8.set_key_state(0x2, true),
						Keycode::Num3	 	=> chip8.set_key_state(0x3, true),
						Keycode::Num4	 	=> chip8.set_key_state(0xC, true),
						Keycode::Q 			=> chip8.set_key_state(0x4, true),
						Keycode::W 			=> chip8.set_key_state(0x5, true),
						Keycode::E 			=> chip8.set_key_state(0x6, true),
						Keycode::R 			=> chip8.set_key_state(0xD, true),
						Keycode::A 			=> chip8.set_key_state(0x7, true),
						Keycode::S 			=> chip8.set_key_state(0x8, true),
						Keycode::D 			=> chip8.set_key_state(0x9, true),
						Keycode::F 			=> chip8.set_key_state(0xE, true),
						Keycode::Z 			=> chip8.set_key_state(0xA, true),
						Keycode::X 			=> chip8.set_key_state(0x0, true),
						Keycode::C 			=> chip8.set_key_state(0xB, true),
						Keycode::V 			=> chip8.set_key_state(0xF, true),
						_					=> {},
					}
				},
				Event::KeyUp { keycode: Some(key), .. } => {
					match key {
						Keycode::Num1	 	=> chip8.set_key_state(0x1, false),
						Keycode::Num2	 	=> chip8.set_key_state(0x2, false),
						Keycode::Num3	 	=> chip8.set_key_state(0x3, false),
						Keycode::Num4	 	=> chip8.set_key_state(0xC, false),
						Keycode::Q 			=> chip8.set_key_state(0x4, false),
						Keycode::W 			=> chip8.set_key_state(0x5, false),
						Keycode::E 			=> chip8.set_key_state(0x6, false),
						Keycode::R 			=> chip8.set_key_state(0xD, false),
						Keycode::A 			=> chip8.set_key_state(0x7, false),
						Keycode::S 			=> chip8.set_key_state(0x8, false),
						Keycode::D 			=> chip8.set_key_state(0x9, false),
						Keycode::F 			=> chip8.set_key_state(0xE, false),
						Keycode::Z 			=> chip8.set_key_state(0xA, false),
						Keycode::X 			=> chip8.set_key_state(0x0, false),
						Keycode::C 			=> chip8.set_key_state(0xB, false),
						Keycode::V 			=> chip8.set_key_state(0xF, false),
						_					=> {},
					}
				},
				_ => {
					// Skip this event.
				}
			}
		}

		// Updating the emu
		// Note that this should really be done on a timer
		for _ in 0..10 {
			chip8.execute_next_opcode();
		}
		chip8.update_timers();

		// Drawing
		// Note that we're not making use of the emu draw flag since we're drawing whatever we find.
		{
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

		// TODO sound??
		// TODO input??	
	}
}
