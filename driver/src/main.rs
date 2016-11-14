extern crate chip8_emu;
extern crate curl;

use std::io::Write;

use chip8_emu::Chip8;

use curl::easy::Easy;

fn main() {
	let mut buffer = [0u8; 1536];
    let mut easy = Easy::new();
    easy.url("https://github.com/badlogic/chip8/blob/master/roms/pong.rom?raw=true").unwrap();

    {
	    let mut transfer = easy.transfer();
	    transfer.write_function(|data| {
	    	Ok((&mut buffer[..]).write(data).unwrap())
	    }).unwrap();
	    transfer.perform().unwrap();
	}

    let mut chip8 = Chip8::new_and_init();
    chip8.load_rom(&buffer);

}
