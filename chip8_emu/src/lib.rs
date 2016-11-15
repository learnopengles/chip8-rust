// TODO remove this external dependency
extern crate byteorder;

use byteorder::{BigEndian, ByteOrder};

struct Memory {
	ram: [u8; 2048],
}

// TODO reorganize?

impl Memory {
	fn load_font_into_memory(&mut self) {
		let chip8_fontset: [u8; 80] =
		[
			// Zero
			0b11110000,
			0b10010000,
			0b10010000,
			0b10010000,
			0b11110000,

			// One
			0b00100000, 
			0b01100000, 
			0b00100000, 
			0b00100000, 
			0b01110000,

			// Two
		  	0b11110000, 
		  	0b00010000, 
		  	0b11110000, 
		  	0b10000000, 
		  	0b11110000,

		  	// Three
		  	0b11110000, 
		  	0b00010000, 
		  	0b11110000, 
		  	0b00010000, 
		  	0b11110000,

		  	// Four
		  	0b10010000, 
		  	0b10010000, 
		  	0b11110000, 
		  	0b00010000, 
		  	0b00010000,

		  	// Five
		  	0b11110000, 
		  	0b10000000, 
		  	0b11110000, 
		  	0b00010000,
		  	0b11110000,

		  	// Six
		  	0b11110000, 
		  	0b10000000, 
		  	0b11110000, 
		  	0b10010000, 
		  	0b11110000,

		  	// Seven
		  	0b11110000, 
		  	0b00010000, 
		  	0b00100000, 
		  	0b01000000, 
		  	0b01000000,

		  	// Eight
		  	0b11110000, 
		  	0b10010000, 
		  	0b11110000, 
		  	0b10010000, 
		  	0b11110000,

		  	// Nine		  	
		  	0b11110000, 
		  	0b10010000, 
		  	0b11110000, 
		  	0b00010000, 
		  	0b11110000,

		  	// A
		  	0b11110000, 
		  	0b10010000, 
		  	0b11110000, 
		  	0b10010000, 
		  	0b10010000,

		  	// B
		  	0b11100000, 
		  	0b10010000, 
		  	0b11100000, 
		  	0b10010000, 
		  	0b11100000,

		  	// C
		  	0b11110000, 
		  	0b10000000, 
		  	0b10000000,
		  	0b10000000, 
		  	0b11110000,

		  	// D
		  	0b11100000, 
		  	0b10010000, 
		  	0b10010000, 
		  	0b10010000, 
		  	0b11100000,

		  	// E
		  	0b11110000, 
		  	0b10000000, 
		  	0b11110000, 
		  	0b10000000, 
		  	0b11110000,

		  	// F
		  	0b11110000, 
		  	0b10000000, 
		  	0b11110000, 
		  	0b10000000, 
		  	0b10000000,
		];	
		// Font should be loaded into offset 0x50 (80).
		self.ram[0x50..0xA0].copy_from_slice(&chip8_fontset);
	}

	#[inline]
	fn load_rom_into_memory(&mut self, rom: &[u8; 1536]) {
		self.ram[0x200..].copy_from_slice(rom);		
	}

	#[inline]
	fn read_unsigned_short(&self, position: u16) -> u16 {
		BigEndian::read_u16(&self.ram[position as usize..])		
	}
}

struct Registers {
	// Program counter
	pc: u16,
	// Index register
	index: u16,
	// 15 general-purpose registers + 1 carry flag
	v: [u8; 16],
}

struct Stack {
	// 16 stack addresses
	ret_addresses: [u16; 16],
	// Stack pointer
	sp: u16,
}

struct Input {
	// 16 keys
	keys: [u8; 16],
}

// TODO need a flag when need to update (AFAIK two ops should affect that?)

struct Display {
	//  64x32, black or white
	screen: [[bool; 64]; 32],
	needs_draw: bool,
}

pub struct Chip8 {
	memory: Memory,
	registers: Registers,
	stack: Stack,
	input: Input,
	display: Display,
}

impl Chip8 {
	pub fn new_and_init() -> Chip8 {
		let mut chip8 = Chip8 { 
			memory: Memory { ram: [0; 2048]},
			// Program counter starts at 0x200
			registers: Registers { pc: 0x200, index: 0, v: [0; 16]},
			stack: Stack { ret_addresses: [0; 16], sp: 0},
			input: Input { keys: [0; 16]},
			display: Display { screen: [[false; 64]; 32], needs_draw: false},
		};
		chip8.reset();
		return chip8;
	}	
	
	pub fn reset(&mut self) {
		self.memory.ram = [0; 2048];
		self.registers.pc = 0x200;
		self.registers.index = 0;
		self.registers.v = [0; 16];
		self.stack.ret_addresses = [0; 16];
		self.stack.sp = 0;
		self.input.keys = [0; 16];
		self.display.screen = [[false; 64]; 32];

		self.memory.load_font_into_memory();
	}

	pub fn load_rom(&mut self, rom: &[u8; 1536]) {
		self.memory.load_rom_into_memory(rom);
	}	

	fn execute_next_opcode(&mut self) {
		let opcode = self.memory.read_unsigned_short(self.registers.pc);				
		match opcode {
			0x00E0 => {
				// Clear the screen.
				self.display.screen = [[false; 64]; 32];
				self.display.needs_draw = true;
				self.registers.pc += 2;
			},
			0x00EE => {
				// Return from a subroutine.
				self.stack.sp -= 1;
				self.registers.pc = self.stack.ret_addresses[self.stack.sp as usize];
			},
			0x1000...0x1FFF => {
				// Jump
				let address = get_address_from_opcode(opcode);
				self.registers.pc = address;
			},
			0x2000...0x2FFF => {
				// Call a subroutine.
				// The return address should be after the opcode we're executing now.
				self.stack.ret_addresses[self.stack.sp as usize] = self.registers.pc + 2;
				self.stack.sp += 1;
				// Jump to the address in the opcode.
				let address = get_address_from_opcode(opcode);
				self.registers.pc = address;
			},
			_ => {
				// TODO unknown opcode
			}
		}
	}
}

fn get_address_from_opcode(opcode: u16) -> u16 {
	opcode & 0x0FFF
}

#[cfg(test)]
mod tests {
	use super::Memory;
    use super::Chip8;

    /// Memory tests ///  

    #[test]
    fn test_load_font() {
    	let mut memory = Memory { ram: [0; 2048]};
    	memory.load_font_into_memory();
    	for i in 0..80 {
    		assert_eq!(0, memory.ram[i]);
    	}
    }

    fn test_font_in_memory(memory: &Memory) {
    	// Check if zero is at the right place:
    	assert_eq!(0b11110000, memory.ram[0x50]);
    	assert_eq!(0b10010000, memory.ram[0x51]);
    	assert_eq!(0b10010000, memory.ram[0x52]);
    	assert_eq!(0b10010000, memory.ram[0x53]);
    	assert_eq!(0b11110000, memory.ram[0x54]);
    }

    #[test]
    fn test_load_rom() {
    	let mut memory = Memory { ram: [0; 2048]};
    	let mut rom: [u8; 1536] = [0; 1536];
    	rom[0] = 0xFF;
    	rom[1] = 0xCC;

    	memory.load_rom_into_memory(&rom);
    	assert_eq!(memory.ram[0x200], 0xFF);
    	assert_eq!(memory.ram[0x201], 0xCC);
    }

    #[test]
    fn test_read_unsigned_short() {
    	let mut chip8 = Chip8::new_and_init();
    	chip8.memory.ram[chip8.registers.pc as usize] = 0xFF;
    	chip8.memory.ram[(chip8.registers.pc + 1) as usize] = 0x77;
    	let next_opcode = chip8.memory.read_unsigned_short(chip8.registers.pc);
    	// If fetched in big-endian order, then should match as below:
    	assert_eq!(65399, next_opcode);
    }

    /// State tests ///
      
    #[test]
    fn test_default_state() {
    	// PC counter should default to 0x200:
    	let chip8 = Chip8::new_and_init();
    	assert_eq!(0x200, chip8.registers.pc);
    	// We should also already have the font in ram:
    	test_font_in_memory(&chip8.memory);
    }

    /// Opcode tests ///

    #[test]
    fn test_clear_screen() {
    	let mut chip8 = Chip8::new_and_init();
    	chip8.display.screen[0][0] = true;	
    	chip8.display.screen[0][1] = true;
    	chip8.display.screen[0][2] = true;

    	chip8.memory.ram[chip8.registers.pc as usize] = 0x00;
    	chip8.memory.ram[(chip8.registers.pc + 1) as usize] = 0xE0;
    	chip8.execute_next_opcode();

    	assert_eq!(false, chip8.display.screen[0][0]);
    	assert_eq!(false, chip8.display.screen[0][1]);
    	assert_eq!(false, chip8.display.screen[0][2]);
    }

    #[test]
    fn test_jump() {
   		let mut chip8 = Chip8::new_and_init();
   		chip8.memory.ram[chip8.registers.pc as usize] = 0x1E;
   		chip8.memory.ram[(chip8.registers.pc + 1) as usize] = 0xEE;
   		chip8.execute_next_opcode();

   		assert_eq!(0xEEE, chip8.registers.pc);
    }

    #[test]
    fn test_push_and_pop_stack() {
    	let mut chip8 = Chip8::new_and_init();
    	// Call subroutine at 2000
    	chip8.memory.ram[chip8.registers.pc as usize] = 0x27;
   		chip8.memory.ram[(chip8.registers.pc + 1) as usize] = 0xD0;

   		// Return back to caller
   		chip8.memory.ram[2000] = 0x00;
   		chip8.memory.ram[2001] = 0xEE;

   		// We won't execute this, will just use it to see that it's our next expected instruction.
   		chip8.memory.ram[(chip8.registers.pc + 2) as usize] = 0xAA;
   		chip8.memory.ram[(chip8.registers.pc + 3) as usize] = 0xAA;

   		// Execute first instruction -- should jump us to address 2000.
   		chip8.execute_next_opcode();
   		assert_eq!(2000, chip8.registers.pc);

   		// Next instruction should bring us back to the caller address + 2.
   		chip8.execute_next_opcode();
   		assert_eq!(0x200 + 2, chip8.registers.pc);

   		// Next two instructions should match 0xAAAA 
   		let next_opcode = chip8.memory.read_unsigned_short(chip8.registers.pc);
   		assert_eq!(0xAAAA, next_opcode);
    }      
}