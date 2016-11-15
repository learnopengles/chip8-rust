extern crate chip8_emu;
extern crate sdl2;

use std::env;
use std::fs::File;
use std::io::{Read, Write};

use chip8_emu::Chip8;

use sdl2::{Sdl};
use sdl2::event::Event;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Renderer;

fn main() {
	let mut buffer = [0u8; 1536];
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
				for x in 0..63 {
					let is_active_cell = emu_screen[y][x];
					if is_active_cell {
						renderer.set_draw_color(Color::RGB(255, 255, 224));
					} else {
						renderer.set_draw_color(Color::RGB(0, 0, 0));
					}
					let rect = Rect::new(x as i32 * 8, y as i32 * 8, 8, 8);
					renderer.fill_rect(rect).unwrap();
				}
			}
			renderer.present();
		}

		// TODO sound??
		// TODO input??	
	}
}
