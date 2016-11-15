extern crate rand;

use rand::{Rng, SeedableRng, XorShiftRng};

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

	// TODO maybe shouldn't be here? Doesn't really convey knowledge that the opcodes
	// are big-endian.
	#[inline]
	fn read_unsigned_short(&self, position: u16) -> u16 {
		let msb = self.ram[position as usize];
		let lsb = self.ram[(position + 1) as usize];
		return ((msb as u16) << 8) | (lsb as u16);		
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
	keys: [bool; 16],
}

// TODO need a flag when need to update (AFAIK two ops should affect that?)

struct Display {
	//  64x32, black or white
	screen: [[bool; 64]; 32],
	needs_draw: bool,
}

pub struct Chip8<R: Rng> {
	memory: Memory,
	registers: Registers,
	stack: Stack,
	input: Input,
	display: Display,
	rng: R,
}

// TODO docs
// Use a fast RNG as the default.
impl Chip8<XorShiftRng> {
	pub fn new_and_init() -> Chip8<XorShiftRng> {
		Chip8::new_and_init_with_rng(SeedableRng::from_seed(rand::random()))
	}
}

impl<R: Rng> Chip8<R> {		
	pub fn new_and_init_with_rng(r: R) -> Chip8<R> {
		let mut chip8 = Chip8 { 
			memory: Memory { ram: [0; 2048]},
			// Program counter starts at 0x200
			registers: Registers { pc: 0x200, index: 0, v: [0; 16]},
			stack: Stack { ret_addresses: [0; 16], sp: 0},
			input: Input { keys: [false; 16]},
			display: Display { screen: [[false; 64]; 32], needs_draw: false},
			rng: r,
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
		self.input.keys = [false; 16];
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
				let address = opcode_address(opcode);
				self.registers.pc = address;
			},
			0x2000...0x2FFF => {
				// Call a subroutine.
				// The return address should be after the opcode we're executing now.
				self.stack.ret_addresses[self.stack.sp as usize] = self.registers.pc + 2;
				self.stack.sp += 1;
				// Jump to the address in the opcode.
				let address = opcode_address(opcode);
				self.registers.pc = address;
			},
			0x3000...0x3FFF => {
				// Skip next instruction if register Vx is equal to last two bytes.
				let reg = opcode_register_index_second_octet(opcode);
				let operand = opcode_operand(opcode);
				if self.registers.v[reg] == operand {
					self.registers.pc += 4;
				} else {
					self.registers.pc += 2;
				}
			},
			0x4000...0x4FFF => {
				// Skip next instruction if register Vx is NOT equal to last two bytes.
				let reg = opcode_register_index_second_octet(opcode);
				let operand = opcode_operand(opcode);
				if self.registers.v[reg] != operand {
					self.registers.pc += 4;
				} else {
					self.registers.pc += 2;
				}
			},
			0x5000...0x5FFF => {
				// 0x5xy0
				// Skip next instruction if registers Vx and Vy are equal.
				// The last octet should be zero but we won't fail on that here.				
				let reg_x = opcode_register_index_second_octet(opcode);
				let reg_y = opcode_register_index_third_octet(opcode);
				
				if self.registers.v[reg_x] == self.registers.v[reg_y] {
					self.registers.pc += 4;
				} else {
					self.registers.pc += 2;
				}
			},
			0x6000...0x6FFF => {				
				// Store second byte in the specified register.
				let reg = opcode_register_index_second_octet(opcode);
				let operand = opcode_operand(opcode);
				self.registers.v[reg] = operand;
				self.registers.pc += 2;
			},
			0x7000...0x7FFF => {				
				// Add operand to register.
				let reg = opcode_register_index_second_octet(opcode);
				let operand = opcode_operand(opcode);
				self.registers.v[reg] = self.registers.v[reg].wrapping_add(operand);
				// TODO where else do I need to do wrapping adds?
				self.registers.pc += 2;
			},
			0x8000...0x8FFF => {
				let op = last_octet(opcode);
				let reg_x = opcode_register_index_second_octet(opcode);
				let reg_y = opcode_register_index_third_octet(opcode);				

				match op {
					0x0 => {
						self.registers.v[reg_x] = self.registers.v[reg_y];
					},
					0x1 => {
						self.registers.v[reg_x] |= self.registers.v[reg_y];	
					},
					0x2 => {
						self.registers.v[reg_x] &= self.registers.v[reg_y];		
					},
					0x3 => {
						self.registers.v[reg_x] ^= self.registers.v[reg_y];		
					},
					0x4 => {
						let v_x = self.registers.v[reg_x];
						let v_y = self.registers.v[reg_y];
						let (result, did_overflow) = v_x.overflowing_add(v_y);
						self.registers.v[reg_x] = result;
						self.registers.v[0xF] = if did_overflow { 1 } else { 0 };						
					},
					0x5 => {
						let v_x = self.registers.v[reg_x];
						let v_y = self.registers.v[reg_y];
						self.registers.v[0xF] = if v_x > v_y { 1 } else { 0 };
						self.registers.v[reg_x] = v_x.wrapping_sub(v_y);						
					},
					0x6 => {
						let v_x = self.registers.v[reg_x];						
						self.registers.v[0xF] = v_x & 0x1;
						self.registers.v[reg_x] = v_x >> 1;
					},
					0x7 => {
						let v_x = self.registers.v[reg_x];
						let v_y = self.registers.v[reg_y];
						self.registers.v[0xF] = if v_y > v_x { 1 } else { 0 };
						self.registers.v[reg_x] = v_y.wrapping_sub(v_x);
					},
					0xE => {
						let v_x = self.registers.v[reg_x];
						self.registers.v[0xF] = (v_x & 0x80) >> 7;
						self.registers.v[reg_x] = v_x << 1;
					},
					_ => {
						// Unknown opcode. Just skip over it.
					}
				}

				self.registers.pc += 2;	
			},
			0x9000...0x9FFF => {
				// 0x9xy0
				// Skip next instruction if registers Vx and Vy are NOT equal.
				// The last octet should be zero but we won't fail on that here.				
				let reg_x = opcode_register_index_second_octet(opcode);
				let reg_y = opcode_register_index_third_octet(opcode);
				
				if self.registers.v[reg_x] != self.registers.v[reg_y] {
					self.registers.pc += 4;
				} else {
					self.registers.pc += 2;
				}
			},
			0xA000...0xAFFF => {
				// Set index register to address.
				let address = opcode_address(opcode);
				self.registers.index = address;
				self.registers.pc += 2;
			},
			0xB000...0xBFFF => {
				// Jump to address + V0.
				let address = opcode_address(opcode);
				let computed_address = address.wrapping_add(self.registers.v[0x0] as u16);
				self.registers.pc = computed_address;
			},
			0xC000...0xCFFF => {
				// Ckxx - Takes a random number and ANDS it with the specified register.
				let reg = opcode_register_index_second_octet(opcode);
				let operand = opcode_operand(opcode);
				let next_random: u8 = self.rng.gen();
				let result = next_random & operand;
				self.registers.v[reg] = result;
			},
			0xD000...0xDFFF => {
				// Draw a sprite from memory at I at position (Vx, Vy),
				// and set v[0xF] in the case of a collision.
				let reg_x = opcode_register_index_second_octet(opcode);
				let reg_y = opcode_register_index_third_octet(opcode);
				let v_x = self.registers.v[reg_x];
				let v_y = self.registers.v[reg_y];
				let num_bytes = last_octet(opcode);
				let memory_base = self.registers.index;
				
				let mut did_overwrite = false;				
				
				// We'll draw in rows of 8 for num_bytes
				for sprite_y in 0..num_bytes {					
					let sprite_row = self.memory.ram[(memory_base + sprite_y as u16) as usize];					
					let screen_y = ((v_y + sprite_y) % 32) as usize;
					for sprite_x in 0..8 {						
						// Need to mask off the pixel since each byte represents a row of 8 pixels.
						let sprite_pixel = if (sprite_row & 0x80 >> sprite_x) > 0 { true } else { false };

						let screen_x = ((v_x + sprite_x) % 64) as usize;
						let current_pixel = self.display.screen[screen_y][screen_x];
						let new_pixel = sprite_pixel ^ current_pixel;

						if current_pixel == true {
							did_overwrite = true;
						}

						self.display.screen[screen_y][screen_x] = new_pixel;						
					}
				}

				self.registers.v[0xF] = if did_overwrite { 1 } else { 0 };
				self.display.needs_draw = true;
				self.registers.pc += 2;
			},
			0xE000...0xEFFF => {
				// Handle key input
				let second_byte = opcode_second_byte(opcode);
				let reg_x = opcode_register_index_second_octet(opcode);
				let v_x = self.registers.v[reg_x];
				let key = self.input.keys[v_x as usize];

				if second_byte == 0x9E && key == true {
					// Skip if key is pressed
					self.registers.pc += 4;
				} else if second_byte == 0xA1 && key == false {
					// Skip if key is NOT pressed
					self.registers.pc += 4;
				} else {
					// Only skip over our own opcode.
					self.registers.pc += 2;
				}	
			},
			_ => {
				// Unknown opcode, just skip over it.
				self.registers.pc += 2;
			}
		}
	}
}

// TODO names
// TODO maybe use trait / newtype

#[inline]
fn opcode_address(opcode: u16) -> u16 {
	opcode & 0x0FFF
}

#[inline]
fn opcode_register_index_second_octet(opcode: u16) -> usize {
	((opcode & 0x0F00) >> 8) as usize
}

#[inline]
fn opcode_register_index_third_octet(opcode: u16) -> usize {
	((opcode & 0x00F0) >> 4) as usize
}

#[inline]
fn opcode_operand(opcode: u16) -> u8 {
	(opcode & 0x00FF) as u8
}

#[inline]
fn opcode_second_byte(opcode: u16) -> u8 {
	opcode_operand(opcode)
}

#[inline]
fn last_octet(opcode: u16) -> u8 {
	(opcode & 0x000F) as u8
}

#[cfg(test)]
mod tests {
	use rand::{Rng, XorShiftRng};
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

    #[test]
    fn test_skip_when_equal_instruction_skips() {
    	let mut chip8 = Chip8::new_and_init();
    	chip8.memory.ram[chip8.registers.pc as usize] = 0x3A;
   		chip8.memory.ram[(chip8.registers.pc + 1) as usize] = 0xDD;	
   		// If register V[A] == 0xDD, then we should skip
   		chip8.registers.v[0xA] = 0xDD;
   		chip8.execute_next_opcode();
   		assert_eq!(0x204, chip8.registers.pc);
    }

    #[test]
    fn test_skip_when_equal_instruction_doesnt_skip() {
    	let mut chip8 = Chip8::new_and_init();
    	chip8.memory.ram[chip8.registers.pc as usize] = 0x3A;
   		chip8.memory.ram[(chip8.registers.pc + 1) as usize] = 0xDD;	
   		// Doesn't match default mem value of 0x00, should only increment program counter by two.
   		chip8.execute_next_opcode();
   		assert_eq!(0x202, chip8.registers.pc);
    }    

    #[test]
    fn test_skip_when_not_equal_instruction_skips() {
    	let mut chip8 = Chip8::new_and_init();
    	chip8.memory.ram[chip8.registers.pc as usize] = 0x4A;
   		chip8.memory.ram[(chip8.registers.pc + 1) as usize] = 0xDD;	
   		// Doesn't match default mem value of 0x00, should increment program counter by four.
   		chip8.execute_next_opcode();
   		assert_eq!(0x204, chip8.registers.pc);
    }

    #[test]
    fn test_skip_when_not_equal_instruction_doesnt_skip() {
    	let mut chip8 = Chip8::new_and_init();
    	chip8.memory.ram[chip8.registers.pc as usize] = 0x4A;
   		chip8.memory.ram[(chip8.registers.pc + 1) as usize] = 0xDD;	
   		// If register V[A] == 0xDD, then we should NOT skip
   		chip8.registers.v[0xA] = 0xDD;
   		chip8.execute_next_opcode();
   		assert_eq!(0x202, chip8.registers.pc);
    }    

    #[test]
    fn test_skip_when_two_registers_equal_skips() {
    	let mut chip8 = Chip8::new_and_init();
    	chip8.memory.ram[chip8.registers.pc as usize] = 0x5A;
   		chip8.memory.ram[(chip8.registers.pc + 1) as usize] = 0xB0;	
   		chip8.registers.v[0xA] = 0xBB;
   		chip8.registers.v[0xB] = 0xBB;
   		chip8.execute_next_opcode();
   		assert_eq!(0x204, chip8.registers.pc);
    }  
    #[test]
    fn test_skip_when_two_registers_not_equal_doesnt_skip() {
    	let mut chip8 = Chip8::new_and_init();
    	chip8.memory.ram[chip8.registers.pc as usize] = 0x5A;
   		chip8.memory.ram[(chip8.registers.pc + 1) as usize] = 0xB0;	
   		chip8.registers.v[0xA] = 0xBB;
   		chip8.registers.v[0xB] = 0xCC;
   		chip8.execute_next_opcode();
   		assert_eq!(0x202, chip8.registers.pc);
    }

    #[test]
    fn test_store() {
   		let mut chip8 = Chip8::new_and_init();
   		chip8.memory.ram[chip8.registers.pc as usize] = 0x62;
   		chip8.memory.ram[(chip8.registers.pc + 1) as usize] = 0xAB;
   		chip8.execute_next_opcode();   		
   		assert_eq!(0xAB, chip8.registers.v[2]);
    }

    #[test]
    fn test_add() {
   		let mut chip8 = Chip8::new_and_init();
   		chip8.memory.ram[chip8.registers.pc as usize] = 0x7A;
   		chip8.memory.ram[(chip8.registers.pc + 1) as usize] = 0x0A;
   		chip8.registers.v[0xA] = 0xA;
   		chip8.execute_next_opcode();   		
   		assert_eq!(0x14, chip8.registers.v[0xA]);
    }

    #[test]
    fn test_add_wraps() {
   		let mut chip8 = Chip8::new_and_init();
   		chip8.memory.ram[chip8.registers.pc as usize] = 0x7A;
   		chip8.memory.ram[(chip8.registers.pc + 1) as usize] = 0x96;
   		chip8.registers.v[0xA] = 0x80;
   		chip8.execute_next_opcode();   		
   		assert_eq!(0x16, chip8.registers.v[0xA]);
    }

    #[test]
    fn test_store_one_reg_in_another() {
   		let mut chip8 = Chip8::new_and_init();
   		chip8.memory.ram[chip8.registers.pc as usize] = 0x80;
   		chip8.memory.ram[(chip8.registers.pc + 1) as usize] = 0x10;
   		chip8.registers.v[0x0] = 0x20;
   		chip8.registers.v[0x1] = 0x30;
   		chip8.execute_next_opcode();   		
   		assert_eq!(0x30, chip8.registers.v[0x0]);
    }

    #[test]
    fn test_or() {
   		let mut chip8 = Chip8::new_and_init();
   		chip8.memory.ram[chip8.registers.pc as usize] = 0x8A;
   		chip8.memory.ram[(chip8.registers.pc + 1) as usize] = 0xB1;
   		chip8.registers.v[0xA] = 0x92;
   		chip8.registers.v[0xB] = 0x32;
   		chip8.execute_next_opcode();   		
   		assert_eq!(0xB2, chip8.registers.v[0xA]);
    }

    #[test]
    fn test_and() {
   		let mut chip8 = Chip8::new_and_init();
   		chip8.memory.ram[chip8.registers.pc as usize] = 0x8A;
   		chip8.memory.ram[(chip8.registers.pc + 1) as usize] = 0xB2;
   		chip8.registers.v[0xA] = 0x92;
   		chip8.registers.v[0xB] = 0x32;
   		chip8.execute_next_opcode();   		
   		assert_eq!(0x12, chip8.registers.v[0xA]);
    }

    #[test]
    fn test_xor() {
   		let mut chip8 = Chip8::new_and_init();
   		chip8.memory.ram[chip8.registers.pc as usize] = 0x8A;
   		chip8.memory.ram[(chip8.registers.pc + 1) as usize] = 0xB3;
   		chip8.registers.v[0xA] = 0x92;
   		chip8.registers.v[0xB] = 0x32;
   		chip8.execute_next_opcode();   		
   		assert_eq!(0xA0, chip8.registers.v[0xA]);
    }

    #[test]
    fn test_add_two_regs() {
   		let mut chip8 = Chip8::new_and_init();
   		chip8.memory.ram[chip8.registers.pc as usize] = 0x8A;
   		chip8.memory.ram[(chip8.registers.pc + 1) as usize] = 0xB4;
   		chip8.registers.v[0xA] = 0x92;
   		chip8.registers.v[0xB] = 0x32;
   		chip8.execute_next_opcode();   		
   		assert_eq!(0xC4, chip8.registers.v[0xA]);
   		assert_eq!(0x0, chip8.registers.v[0xF]);
    }

    #[test]
    fn test_add_two_regs_with_overflow() {
   		let mut chip8 = Chip8::new_and_init();
   		chip8.memory.ram[chip8.registers.pc as usize] = 0x8A;
   		chip8.memory.ram[(chip8.registers.pc + 1) as usize] = 0xB4;
   		chip8.registers.v[0xA] = 0x92;
   		chip8.registers.v[0xB] = 0x92;
   		chip8.execute_next_opcode();   		
   		assert_eq!(0x24, chip8.registers.v[0xA]);
   		assert_eq!(0x1, chip8.registers.v[0xF]);
    }

    #[test]
    fn test_vx_sub_vy_without_borrow() {
   		let mut chip8 = Chip8::new_and_init();
   		chip8.memory.ram[chip8.registers.pc as usize] = 0x8A;
   		chip8.memory.ram[(chip8.registers.pc + 1) as usize] = 0xB5;
   		chip8.registers.v[0xA] = 0x92;
   		chip8.registers.v[0xB] = 0x32;
   		chip8.execute_next_opcode();   		
   		assert_eq!(0x60, chip8.registers.v[0xA]);
   		assert_eq!(0x1, chip8.registers.v[0xF]);
    }

    #[test]
    fn test_vx_sub_vy_with_borrow() {
   		let mut chip8 = Chip8::new_and_init();
   		chip8.memory.ram[chip8.registers.pc as usize] = 0x8A;
   		chip8.memory.ram[(chip8.registers.pc + 1) as usize] = 0xB5;
   		chip8.registers.v[0xA] = 0x32;
   		chip8.registers.v[0xB] = 0x92;
   		chip8.execute_next_opcode();   		
   		assert_eq!(0xA0, chip8.registers.v[0xA]);
   		assert_eq!(0x0, chip8.registers.v[0xF]);
    }

    #[test]
    fn test_vx_shr_by_one() {
   		let mut chip8 = Chip8::new_and_init();
   		chip8.memory.ram[chip8.registers.pc as usize] = 0x8A;
   		chip8.memory.ram[(chip8.registers.pc + 1) as usize] = 0xB6;
   		chip8.registers.v[0xA] = 0b11001010;   		
   		chip8.execute_next_opcode();   		
   		assert_eq!(0b01100101, chip8.registers.v[0xA]);
   		assert_eq!(0x0, chip8.registers.v[0xF]);
    }

    #[test]
    fn test_vx_shr_by_one_with_truncation() {
   		let mut chip8 = Chip8::new_and_init();
   		chip8.memory.ram[chip8.registers.pc as usize] = 0x8A;
   		chip8.memory.ram[(chip8.registers.pc + 1) as usize] = 0xB6;
   		chip8.registers.v[0xA] = 0b11001011;   		
   		chip8.execute_next_opcode();   		
   		assert_eq!(0b01100101, chip8.registers.v[0xA]);
   		assert_eq!(0x1, chip8.registers.v[0xF]);
    }

    #[test]
    fn test_vy_sub_vx_without_borrow() {
   		let mut chip8 = Chip8::new_and_init();
   		chip8.memory.ram[chip8.registers.pc as usize] = 0x8A;
   		chip8.memory.ram[(chip8.registers.pc + 1) as usize] = 0xB7;
   		chip8.registers.v[0xA] = 0x32;
   		chip8.registers.v[0xB] = 0x92;
   		chip8.execute_next_opcode();   		
   		assert_eq!(0x60, chip8.registers.v[0xA]);
   		assert_eq!(0x1, chip8.registers.v[0xF]);
    }

    #[test]
    fn test_vy_sub_vx_with_borrow() {
   		let mut chip8 = Chip8::new_and_init();
   		chip8.memory.ram[chip8.registers.pc as usize] = 0x8A;
   		chip8.memory.ram[(chip8.registers.pc + 1) as usize] = 0xB7;
   		chip8.registers.v[0xA] = 0x92;
   		chip8.registers.v[0xB] = 0x32;
   		chip8.execute_next_opcode();   		
   		assert_eq!(0xA0, chip8.registers.v[0xA]);
   		assert_eq!(0x0, chip8.registers.v[0xF]);
    }

    #[test]
    fn test_vx_shl_by_one() {
   		let mut chip8 = Chip8::new_and_init();
   		chip8.memory.ram[chip8.registers.pc as usize] = 0x8A;
   		chip8.memory.ram[(chip8.registers.pc + 1) as usize] = 0xBE;
   		chip8.registers.v[0xA] = 0b01001010;   		
   		chip8.execute_next_opcode();   		
   		assert_eq!(0b10010100, chip8.registers.v[0xA]);
   		assert_eq!(0x0, chip8.registers.v[0xF]);
    }

    #[test]
    fn test_vx_shl_by_one_with_overflow() {
   		let mut chip8 = Chip8::new_and_init();
   		chip8.memory.ram[chip8.registers.pc as usize] = 0x8A;
   		chip8.memory.ram[(chip8.registers.pc + 1) as usize] = 0xBE;
   		chip8.registers.v[0xA] = 0b11001010;   		
   		chip8.execute_next_opcode();   		
   		assert_eq!(0b10010100, chip8.registers.v[0xA]);
   		assert_eq!(0x1, chip8.registers.v[0xF]);
    }

    #[test]
    fn test_9xy0_skip_when_two_registers_not_equal() {
    	let mut chip8 = Chip8::new_and_init();
    	chip8.memory.ram[chip8.registers.pc as usize] = 0x9A;
   		chip8.memory.ram[(chip8.registers.pc + 1) as usize] = 0xB0;	
   		chip8.registers.v[0xA] = 0xBB;
   		chip8.registers.v[0xB] = 0xCC;
   		chip8.execute_next_opcode();
   		assert_eq!(0x204, chip8.registers.pc);
    }  
    #[test]
    fn test_9xy0_dont_skip_when_two_registers_equal() {
    	let mut chip8 = Chip8::new_and_init();
    	chip8.memory.ram[chip8.registers.pc as usize] = 0x9A;
   		chip8.memory.ram[(chip8.registers.pc + 1) as usize] = 0xB0;	
   		chip8.registers.v[0xA] = 0xBB;
   		chip8.registers.v[0xB] = 0xBB;
   		chip8.execute_next_opcode();
   		assert_eq!(0x202, chip8.registers.pc);
    }

    #[test]
    fn test_annn_set_index_register_to_address() {
   		let mut chip8 = Chip8::new_and_init();
   		chip8.memory.ram[chip8.registers.pc as usize] = 0xAE;
   		chip8.memory.ram[(chip8.registers.pc + 1) as usize] = 0xEE;
   		chip8.execute_next_opcode();

   		assert_eq!(0xEEE, chip8.registers.index);
    }

    #[test]
    fn test_bnnn_computed_jump() {
   		let mut chip8 = Chip8::new_and_init();
   		chip8.memory.ram[chip8.registers.pc as usize] = 0xBE;
   		chip8.memory.ram[(chip8.registers.pc + 1) as usize] = 0xEE;
   		chip8.registers.v[0x0] = 0xBB;
   		chip8.execute_next_opcode();

   		assert_eq!(0xFA9, chip8.registers.pc);
    }

    #[test]
    fn test_dxkk_random_byte_anded_and_stored() {
    	let first_rng = XorShiftRng::new_unseeded(); 
    	let mut second_rng = XorShiftRng::new_unseeded();
    	let mut chip8 = Chip8::new_and_init_with_rng(first_rng);
    	chip8.memory.ram[chip8.registers.pc as usize] = 0xCA;
   		chip8.memory.ram[(chip8.registers.pc + 1) as usize] = 0xAA;
   		chip8.execute_next_opcode();

   		let result = chip8.registers.v[0xA];

   		// Compute a result and see if it matches
   		let next_random: u8 = second_rng.gen();
   		let computed_result = next_random & 0xAA;
   		assert_eq!(result, computed_result);
    }

    #[test]
    fn test_dxyn_blit_sprite() {
    	let mut chip8 = Chip8::new_and_init();
   		
   		// Blit "0" from the font which has dimensions of 8x5
   		chip8.memory.ram[chip8.registers.pc as usize] = 0xDA;
   		chip8.memory.ram[(chip8.registers.pc + 1) as usize] = 0xB5;	   	   		
   		chip8.registers.v[0xA] = 0x0;
   		chip8.registers.v[0xB] = 0x0;
   		chip8.registers.index = 0x50;
   		chip8.execute_next_opcode();

   		// Zero
		// 0b11110000,
		// 0b10010000,
		// 0b10010000,
		// 0b10010000,
		// 0b11110000,

   		// Check the display

   		// First row and fifth row
   		for y in [0, 4].iter() { 
   			let y = *y;  			
	   		assert_eq!(true,  chip8.display.screen[y][0]);
	   		assert_eq!(true,  chip8.display.screen[y][1]);
	   		assert_eq!(true,  chip8.display.screen[y][2]);
	   		assert_eq!(true,  chip8.display.screen[y][3]);
	   		assert_eq!(false, chip8.display.screen[y][4]);
	   		assert_eq!(false, chip8.display.screen[y][5]);
	   		assert_eq!(false, chip8.display.screen[y][6]);
	   		assert_eq!(false, chip8.display.screen[y][7]);
	   	}

   		// Second through fourth rows

   		for y in 1..4 {
	   		assert_eq!(true,  chip8.display.screen[y][0]);
	   		assert_eq!(false, chip8.display.screen[y][1]);
	   		assert_eq!(false, chip8.display.screen[y][2]);
	   		assert_eq!(true,  chip8.display.screen[y][3]);
	   		assert_eq!(false, chip8.display.screen[y][4]);
	   		assert_eq!(false, chip8.display.screen[y][5]);
	   		assert_eq!(false, chip8.display.screen[y][6]);
	   		assert_eq!(false, chip8.display.screen[y][7]);
	   	}

	   	// Nothing was overwritten
	   	assert_eq!(0x0, chip8.registers.v[0xF]);

	   	// Draw over the same position again
	   	chip8.memory.ram[chip8.registers.pc as usize] = 0xDA;
   		chip8.memory.ram[(chip8.registers.pc + 1) as usize] = 0xB5;
   		chip8.execute_next_opcode();

   		// Now everything should be cleared because of XOR.
   		for y in 0..5 {
   			for x in 0..8 {
   				assert_eq!(false, chip8.display.screen[y][x]);
   			}
   		}

   		// The zero was cleared.   		
	   	assert_eq!(0x1, chip8.registers.v[0xF]);
    }

    #[test]
    fn test_dxyn_blit_sprite_wraps_around_edge_of_screen() {
    	let mut chip8 = Chip8::new_and_init();
   		
   		// This time we're going to blit a solid 8x8 block at (60, 28).
   		chip8.memory.ram[chip8.registers.pc as usize] = 0xDA;
   		chip8.memory.ram[(chip8.registers.pc + 1) as usize] = 0xB8;
   		chip8.registers.v[0xA] = 60;
   		chip8.registers.v[0xB] = 28;
   		chip8.registers.index = 0x400;

   		for y in 0x400..0x408 {
   			chip8.memory.ram[y] = 0xFF;
   		}

   		chip8.execute_next_opcode();

   		// Should have wrapped around
   		for y in 28..32 {
   			let y = y % 32;

   			for x in 60..68 {
   				let x = x % 64;

   				assert_eq!(true, chip8.display.screen[y][x]);
   			}
   		}

	   	// Nothing was overwritten
	   	assert_eq!(0x0, chip8.registers.v[0xF]);
    }

    #[test]
    fn test_ex9e_skips_if_key_pressed() {
    	let mut chip8 = Chip8::new_and_init();   		
   		chip8.memory.ram[chip8.registers.pc as usize] = 0xEA;
   		chip8.memory.ram[(chip8.registers.pc + 1) as usize] = 0x9E;
   		chip8.registers.v[0xA] = 0xB;
   		chip8.input.keys[0xB] = true;
   		chip8.execute_next_opcode();
		assert_eq!(0x204, chip8.registers.pc);
    }

    #[test]
    fn test_ex9e_doesnt_skip_if_key_not_pressed() {
    	let mut chip8 = Chip8::new_and_init();   		
   		chip8.memory.ram[chip8.registers.pc as usize] = 0xEA;
   		chip8.memory.ram[(chip8.registers.pc + 1) as usize] = 0x9E;
   		chip8.registers.v[0xA] = 0xB;
   		chip8.input.keys[0xB] = false;
   		chip8.execute_next_opcode();
		assert_eq!(0x202, chip8.registers.pc);
    }

    #[test]
    fn test_exa1_skips_if_key_not_pressed() {
    	let mut chip8 = Chip8::new_and_init();   		
   		chip8.memory.ram[chip8.registers.pc as usize] = 0xEA;
   		chip8.memory.ram[(chip8.registers.pc + 1) as usize] = 0xA1;
   		chip8.registers.v[0xA] = 0xB;
   		chip8.input.keys[0xB] = false;
   		chip8.execute_next_opcode();
		assert_eq!(0x204, chip8.registers.pc);
    }

    #[test]
    fn test_exa1_doesnt_skip_if_key_pressed() {
    	let mut chip8 = Chip8::new_and_init();   		
   		chip8.memory.ram[chip8.registers.pc as usize] = 0xEA;
   		chip8.memory.ram[(chip8.registers.pc + 1) as usize] = 0xA1;
   		chip8.registers.v[0xA] = 0xB;
   		chip8.input.keys[0xB] = true;
   		chip8.execute_next_opcode();
		assert_eq!(0x202, chip8.registers.pc);
    }

    // TODO clean up the tests using helper functions
    // TODO be more specific about test names
}