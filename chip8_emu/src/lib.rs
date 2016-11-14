struct Memory {
	ram: [u8; 2048],
}

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
	stack: [u16; 16],
	// Stack pointer
	sp: u16,
}

struct Input {
	// 16 keys
	keys: [u8; 16],
}

struct Display {
	//  64x32, black or white
	screen: [[bool; 64]; 32],
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
			stack: Stack { stack: [0; 16], sp: 0},
			input: Input { keys: [0; 16]},
			display: Display { screen: [[false; 64]; 32]},
		};
		chip8.reset();
		return chip8;
	}	
	
	pub fn reset(&mut self) {
		self.memory.ram = [0; 2048];
		self.registers.pc = 0x200;
		self.registers.index = 0;
		self.registers.v = [0; 16];
		self.stack.stack = [0; 16];
		self.stack.sp = 0;
		self.input.keys = [0; 16];
		self.display.screen = [[false; 64]; 32];

		self.memory.load_font_into_memory();
	}

	pub fn load_rom(&mut self, rom: &[u8; 1536]) {
		self.memory.load_rom_into_memory(rom);
	}
}


#[cfg(test)]
mod tests {
	use super::Memory;
    use super::Chip8;
      
    #[test]
    fn test_default_state() {
    	// PC counter should default to 0x200:
    	let chip8 = Chip8::new_and_init();
    	assert_eq!(0x200, chip8.registers.pc);
    	// We should also already have the font in ram:
    	test_font_in_memory(&chip8.memory);
    }

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
    	let rom: [u8; 1536] = [0; 1536];
    	rom[0] = 0xFF;
    	rom[1] = 0xCC;

    	memory.load_rom_into_memory(&rom);
    	assert_eq!(memory.ram[0x200], 0xFF);
    	assert_eq!(memory.ram[0x201], 0xCC);
    }
}