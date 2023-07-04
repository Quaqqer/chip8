#![allow(dead_code)]

use std::fmt;

/// The Chip8 emulator
///
/// * `mem`: 4KB memory
/// * `display`: 64x32 binary display
/// * `pc`: 16 bit program counter
/// * `i`: The memory index used for sprites
/// * `stack`: The stack for subroutines
/// * `delay_timer`: The delay timer, decreased at 60Hz
/// * `sound_timer`: The sound timer, like delay timer decreased at 60Hz, *should* cause a sound
///    but I have not implemented that *yet*
/// * `vs`: The registers v0-vF
/// * `key_pressed`: An array of currently pressed keys
/// * `state`: The state of the emulator, used for blocking key grabs
pub struct Chip8 {
    mem: [u8; 4096],
    display: [bool; 64 * 32],
    pc: u16,
    i: u16,
    stack: Vec<u16>,
    delay_timer: u8,
    sound_timer: u8,
    vs: [u8; 16],
    key_pressed: [bool; 16],
    state: State,
}

// Custom debug print for Chip8 because printing the whole memory is not reasonable
impl fmt::Debug for Chip8 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Chip8")
            .field("pc", &self.pc)
            .field("i", &self.i)
            .field("delay", &self.delay_timer)
            .field("sound", &self.sound_timer)
            .field("registers", &self.vs)
            .field("pressed", &self.key_pressed)
            .field("state", &self.state)
            .finish()
    }
}

#[derive(Debug)]
pub enum State {
    Default,
    GetKey(u8),
}

/// A commonly used font for CHIP-8
static FONT: [u8; 5 * 16] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

impl Chip8 {
    /// Read a byte
    fn read8(&self, addr: u16) -> u8 {
        self.mem[addr as usize]
    }

    /// Read a word
    fn read16(&self, addr: u16) -> u16 {
        let first = self.read8(addr) as u16;
        let second = self.read8(addr + 1) as u16;
        (first << 8) + second
    }

    /// Fetch a byte and increment the program counter
    fn fetch8(&mut self) -> u8 {
        let r = self.read8(self.pc);
        self.pc += 1;
        r
    }

    /// Fetch a word and increment the program counter
    fn fetch16(&mut self) -> u16 {
        let r = self.read16(self.pc);
        self.pc += 2;
        r
    }

    /// Perform a clock cycle, unless blocking GetKey
    pub fn cycle(&mut self) {
        // Get key is blocking
        if let State::GetKey(_) = self.state {
            return;
        };

        // Fetch the opcode, split into different parts for ease of use
        let abcd = self.fetch16();
        let a = ((abcd >> 12) & 0x0f) as u8;
        let b = ((abcd >> 8) & 0x0f) as u8;
        let c = ((abcd >> 4) & 0x0f) as u8;
        let d = ((abcd >> 0) & 0x0f) as u8;
        let cd = ((abcd >> 0) & 0xff) as u8;
        let bcd = ((abcd >> 0) & 0xfff) as u16;

        match (a, b, c, d) {
            // Clear the display
            (0x0, 0x0, 0xe, 0x0) => {
                self.clear_display();
            }

            // Return from a subroutine
            (0x0, 0x0, 0xe, 0xe) => {
                self.pc = self.stack.pop().unwrap();
            }

            // Jump
            (0x1, _, _, _) => {
                self.pc = bcd;
            }

            // Call a subroutine, jumps and push to stack
            (0x2, _, _, _) => {
                self.stack.push(self.pc);
                self.pc = bcd;
            }

            // Skip next if vX == NN
            (0x3, x, _, _) => {
                if self.vr(x) == cd {
                    self.pc += 2;
                };
            }

            // Skip next if vX != NN
            (0x4, x, _, _) => {
                if self.vr(x) != cd {
                    self.pc += 2;
                }
            }

            // Skip next if vX == vY
            (0x5, x, y, 0) => {
                if self.vr(x) == self.vr(y) {
                    self.pc += 2;
                }
            }

            // Write NN to vX
            (0x6, x, _, _) => {
                self.vw(x, cd);
            }

            // Increase vX by NN
            (0x7, x, _, _) => {
                self.vw(x, self.vr(x).wrapping_add(cd));
            }

            // Write vY to vX
            (0x8, x, y, 0x0) => {
                self.vw(x, self.vr(y));
            }

            // Or vX with vY
            (0x8, x, y, 0x1) => {
                self.vw(x, self.vr(x) | self.vr(y));
                self.vw(0xf, 0x0); // CHIP-8 quirk
            }

            // And vX with vY
            (0x8, x, y, 0x2) => {
                self.vw(x, self.vr(x) & self.vr(y));
                self.vw(0xf, 0x0); // CHIP-8 quirk
            }

            // Xor vX with vY
            (0x8, x, y, 0x3) => {
                self.vw(x, self.vr(x) ^ self.vr(y));
                self.vw(0xf, 0x0); // CHIP-8 quirk
            }

            // Add vY to vX, set overflow in flag register
            (0x8, x, y, 0x4) => {
                let (res, of) = self.vr(x).overflowing_add(self.vr(y));
                self.vw(x, res);
                self.vw(0xf, if of { 1 } else { 0 });
            }

            // Subtract vX by vY, set flag to the borrow bit, set if subtraction did not require a
            // borrow
            (0x8, x, y, 0x5) => {
                let unborrowed = self.vr(x) > self.vr(y);
                let res = self.vr(x).wrapping_sub(self.vr(y));
                self.vw(x, res);
                self.vw(0xf, if unborrowed { 1 } else { 0 });
            }

            // Shift vY to the right and store in vX, save shifted bit in flag
            (0x8, x, y, 0x6) => {
                self.vw(x, self.vr(y)); // CHIP-8 quirk
                let vx = self.vr(x);
                self.vw(x, vx >> 1);
                self.vw(0xf, vx & 0b1);
            }

            // Subtract vY by vX and store in vX, set flag to borrow bit
            (0x8, x, y, 0x7) => {
                let unborrowed = self.vr(y) > self.vr(x);
                let (res, _) = self.vr(y).overflowing_sub(self.vr(x));
                self.vw(x, res);
                self.vw(0xf, if unborrowed { 1 } else { 0 });
            }

            // Shift vY to the left and store in vY, save shifted bit in flag
            (0x8, x, y, 0xe) => {
                self.vw(x, self.vr(y)); // CHIP-8 quirk
                let vx = self.vr(x);
                self.vw(x, vx << 1);
                self.vw(0xf, if vx & 0x80 != 0 { 1 } else { 0 });
            }

            // Skip if vX !- vY
            (0x9, x, y, 0x0) => {
                if self.vr(x) != self.vr(y) {
                    self.pc += 2;
                }
            }

            // Write NNN to I
            (0xa, _, _, _) => {
                self.i = bcd;
            }

            // Jump to v0 + NNN
            (0xb, _, _, _) => {
                self.pc = self.vr(0x0) as u16 + bcd;
            }

            // Get a random number anded with NN
            (0xc, x, _, _) => {
                let random = rand::random::<u8>();
                self.vw(x, random & cd);
            }

            // Draw sprite in memory at position I offset by vX and vY, N rows long
            (0xd, x, y, n) => {
                self.draw(self.vr(x), self.vr(y), n);
            }

            // Skip if key vX is pressed
            (0xe, x, 0x9, 0xe) => {
                if self.key_pressed[self.vr(x) as usize] {
                    self.pc += 2;
                }
            }

            // Skip if key vX is not pressed
            (0xe, x, 0xa, 0x1) => {
                if !self.key_pressed[self.vr(x) as usize] {
                    self.pc += 2;
                }
            }

            // Write delay timer to vX
            (0xf, x, 0x0, 0x7) => {
                self.vw(x, self.delay_timer);
            }

            // Block get key to vX
            (0xf, x, 0x0, 0xa) => {
                self.state = State::GetKey(x);
            }

            // Write delay timer to vX
            (0xf, x, 0x1, 0x1) => {
                self.vw(x, self.delay_timer);
            }

            // Write vX to delay timer
            (0xf, x, 0x1, 0x5) => {
                self.delay_timer = self.vr(x);
            }

            // Write vX to sound timer
            (0xf, x, 0x1, 0x8) => {
                self.sound_timer = self.vr(x);
            }

            // Increment I by vX
            (0xf, x, 0x1, 0xe) => {
                self.i = self.i.wrapping_add(self.vr(x) as u16);
            }

            // Get sprite for character X, stored in the beginning of memory
            (0xf, x, 0x2, 0x9) => {
                self.i = 0 + 5 * self.vr(x) as u16;
            }

            // Write decimal digits to memory at I, memory[I] = 100's digit, memory[I+1] = 10's
            // digit, memory[I+2] = 1's digit
            (0xf, x, 0x3, 0x3) => {
                let mut curr = self.vr(x);

                for i in 0..3 {
                    self.mem[self.i as usize + 2 - i] = curr % 10;
                    curr /= 10;
                }
            }

            // Store registers v0..=vX to memory[I..]
            (0xf, x, 0x5, 0x5) => {
                for i in 0x0..=x {
                    self.mem[self.i as usize + i as usize] = self.vs[i as usize];
                }
                self.i += 1; // CHIP-8 quirk
            }

            // Load registers v0..=vX from memory[I..]
            (0xf, x, 0x6, 0x5) => {
                for i in 0x0..=x {
                    self.vs[i as usize] = self.mem[self.i as usize + i as usize];
                }
                self.i += 1; // CHIP-8 quirk
            }
            _ => panic!("Unimplemented instruction {:#06x}", abcd),
        }
    }

    /// Clear the display
    fn clear_display(&mut self) {
        self.display.fill(false);
    }

    /// Draw sprite instruction
    fn draw(&mut self, mut x_offset: u8, mut y_offset: u8, height: u8) {
        self.vw(0xf, 0);

        x_offset %= 64;
        y_offset %= 32;

        for dy in 0..height {
            let y = y_offset as usize + dy as usize;

            for dx in 0..8 {
                let x = x_offset as usize + dx as usize;

                if 64 <= x || 32 <= y {
                    continue;
                };

                let tile = self.mem[self.i as usize + dy as usize];
                let pixel = (tile << dx) & 0b10000000 != 0;

                if pixel {
                    let disp_i = y * 64 + x;

                    if self.display[disp_i] {
                        self.vw(0xf, 1)
                    }

                    self.display[disp_i] = !self.display[disp_i];
                }
            }
        }
    }

    /// Read register vX
    fn vr(&self, x: u8) -> u8 {
        self.vs[x as usize]
    }

    /// Write to a register vX
    fn vw(&mut self, x: u8, v: u8) {
        self.vs[x as usize] = v;
    }

    /// Create a new emulator, initialise with ROM (which is not really a ROM)
    pub fn new(rom: Vec<u8>) -> Self {
        let mut mem = [0; 4096];

        for i in 0..FONT.len() {
            mem[i] = FONT[i];
        }

        for i in 0..rom.len() {
            mem[i + 0x200] = rom[i];
        }

        Self {
            mem,
            display: [false; 64 * 32],
            pc: 0x200,
            i: 0,
            stack: Vec::new(),
            delay_timer: 0,
            sound_timer: 0,
            vs: [0; 16],
            key_pressed: [false; 16],
            state: State::Default,
        }
    }

    /// Send a key down event
    pub fn down(&mut self, v: u8) {
        if !self.key_pressed[v as usize] {
            if let State::GetKey(x) = self.state {
                self.vw(x, v);
                self.state = State::Default;
            };

            self.key_pressed[v as usize] = true;
        }
    }

    /// Send a key up event
    pub fn up(&mut self, v: u8) {
        self.key_pressed[v as usize] = false;
    }

    /// Tick the delay and sound timer, call 60 times a second
    pub fn decrease_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    /// Get the display, indexed [y * 64 + 32]
    pub fn display(&self) -> &[bool; 64 * 32] {
        &self.display
    }
}
