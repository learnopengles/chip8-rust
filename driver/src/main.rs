extern crate chip8_emu;
extern crate sdl2;

use std::env;
use std::fs::File;
use std::io::{Read, Write};

use chip8_emu::Chip8;

use sdl2::{Sdl};
use sdl2::pixels::PixelFormatEnum::BGR24;
use sdl2::render::{Renderer, Texture, TextureAccess};

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
	let renderer = window.renderer().accelerated().present_vsync().build().unwrap();
	let texture = renderer.create_texture(BGR24, TextureAccess::Streaming, 640, 320).unwrap();
	
	// TODO this burns up 100% of CPU
	loop {
		for event in event_pump.poll_iter() {
			use sdl2::event::Event;
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
	}
}
