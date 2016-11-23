"use strict";

// CHIP-8 code //

function Memory() {
    this.ram = new Uint8Array(4096);

    this.loadFontIntoMemory = function() {
        var chip8_fontset =
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
        var i = 0;
        var o = 0x50;
        while (o < 0xA0) {
            this.ram[o] = chip8_fontset[i];
            ++i; ++o;
        }
    }

    this.loadRomIntoMemory = function(romFile) {
        var i = 0;
        var o = 0x200;
        while (i < romFile.length) {
            this.ram[o] = romFile.charCodeAt(i) & 0xFF;
            ++i; ++o;
        }
    }
}

function Registers() {
    // Program counter (u16)
    this.pc = 0x200 & 0xFFFF;
    // Index register (u16)
    this.i = 0 & 0xFFFF;
    // 15 general-purpose registers + 1 carry flag
    this.v = new Uint8Array(16);
}

function Stack() {
    // 16 stack addresses
    this.retAddresses = new Uint16Array(16);
    // Stack pointer (u8)
    this.sp = 0 & 0xFF;
}

function Input() {
    // 16 keys
    this.keys = new Uint8Array(16);
}

function Display() {
    // 64x32; black or white (bool)
    this.screen = new Array(32);
    for (var y = 0; y < 32; ++y) {
        this.screen[y] = new Uint8Array(64);
    }
}

function Timers() {
    this.delayTimer = 0 & 0xFF;    // u8
    this.soundTimer = 0 & 0xFF;    // u8
}

function Chip8() {
    this.memory = new Memory;
    this.registers = new Registers;
    this.stack = new Stack;
    this.input = new Input;
    this.display = new Display;
    this.timers = new Timers;

    this.loadRom = function(rom) {
        this.memory = new Memory;
        this.registers = new Registers;
        this.stack = new Stack;
        this.input = new Input;
        this.display = new Display;
        this.timers = new Timers;
        this.memory.loadFontIntoMemory();
        this.memory.loadRomIntoMemory(rom);        
    }

    this.isRomLoaded = function() {
        return this.memory.ram[0x200] != 0 || this.memory.ram[0x201] != 0;
    }

    this.readNextOpcode = function() {
        var msb = this.memory.ram[this.registers.pc];
        var lsb = this.memory.ram[this.registers.pc + 1];
        return ((msb << 8) | lsb) & 0xFFFF;
    }

    this.updateTimers = function() {
        if (this.timers.delayTimer > 0) {
            this.timers.delayTimer -= 1;
        }
        if (this.timers.soundTimer > 0) {
            this.timers.soundTimer -= 1;
        }
    }

    this.setKeyState = function(keyIndex, isPressed) {
        this.input.keys[keyIndex] = isPressed;
    }

    this.shouldPlaySound = function() {
        return this.timers.soundTimer > 0;
    }

    this.executeNextOpcode = function() {
        var opcode = this.readNextOpcode();
        // Break down the opcode into components -- not all used for all instructions.
        var address = opcode & 0x0FFF;
        var regX = (opcode & 0x0F00) >>> 8;
        var regY = (opcode & 0x00F0) >>> 4;
        var operand = opcode & 0x00FF;    

        this.registers.pc = (this.registers.pc + 2) & 0xFFFF;
        switch (opcode >>> 12) {
            case 0x0:
                switch (opcode & 0x0FFF) {
                    case 0x0E0:
                        // Clear the screen.
                        for (var y = 0; y < 32; ++y) {
                            for (var x = 0; x < 64; ++x) {
                                this.display.screen[y][x] = 0;
                                if (this.drawFunction) {
                                    this.drawFunction(x, y, 0);
                                }
                            }
                        }
                        break;
                    case 0x0EE:
                        // Return from a subroutine.
                        this.registers.pc = this.stack.retAddresses[--this.stack.sp];
                        break;
                }
                break;
            case 0x1:
                // Jump.
                this.registers.pc = address;
                break;
            case 0x2:
                // Call a subroutine.
                this.stack.retAddresses[this.stack.sp++] = this.registers.pc;
                this.registers.pc = address;
                break;
            case 0x3:
                // Skip next instruction if register Vx is equal to last two bytes of opcode.
                if (this.registers.v[regX] == operand) {
                    this.registers.pc = (this.registers.pc + 2) & 0xFFFF;
                }
                break;
            case 0x4:
                // Skip next instruction if register Vx is NOT equal to last two bytes of opcode.
                if (this.registers.v[regX] != operand) {
                    this.registers.pc = (this.registers.pc + 2) & 0xFFFF;
                }
                break;
            case 0x5:
                // 0x5xy0: Skip next instruction if registers Vx and Vy are equal.
                // The last octet should be zero but we won't fail on that here.
                if (this.registers.v[regX] == this.registers.v[regY]) {
                    this.registers.pc = (this.registers.pc + 2) & 0xFFFF;
                }
                break;
            case 0x6:
                // Store second byte of opcode in the specified register.
                this.registers.v[regX] = operand;
                break;
            case 0x7:
                // Add operand to register.
                this.registers.v[regX] = (this.registers.v[regX] + operand) & 0xFF;
                break;
            case 0x8:
            {
                // Not used for all opcodes.
                var vX = this.registers.v[regX];
                var vY = this.registers.v[regY];

                switch (opcode & 0x00F) {
                    case 0x0:
                        // Store Vy in Vx.
                        this.registers.v[regX] = this.registers.v[regY];
                        break;
                    case 0x1:
                        // Set Vx = Vx OR Vy.
                        this.registers.v[regX] = (this.registers.v[regX] | this.registers.v[regY]) & 0xFF;
                        break;
                    case 0x2:
                        // Set Vx = Vx AND Vy.
                        this.registers.v[regX] = (this.registers.v[regX] & this.registers.v[regY]) & 0xFF;
                        break;
                    case 0x3:
                        // Set Vx = Vx XOR Vy.
                        this.registers.v[regX] = (this.registers.v[regX] ^ this.registers.v[regY]) & 0xFF;
                        break;
                    case 0x4:
                    {
                        // Set Vx = Vx + Vy, with VF = carry.
                        var result = vX + vY;
                        this.registers.v[0xF] = result > 255 ? 1 : 0;
                        this.registers.v[regX] = result & 0xFF;
                        break;
                    }
                    case 0x5:
                        // Set Vx = Vx - Vy, with VF = NO borrow happened.
                        this.registers.v[0xF] = vX > vY ? 1 : 0;
                        this.registers.v[regX] = (vX - vY) & 0xFF;
                        break;
                    case 0x6:
                        // Set Vx = Vx shifted right by 1, with VF = LSB of Vx equals 1.
                        this.registers.v[0xF] = vX & 0x1;
                        this.registers.v[regX] = vX >>> 1;
                        break;
                    case 0x7:
                        // Set Vx = Vy - Vx, with VF = NO borrow happened.
                        this.registers.v[0xF] = vY > vX ? 1 : 0;
                        this.registers.v[regX] = (vY - vX) & 0xFF;
                        break;
                    case 0xE:
                        // Set Vx = Vx shifted left by 1, with VF = MSB of Vx equals 1.
                        this.registers.v[0xF] = (vX & 0x80) >>> 7;
                        this.registers.v[regX] = (vX << 1) & 0xFF;
                        break;
                }
                break;
            }
            case 0x9:
                // 0x9xy0: Skip next instruction if registers Vx and Vy are NOT equal.
                // The last octet should be zero but we won't fail on that here.
                if (this.registers.v[regX] != this.registers.v[regY]) {
                    this.registers.pc = (this.registers.pc + 2) & 0xFFFF;
                }
                break;
            case 0xA:
                // Set index register to address.
                this.registers.i = address;
                break;
            case 0xB:
                // Jump to address + V0.
                this.registers.pc = (address + this.registers.v[0x0]) & 0xFFFF;
                break;
            case 0xC:
            {
                // Ckxx: Takes a random number and ANDS it with the specified register.
                var random = (Math.random() * 256) & 0xFF;
                this.registers.v[regX] = random & operand & 0xFF;
                break;
            }
            case 0xD:
            {
                // Draw a sprite from memory at I at position (Vx, Vy),
                // and set v[0xF] in the case of a collision.
                var vX = this.registers.v[regX];
                var vY = this.registers.v[regY];
                var numBytes = opcode & 0x000F;
                var memoryBase = this.registers.i;
                var didOverwrite = false;

                // We'll draw in rows of 8 for num_bytes
                for (var spriteY = 0; spriteY < numBytes; ++spriteY) {
                    var spriteRow = this.memory.ram[memoryBase + spriteY];
                    var screenY = (vY + spriteY) % 32;
                    for (var spriteX = 0; spriteX < 8; ++spriteX) {
                        // Need to mask off the pixel since each byte represents a row of 8 pixels.
                        var spritePixel = (spriteRow & (0x80 >>> spriteX)) > 0 ? 1 : 0;
                        var screenX = (vX + spriteX) % 64;

                        var currentPixel = this.display.screen[screenY][screenX];
                        var newPixel = (spritePixel ^ currentPixel) & 0x1;

                        if (currentPixel > 0 && spritePixel > 0) {
                            didOverwrite = true;
                        }

                        this.display.screen[screenY][screenX] = newPixel;
                        if (this.drawFunction) {
                            this.drawFunction(screenX, screenY, newPixel);
                        }
                    }
                }

                this.registers.v[0xF] = didOverwrite ? 1 : 0;
                break;
            }
            case 0xE:
            {
                // Handle key input
                var vX = this.registers.v[regX];
                var key = this.input.keys[vX];

                if (operand == 0x9E && key > 0) {
                    // Skip if key is pressed
                    this.registers.pc = (this.registers.pc + 2) & 0xFFFF;
                } else if (operand == 0xA1 && key == 0) {
                    // Skip if key is NOT pressed
                    this.registers.pc = (this.registers.pc + 2) & 0xFFFF;
                }
                break;
            }
            case 0xF:
            {
                var vX = this.registers.v[regX];

                switch (operand) {
                    case 0x07:
                        // Delay timer value.
                        this.registers.v[regX] = this.timers.delayTimer & 0xFF;
                        break;
                    case 0x0A:
                        // Check if key pressed; only continue execution if pressed.
                        this.registers.pc = (this.registers.pc - 2) & 0xFFFF;
                        for (var i = 0; i < this.input.keys.length; ++i) {
                            var key = this.input.keys[i];
                            if (key > 0) {
                                this.registers.v[regX] = i & 0xFF;
                                this.registers.pc = (this.registers.pc + 2) & 0xFFFF;
                                break;
                            }
                        }
                        break;
                    case 0x15:
                        // Set delay timer.
                        this.timers.delayTimer = this.registers.v[regX];
                        break;
                    case 0x18:
                        // Set sound timer.
                        this.timers.soundTimer = this.registers.v[regX];
                        break;
                    case 0x1E:
                    {
                        // Increment index.
                        var newIndex = this.registers.i + this.registers.v[regX];
                        // Undocumented feature, according to Wiki. Should wrap around 0xFFF.
                        var didOverflow = newIndex > 0xFFF;
                        newIndex = newIndex & 0xFFF;
                        this.registers.i = newIndex;
                        this.registers.v[0xF] = didOverflow ? 1 : 0;
                        break;
                    }
                    case 0x29:
                    {
                        // Location of sprite.
                        var spriteIndex = this.registers.v[regX];
                        var spriteLocation = (0x50 + (5 * spriteIndex)) & 0xFFFF;
                        this.registers.i = spriteLocation;
                        break;
                    }
                    case 0x33:
                    {
                        // Converts register to decimal format in memory at
                        // location pointed to by index.
                        var index = this.registers.i;

                        // Hundredth's digit.
                        this.memory.ram[index] = (vX / 100) & 0xFF;
                        // Tenth's digit.
                        this.memory.ram[index + 1] = ((vX / 10) % 10) & 0xFF;
                        // One's digit.
                        this.memory.ram[index + 2] = ((vX % 100) % 10) & 0xFF;
                        break;
                    }
                    case 0x55:
                    {
                        // Spill registers from 0 to x to memory, inclusive.
                        var i = 0;
                        var o = this.registers.i;
                        while (i <= regX) {
                            this.memory.ram[o] = this.registers.v[i];
                            i = (i + 1) & 0xFF;
                            o = (o + 1) & 0xFFFF;
                        }
                        break;
                    }
                    case 0x65:
                    {
                        // Load memory into registers from 0 to x, inclusive.
                        var i = this.registers.i;
                        var o = 0;
                        while (o <= regX) {
                            this.registers.v[o] = this.memory.ram[i];
                            i = (i + 1) & 0xFFFF;
                            o = (o + 1) & 0xFF;
                        }
                        break;
                    }
                }
                break;
            }
        }
    }

    this.memory.loadFontIntoMemory();
}

// Main driver code //

// Graphics
var canvas = document.getElementById("chip8_display");
var ctx = canvas.getContext("2d");

// Audio
var audioCtx = new (window.AudioContext || window.webkitAudioContext)();
var gain = audioCtx.createGain();
gain.gain.value = 0.25;
gain.connect(audioCtx.destination);
var oscillator = null;

// Document elements
var select = document.getElementById("chip8_rom_select");
var pauseButton = document.getElementById("chip8_pause");
var resumeButton = document.getElementById("chip8_resume");

// Interpreter state
var chip8 = new Chip8();
var isAnimating = false;

// Functions
function beep() {
    if (!oscillator) {
        oscillator = audioCtx.createOscillator();
        oscillator.connect(gain);
        oscillator.type = 'square';
        oscillator.frequency.value = 125;
        oscillator.connect(gain);
        oscillator.start();
    }
}

function stopBeep() {
    if (oscillator) {
        oscillator.stop();
        oscillator.disconnect();
        oscillator = null;
    }
}

function startAnimating() {
    if (!isAnimating && chip8.isRomLoaded()) {
        isAnimating = true;
        window.requestAnimationFrame(stepEmu);
    }
}

function stopAnimating() {
    isAnimating = false;
}

// Note: The execution rate and timers are frame-rate dependent.
// An improvement would be to measure elapsed time and execute based on that.
function stepEmu(timestamp) {
    if (isAnimating) {
        // Execute 10 opcodes every frame
        for (var i = 0; i < 10; ++i) {
            chip8.executeNextOpcode();
        }
        // Timers should execute at 60Hz.
        chip8.updateTimers();
        // Update sound state
        if (chip8.shouldPlaySound()) {
            beep();
        } else {
            stopBeep();
        }

        window.requestAnimationFrame(stepEmu);
    }
}

function forwardKeyPressToChip8(keyChar, isPressed) {
    switch(keyChar) {
        case "1": chip8.input.keys[0x1] = isPressed; break;
        case "2": chip8.input.keys[0x2] = isPressed; break;
        case "3": chip8.input.keys[0x3] = isPressed; break;
        case "4": chip8.input.keys[0xC] = isPressed; break;

        case "Q": chip8.input.keys[0x4] = isPressed; break;
        case "W": chip8.input.keys[0x5] = isPressed; break;
        case "E": chip8.input.keys[0x6] = isPressed; break;
        case "R": chip8.input.keys[0xD] = isPressed; break;

        case "A": chip8.input.keys[0x7] = isPressed; break;
        case "S": chip8.input.keys[0x8] = isPressed; break;
        case "D": chip8.input.keys[0x9] = isPressed; break;
        case "F": chip8.input.keys[0xE] = isPressed; break;

        case "Z": chip8.input.keys[0xA] = isPressed; break;
        case "X": chip8.input.keys[0x0] = isPressed; break;
        case "C": chip8.input.keys[0xB] = isPressed; break;
        case "V": chip8.input.keys[0xF] = isPressed; break;
    }
}

function forwardTouchToChip8(event, id, isPressed) {
    var forwarded = false;
    switch (id) {
        case "chip8_key_zero"   : chip8.input.keys[0x0] = isPressed; forwarded = true; break;
        case "chip8_key_one"    : chip8.input.keys[0x1] = isPressed; forwarded = true; break;
        case "chip8_key_two"    : chip8.input.keys[0x2] = isPressed; forwarded = true; break;
        case "chip8_key_three"  : chip8.input.keys[0x3] = isPressed; forwarded = true; break;

        case "chip8_key_four"   : chip8.input.keys[0x4] = isPressed; forwarded = true; break;
        case "chip8_key_five"   : chip8.input.keys[0x5] = isPressed; forwarded = true; break;
        case "chip8_key_six"    : chip8.input.keys[0x6] = isPressed; forwarded = true; break;
        case "chip8_key_seven"  : chip8.input.keys[0x7] = isPressed; forwarded = true; break;

        case "chip8_key_eight"  : chip8.input.keys[0x8] = isPressed; forwarded = true; break;
        case "chip8_key_nine"   : chip8.input.keys[0x9] = isPressed; forwarded = true; break;
        case "chip8_key_a"      : chip8.input.keys[0xA] = isPressed; forwarded = true; break;
        case "chip8_key_b"      : chip8.input.keys[0xB] = isPressed; forwarded = true; break;

        case "chip8_key_c"      : chip8.input.keys[0xC] = isPressed; forwarded = true; break;
        case "chip8_key_d"      : chip8.input.keys[0xD] = isPressed; forwarded = true; break;
        case "chip8_key_e"      : chip8.input.keys[0xE] = isPressed; forwarded = true; break;
        case "chip8_key_f"      : chip8.input.keys[0xF] = isPressed; forwarded = true; break;
    }
    if (forwarded) {
        event.preventDefault();
    }
}

function handleKeyPress(event) {
    var key = event.which;
    var keyChar = String.fromCharCode(key);
    var isPressed = event.type == 'keydown' ? 1 : 0;
    forwardKeyPressToChip8(keyChar, isPressed);
}

function handleMouseEvent(event) {
    var isPressed = event.type == 'mousedown' ? 1 : 0;
    forwardTouchToChip8(event.target.id, isPressed);
}

function handleTouchEvent(event) {    
    var isPressed = event.type == "touchstart" ? 1 : 0;
    forwardTouchToChip8(event, event.target.id, isPressed);    
}

// Setup CHIP-8 callbacks
chip8.drawFunction = function(x, y, is_bright) {
    if (is_bright > 0) {
        ctx.fillStyle = "rgb(255, 255, 224)";
    } else {
        ctx.fillStyle = "rgb(0, 0, 0)";
    }
    ctx.fillRect(x * 8, y * 8, 8, 8);
}

// Setup event listeners
pauseButton.addEventListener("click", function () {
    stopAnimating();
    stopBeep();
});

resumeButton.addEventListener("click", function () {
    startAnimating();
});

document.onkeyup = handleKeyPress;
document.onkeydown = handleKeyPress;

document.onmousedown = handleMouseEvent;
document.onmouseup = handleMouseEvent;

document.ontouchstart = handleTouchEvent;
document.ontouchend = handleTouchEvent;

select.addEventListener("change", function () {
    // Load file
    var request = new XMLHttpRequest();
    // For some reason using request.responseType = "arraybuffer" and then using that response strips out 
    // newline characters from the ROM, which breaks it since it should be loaded as a binary file.
    request.open("GET", select.options[select.selectedIndex].value);
    request.overrideMimeType("text/plain; charset=x-user-defined");      
    request.onload = function(oEvent) {
        var romFile = request.responseText;        
        chip8.loadRom(romFile);
        // Ensure screen is cleared when switching ROMs.
        ctx.fillStyle = "rgb(0, 0, 0)";
        ctx.fillRect(0, 0, canvas.width, canvas.height);
        // If the animation callback is running, the emulator will start running.        
    }
    request.send();
});

// Clear the canvas before we do anything else.
ctx.fillStyle = "rgb(0, 0, 0)";
ctx.fillRect(0, 0, canvas.width, canvas.height);

// By default, load the first ROM that's available.
select.dispatchEvent(new Event("change"));