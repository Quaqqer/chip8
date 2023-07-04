#![allow(dead_code)]

pub struct Chip8 {
    mem: [u8; 4096],
    pub display: [bool; 64 * 32],
    pc: u16,
    i: u16,
    stack: Vec<u16>,
    delay_timer: u8,
    sound_timer: u8,
    vs: [u8; 16],
    key_pressed: [bool; 16],
}

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
    fn read8(&self, addr: u16) -> u8 {
        self.mem[addr as usize]
    }

    fn read16(&self, addr: u16) -> u16 {
        let first = self.read8(addr) as u16;
        let second = self.read8(addr + 1) as u16;
        (first << 8) + second
    }

    fn fetch8(&mut self) -> u8 {
        let r = self.read8(self.pc);
        self.pc += 1;
        r
    }

    fn fetch16(&mut self) -> u16 {
        let r = self.read16(self.pc);
        self.pc += 2;
        r
    }

    pub fn cycle(&mut self) {
        let abcd = self.fetch16();
        // Opcode == 0xABCD
        let a = ((abcd >> 12) & 0x0f) as u8;
        let b = ((abcd >> 8) & 0x0f) as u8;
        let c = ((abcd >> 4) & 0x0f) as u8;
        let d = ((abcd >> 0) & 0x0f) as u8;
        let ab = ((abcd >> 8) & 0xff) as u8;
        let bc = ((abcd >> 4) & 0xff) as u8;
        let cd = ((abcd >> 0) & 0xff) as u8;
        let abc = ((abcd >> 4) & 0xfff) as u16;
        let bcd = ((abcd >> 0) & 0xfff) as u16;

        println!("{:#06x}", abcd);
        match (a, b, c, d) {
            (0x0, 0x0, 0xe, 0x0) => {
                self.clear_display();
            }
            (0x0, 0x0, 0xe, 0xe) => {
                // Return
                todo!();
            }
            (0x0, _, _, _) => {
                // Call machine code routine
                let addr = bcd;
                todo!();
            }
            (0x1, _, _, _) => {
                // Jump to address NNN
                self.pc = bcd;
            }
            (0x2, _, _, _) => {
                // Call subroutine NNN
                let addr = bcd;
                todo!();
            }
            (0x3, _, _, _) => {
                if self.vr(b) == cd {
                    self.pc += 2;
                };
            }
            (0x4, _, _, _) => {
                if self.vr(b) != cd {
                    self.pc += 2;
                }
            }
            (0x5, x, y, 0) => {
                if self.vr(x) == self.vr(y) {
                    self.pc += 2;
                }
            }
            (0x6, x, _, _) => {
                let nn = cd;
                self.vw(x, nn);
            }
            (0x7, x, _, _) => {
                let nn = cd;
                self.vw(x, self.vr(x) + nn);
            }
            (0x8, x, y, 0x0) => {
                self.vw(x, self.vr(y));
            }
            (0x8, x, y, 0x1) => {
                self.vw(x, self.vr(x) | self.vr(y));
            }
            (0x8, x, y, 0x2) => {
                self.vw(x, self.vr(x) & self.vr(y));
            }
            (0x8, x, y, 0x3) => {
                self.vw(x, self.vr(x) ^ self.vr(y));
            }
            (0x8, x, y, 0x4) => {
                let (res, of) = self.vr(x).overflowing_add(self.vr(y));
                self.vw(x, res);
                self.vw(0xf, if of { 1 } else { 0 });
            }
            (0x8, x, y, 0x5) => {
                let unborrowed = self.vr(x) > self.vr(y);
                let (res, _) = self.vr(x).overflowing_sub(self.vr(y));
                self.vw(x, res);
                self.vw(0xf, if unborrowed { 1 } else { 0 });
            }
            (0x8, x, _, 0x6) => {
                let vx = self.vr(x);
                self.vw(x, x >> 1);
                self.vw(0xf, vx & 0b1);
            }
            (0x8, x, y, 0x7) => {
                let unborrowed = self.vr(y) > self.vr(x);
                let (res, _) = self.vr(y).overflowing_sub(self.vr(x));
                self.vw(x, res);
                self.vw(0xf, if unborrowed { 1 } else { 0 });
            }
            (0x8, x, _, 0xe) => {
                let vx = self.vr(x);
                self.vw(x, x << 1);
                self.vw(0xf, if vx & 0x80 != 0 { 1 } else { 0 });
            }
            (0x9, x, y, 0x0) => {
                if self.vr(x) != self.vr(y) {
                    self.pc += 2;
                }
            }
            (0xa, _, _, _) => {
                self.i = bcd;
            }
            (0xb, _, _, _) => {
                self.pc = self.vr(0x0) as u16 + bcd;
            }
            (0xc, x, _, _) => {
                let rand: u8 = todo!();
                self.vw(x, rand & self.vr(0x0));
            }
            (0xd, x, y, n) => {
                self.draw(self.vr(x), self.vr(y), n);
            }
            (0xe, x, 0x9, 0xe) => {
                if self.key_pressed[x as usize] {
                    self.pc += 2;
                }
            }
            (0xe, x, 0xa, 0x1) => {
                if !self.key_pressed[x as usize] {
                    self.pc += 2;
                }
            }
            (0xf, x, 0x0, 0x7) => {
                self.vw(x, self.delay_timer);
            }
            (0xf, x, 0x0, 0xa) => {
                self.vw(x, self.get_key());
            }
            (0xf, x, 0x1, 0x1) => {
                self.delay_timer = self.vr(x);
            }
            (0xf, x, 0x1, 0x5) => {
                self.sound_timer = self.vr(x);
            }
            (0xf, x, 0x1, 0xe) => {
                self.i += self.vr(x) as u16;
            }
            (0xf, x, 0x2, 0x9) => {
                self.i = 5 * x as u16;
            }
            (0xf, x, 0x3, 0x3) => {
                let mut curr = self.vr(x);

                for i in 0..3 {
                    self.mem[self.i as usize + 2 - i] = curr % 10;
                    curr /= 10;
                }
            }
            (0xf, x, 0x5, 0x5) => {
                for i in 0x0..=x {
                    self.mem[self.i as usize + i as usize] = self.vs[i as usize];
                }
            }
            (0xf, x, 0x6, 0x5) => {
                for i in 0x0..=x {
                    self.vs[i as usize] = self.mem[self.i as usize + i as usize];
                }
            }
            _ => panic!("Unimplemented instruction"),
        }
    }

    fn clear_display(&mut self) {
        self.display.fill(false);
    }

    fn draw(&mut self, x_offset: u8, y_offset: u8, height: u8) {
        for dy in 0..height {
            for dx in 0..8 {
                self.display[(dy + y_offset) as usize * 64 + (7 - dx + x_offset) as usize] =
                    (self.mem[self.i as usize + dy as usize] >> dx) & 0x1 == 0x1;
            }
        }
    }

    fn get_key(&self) -> u8 {
        todo!()
    }

    fn vr(&self, addr: u8) -> u8 {
        self.vs[addr as usize]
    }

    fn vw(&mut self, addr: u8, v: u8) {
        self.vs[addr as usize] = v;
    }

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
        }
    }
}
