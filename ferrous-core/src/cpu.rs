/*
Copyright 2020 Anish Jewalikar

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

//! Contains a simple and full featured implementation
//! of a (super) Chip-8 interpreter.

use crate::font::*;

/// Implementation of a (super) Chip-8 interpreter.
///
/// # Example
///
/// ```rust
/// use ch8_core::CPU;
///
/// let mut cpu = CPU::new();
///
/// // Load ROM, handle display, audio and input.
/// ```
#[derive(Debug, Clone)]
pub struct CPU {
    /// Working RAM of the CPU.
    /// 4 KB in size.
    pub memory: Box<[u8; 0x1000]>,

    /// Return address stack.
    pub stack: Box<[u16; 0x10]>,

    /// Sixteen general purpose registers.
    /// Conventionally named as V0 to VF.
    /// VF is a special register, that is used as a flag.
    pub register: Box<[u8; 0x10]>,

    /// Program Counter; Stores current location in the memory.
    pub pc: usize,

    /// Stack Pointer; Stores current location in the stack.
    pub sp: usize,

    /// Index Register; Stores an arbitrary address, specified by the user.
    pub i: usize,

    /// Delay Timer; Used for sync and more.
    /// It is decremented at a rate of 60Hz when non-zero.
    pub dt: u8,

    /// Sound TImer; An audio beep is played when it's non-zero.
    /// It is also decremented at a rate of 60Hz when non-zero.
    pub st: u8,

    /// Video RAM; Used to store the current state of the 64 * 32 pixels
    /// screen.
    /// Each byte represents an individual pixel, where 1 means ON (White)
    /// and 0 means OFF (Black).
    pub vram: Box<[u8; 128 * 64]>,

    /// Keypad Representation; Conveys whether a key is pressed (true) or not pressed
    /// (false) currently.
    pub keypad: Box<[bool; 0x10]>,

    /// Is the interpreter in high resolution (SCHIP) mode?
    pub is_highres: bool,

    /// Has the interpreter stopped? (through EXIT SCHIP)
    pub is_halted: bool,

    /// If we should not increment I after Fx55, Fx65.
    pub load_store_quirk: bool,

    /// If we should ignore Vy in shift opcodes.
    pub shift_quirk: bool,

    /// Factor in the highest nibble of address to select register
    // for jump.
    pub jump_quirk: bool,

    /// Super Chip 8 flag registers.
    pub flag_regs: Box<[u8; 8]>,
}

impl Default for CPU {
    fn default() -> Self {
        Self::new()
    }
}

/// General Methods
impl CPU {
    /// Create a new `CPU` instance.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ch8_core::CPU;
    ///
    /// let mut cpu = CPU::new();
    /// ```
    pub fn new() -> Self {
        let mut memory = Box::new([0; 0x1000]);

        // Load font sprites into memory.
        memory[0..80].copy_from_slice(&FONT_SPRITES);
        memory[80..240].copy_from_slice(&HIGH_RES_FONT_SPRITES);

        Self {
            memory,
            stack: Box::new([0; 0x10]),
            register: Box::new([0; 0x10]),
            pc: 0x200, // All programs start from 0x200.
            sp: 0,
            i: 0,
            dt: 0,
            st: 0,
            vram: Box::new([0; 128 * 64]),
            keypad: Box::new([false; 0x10]),
            flag_regs: Box::new([0; 8]),
            is_halted: false,
            is_highres: false,
            load_store_quirk: false,
            shift_quirk: false,
            jump_quirk: false,
        }
    }

    /// Reset the interpreter to its initial state.
    /// Flag Registers & Quirks are persistent even after a reset.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ch8_core::CPU;
    ///
    /// let mut cpu = CPU::new();
    ///
    /// // After the ROM's execution is deemed finished,
    /// // and we desire to start afresh, and load a new ROM.
    /// cpu.reset();
    /// ```
    pub fn reset(&mut self) {
        // Clear only the non-reserved memory.
        self.memory[0x200..].iter_mut().for_each(|x| *x = 0);
        self.register.iter_mut().for_each(|x| *x = 0);

        self.pc = 0x200;
        self.sp = 0;
        self.i = 0;
        self.dt = 0;
        self.st = 0;

        self.vram.iter_mut().for_each(|x| *x = 0);
        self.keypad.iter_mut().for_each(|x| *x = false);

        self.is_halted = false;
        self.is_highres = false;
    }

    /// Load a ROM into the working memory thus finalizing for execution.
    ///
    /// Returns an `Err`, if the buffer's length is larger than the permitted,
    /// 3584 bytes.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ch8_core::CPU;
    ///
    /// let mut cpu = CPU::new();
    ///
    /// // Load a Chip-8 binary ROM.
    /// // Here we are just loading a stub.
    /// cpu.load_rom(&[0]);
    /// ```
    pub fn load_rom(&mut self, buffer: &[u8]) -> Result<(), String> {
        // Return an error, if bounds are exceeded.
        if buffer.len() > 3584 {
            return Err(
                "ROM\'s length is larger than the permitted 3584 bytes."
                    .to_string(),
            );
        }

        // Copy the ROM buffer.
        self.memory[0x200..0x200 + buffer.len()].copy_from_slice(&buffer);

        Ok(())
    }

    /// Decrement the delay timer and sound timer if they are non-zero.
    ///
    /// They are expected to be decremented at 60Hz therefore your frontend
    /// should call this function roughly every 16.67 milliseconds for an
    /// accurate emulation.
    pub fn step_timers(&mut self) {
        if self.dt > 0 {
            self.dt -= 1;
        }

        if self.st > 0 {
            self.st -= 1;
        }
    }

    /// Reset the keypad to its initial state.
    ///
    /// This will make all the keys 'unpressed'
    /// by resetting all values in the keypad array to false.
    pub fn reset_keys(&mut self) {
        self.keypad.iter_mut().for_each(|x| *x = false);
    }

    /// Set the key to either be pressed (true) or unpressed (false)
    /// at the given index.
    pub fn set_key_at_index(&mut self, index: usize, value: bool) {
        self.keypad[index] = value;
    }

    /// Set the load/store quirk to the given boolean value.
    pub fn set_load_store(&mut self, value: bool) {
        self.load_store_quirk = value;
    }

    /// Set the shift quirk to the given boolean value.
    pub fn set_shift(&mut self, value: bool) {
        self.shift_quirk = value;
    }

    /// Set the jump quirk to the given value.
    pub fn set_jump(&mut self, value: bool) {
        self.jump_quirk = value;
    }

    /// Execute one fetch-decode-execute cycle,
    /// return the opcode that was fetched in the process.
    ///
    /// Currently if the CPU is halted as a result of a superchip
    /// HALT opcode this function does nothing but return None.
    pub fn execute_cycle(&mut self) -> Option<u16> {
        if self.is_halted {
            return None;
        }

        // Fetch the opcode from memory.
        let opcode = self.fetch_opcode();
        self.pc += 2;

        let bytes = opcode.to_be_bytes();

        // Decode the Instruction, and execute appropriately.
        // Split the two byte opcode into 4 nibbles.
        let nibbles = (
            (bytes[0] & 0xF0) >> 4,
            (bytes[0] & 0x0F),
            (bytes[1] & 0xF0) >> 4,
            (bytes[1] & 0x0F),
        );

        // Common variables are extracted here so that
        // we don't have to do that in the method itself.
        let x = nibbles.1 as usize;
        let y = nibbles.2 as usize;

        let kk = bytes[1];
        let nnn = opcode & 0x0FFF;

        // Match the nibble and call the correct opcode
        // method.
        match nibbles {
            // 0x0000 - 0x1000
            (0x0, 0x0, 0xC, _) => self.op_00cn(nibbles.3),
            (0x0, 0x0, 0xE, 0x0) => self.op_00e0(),
            (0x0, 0x0, 0xE, 0xE) => self.op_00ee(),
            (0x0, 0x0, 0xF, 0xB) => self.op_00fb(),
            (0x0, 0x0, 0xF, 0xC) => self.op_00fc(),
            (0x0, 0x0, 0xF, 0xD) => self.op_00fd(),
            (0x0, 0x0, 0xF, 0xE) => self.op_00fe(),
            (0x0, 0x0, 0xF, 0xF) => self.op_00ff(),

            // 0x1000 - 0x8000
            (0x1, _, _, _) => self.op_1nnn(nnn),
            (0x2, _, _, _) => self.op_2nnn(nnn),
            (0x3, _, _, _) => self.op_3xkk(x, kk),
            (0x4, _, _, _) => self.op_4xkk(x, kk),
            (0x5, _, _, 0) => self.op_5xy0(x, y),
            (0x6, _, _, _) => self.op_6xkk(x, kk),
            (0x7, _, _, _) => self.op_7xkk(x, kk),

            // 0x8000 - 0x9000
            (0x8, _, _, 0x0) => self.op_8xy0(x, y),
            (0x8, _, _, 0x1) => self.op_8xy1(x, y),
            (0x8, _, _, 0x2) => self.op_8xy2(x, y),
            (0x8, _, _, 0x3) => self.op_8xy3(x, y),
            (0x8, _, _, 0x4) => self.op_8xy4(x, y),
            (0x8, _, _, 0x5) => self.op_8xy5(x, y),
            (0x8, _, _, 0x6) => self.op_8xy6(x, y),
            (0x8, _, _, 0x7) => self.op_8xy7(x, y),
            (0x8, _, _, 0xE) => self.op_8xye(x, y),

            // 0x9000 - 0xA000
            (0x9, _, _, 0) => self.op_9xy0(x, y),

            // 0xA000 - 0xC000
            (0xA, _, _, _) => self.op_annn(nnn),
            (0xB, _, _, _) => self.op_bnnn(nnn),

            // 0xC000 - 0xD000
            (0xC, _, _, _) => self.op_cxkk(x, kk),

            // 0xD000 - 0xE000
            (0xD, _, _, _) => self.op_dxyn(x, y, nibbles.3 as usize),

            // 0xE000 - 0xF000
            (0xE, _, 0x9, 0xE) => self.op_ex9e(x),
            (0xE, _, 0xA, 0x1) => self.op_exa1(x),

            // 0xF000
            (0xF, _, 0x0, 0x7) => self.op_fx07(x),
            (0xF, _, 0x0, 0xA) => self.op_fx0a(x),
            (0xF, _, 0x1, 0x5) => self.op_fx15(x),
            (0xF, _, 0x1, 0x8) => self.op_fx18(x),
            (0xF, _, 0x1, 0xE) => self.op_fx1e(x),
            (0xF, _, 0x2, 0x9) => self.op_fx29(x),
            (0xF, _, 0x3, 0x0) => self.op_fx30(x),
            (0xF, _, 0x3, 0x3) => self.op_fx33(x),
            (0xF, _, 0x5, 0x5) => self.op_fx55(x),
            (0xF, _, 0x6, 0x5) => self.op_fx65(x),
            (0xF, _, 0x7, 0x5) => self.op_fx75(x),
            (0xF, _, 0x8, 0x5) => self.op_fx85(x),

            // Unknown/Invalid opcodes
            _ => return None,
        }

        Some(opcode)
    }

    /// Fetch the VRAM as a reference to a u8 slice.
    pub fn get_video_buffer(&self) -> &[u8] {
        self.vram.as_ref()
    }

    /// Get the current number of rows and columns as tuple.
    pub fn get_height_width(&self) -> (usize, usize) {
        if self.is_highres {
            (64, 128)
        } else {
            (32, 64)
        }
    }

    /// Fetch the next opcode that is to be executed from the ROM.
    fn fetch_opcode(&self) -> u16 {
        u16::from_be_bytes([self.memory[self.pc], self.memory[self.pc + 1]])
    }
}

/// Standard CHIP opcodes.
impl CPU {
    /// 00E0 - CLS  
    /// Clear the display.
    fn op_00e0(&mut self) {
        self.vram.iter_mut().for_each(|x| *x = 0);
    }

    /// 00EE - RET  
    /// Return from a subroutine.
    fn op_00ee(&mut self) {
        self.sp -= 1;
        self.pc = self.stack[self.sp] as usize;
    }

    /// 1nnn - JP addr  
    /// Jump to location nnn.
    fn op_1nnn(&mut self, nnn: u16) {
        self.pc = nnn as usize;
    }

    /// 2nnn - CALL addr  
    /// Call subroutine at nnn.
    fn op_2nnn(&mut self, nnn: u16) {
        // Store return address.
        self.stack[self.sp] = self.pc as u16;
        self.sp += 1;

        // Call subroutine.
        self.pc = nnn as usize;
    }

    /// 3xkk - SE Vx, byte  
    /// Skip next instruction if Vx = kk.
    fn op_3xkk(&mut self, x: usize, kk: u8) {
        if self.register[x] == kk {
            self.pc += 2;
        }
    }

    /// 4xkk - SNE Vx, byte  
    /// Skip next instruction if Vx != kk.
    fn op_4xkk(&mut self, x: usize, kk: u8) {
        if self.register[x] != kk {
            self.pc += 2;
        }
    }

    /// 5xy0 - SE Vx, Vy  
    /// Skip next instruction if Vx = Vy.
    fn op_5xy0(&mut self, x: usize, y: usize) {
        if self.register[x] == self.register[y] {
            self.pc += 2;
        }
    }

    /// 6xkk - LD Vx, byte  
    /// Set Vx = kk.
    fn op_6xkk(&mut self, x: usize, kk: u8) {
        self.register[x] = kk;
    }

    /// 7xkk - ADD Vx, byte  
    /// Set Vx = Vx + kk.
    fn op_7xkk(&mut self, x: usize, kk: u8) {
        self.register[x] = self.register[x].wrapping_add(kk);
    }

    /// 8xy0 - LD Vx, Vy  
    /// Set Vx = Vy.
    fn op_8xy0(&mut self, x: usize, y: usize) {
        self.register[x] = self.register[y];
    }

    /// 8xy1 - OR Vx, Vy  
    /// Set Vx = Vx OR Vy.
    fn op_8xy1(&mut self, x: usize, y: usize) {
        self.register[x] |= self.register[y];
    }

    /// 8xy2 - AND Vx, Vy  
    /// Set Vx = Vx AND Vy.
    fn op_8xy2(&mut self, x: usize, y: usize) {
        self.register[x] &= self.register[y];
    }

    /// 8xy3 - XOR Vx, Vy  
    /// Set Vx = Vx XOR Vy.
    fn op_8xy3(&mut self, x: usize, y: usize) {
        self.register[x] ^= self.register[y];
    }

    /// 8xy4 - ADD Vx, Vy  
    /// Set Vx = Vx + Vy, set VF = carry.
    fn op_8xy4(&mut self, x: usize, y: usize) {
        let result = self.register[x].overflowing_add(self.register[y]);

        self.register[x] = result.0;
        self.register[0xF] = if result.1 { 1 } else { 0 };
    }

    /// 8xy5 - SUB Vx, Vy  
    /// Set Vx = Vx - Vy, set VF = NOT borrow.
    fn op_8xy5(&mut self, x: usize, y: usize) {
        let result = self.register[x].overflowing_sub(self.register[y]);

        self.register[x] = result.0;
        self.register[0xF] = if result.1 { 0 } else { 1 };
    }

    /// 8xy6 - SHR Vx {, Vy}  
    /// Set Vx = Vx SHR 1.
    fn op_8xy6(&mut self, x: usize, y: usize) {
        let y = if self.shift_quirk { x } else { y };
        self.register[0xF] = self.register[y] & 0b0000_0001;

        let temporary = self.register[y].wrapping_shr(1);
        self.register[x] = temporary;
    }

    /// 8xy7 - SUBN Vx, Vy  
    /// Set Vx = Vy - Vx, set VF = NOT borrow.
    fn op_8xy7(&mut self, x: usize, y: usize) {
        let result = self.register[y].overflowing_sub(self.register[x]);

        self.register[x] = result.0;
        self.register[0xF] = if result.1 { 0 } else { 1 };
    }

    /// 8xyE - SHL Vx {, Vy}  
    /// Set Vx = Vx SHL 1.
    fn op_8xye(&mut self, x: usize, y: usize) {
        let y = if self.shift_quirk { x } else { y };
        self.register[0xF] = (self.register[y] & 0b1000_0000) >> 7;

        let temporary = self.register[y].wrapping_shl(1);
        self.register[x] = temporary;
    }

    /// 9xy0 - SNE Vx, Vy  
    /// Skip next instruction if Vx != Vy.
    fn op_9xy0(&mut self, x: usize, y: usize) {
        if self.register[x] != self.register[y] {
            self.pc += 2;
        }
    }

    /// Annn - LD I, addr  
    /// Set I = nnn.
    fn op_annn(&mut self, nnn: u16) {
        self.i = nnn as usize;
    }

    /// Bnnn - JP V0, addr  
    /// Jump to location nnn + V0.
    fn op_bnnn(&mut self, nnn: u16) {
        if self.jump_quirk {
            self.pc = nnn as usize
                + self.register[(nnn >> 8) as usize & 0xF] as usize;
        } else {
            self.pc = nnn as usize + self.register[0] as usize;
        }
    }

    /// Cxkk - RND Vx, byte  
    /// Set Vx = random byte AND kk.
    fn op_cxkk(&mut self, x: usize, kk: u8) {
        self.register[x] = rand::random::<u8>() & kk;
    }

    /// Dxyn - DRW Vx, Vy, nibble  
    /// Display n-byte sprite starting at memory location I at (Vx, Vy),
    /// set VF = collision.
    fn op_dxyn(&mut self, vx: usize, vy: usize, n: usize) {
        let (rows, cols) = self.get_height_width();

        let x = self.register[vx] as usize;
        let y = self.register[vy] as usize;

        self.register[0xF] = 0;

        if n == 0 {
            // Super Chip 16x16 sprite
            for r in 0..16 {
                for c in 0..16 {
                    let byte = self.memory
                        [self.i + (r * 2) + (if c > 7 { 1 } else { 0 })]
                        as usize;

                    if (byte & (0x80 >> (c % 8))) != 0 {
                        let index = ((x + c) % cols) + ((y + r) % rows) * cols;

                        if self.vram[index] == 1 {
                            self.register[0xF] = 1;
                        }

                        self.vram[index] ^= 1;
                    }
                }
            }
        } else {
            // Chip 8xN sprite
            for r in 0..n {
                for c in 0..8 {
                    let byte = self.memory[self.i + r] as usize;

                    if (byte & (0x80 >> c)) != 0 {
                        let index = ((x + c) % cols) + ((y + r) % rows) * cols;

                        if self.vram[index] == 1 {
                            self.register[0xF] = 1;
                        }

                        self.vram[index] ^= 1;
                    }
                }
            }
        }
    }

    /// Ex9E - SKP Vx  
    /// Skip next instruction if key with the value of Vx is pressed.
    fn op_ex9e(&mut self, x: usize) {
        if self.keypad[self.register[x] as usize] {
            self.pc += 2;
        }
    }

    /// ExA1 - SKNP Vx  
    /// Skip next instruction if key with the value of Vx is not pressed.
    fn op_exa1(&mut self, x: usize) {
        if !self.keypad[self.register[x] as usize] {
            self.pc += 2;
        }
    }

    /// Fx07 - LD Vx, DT  
    /// Set Vx = delay timer value.
    fn op_fx07(&mut self, x: usize) {
        self.register[x] = self.dt;
    }

    /// Fx0A - LD Vx, K  
    /// Wait for a key press, store the value of the key in Vx.
    fn op_fx0a(&mut self, x: usize) {
        for (count, key) in self.keypad.iter_mut().enumerate() {
            if *key {
                self.register[x] = count as u8;
                return;
            }
        }

        self.pc -= 2;
    }

    /// Fx15 - LD DT, Vx  
    /// Set delay timer = Vx.
    fn op_fx15(&mut self, x: usize) {
        self.dt = self.register[x];
    }

    /// Fx18 - LD ST, Vx  
    /// Set sound timer = Vx.
    fn op_fx18(&mut self, x: usize) {
        self.st = self.register[x];
    }

    /// Fx1E - ADD I, Vx  
    /// Set I = I + Vx.
    fn op_fx1e(&mut self, x: usize) {
        self.i += self.register[x] as usize;
    }

    /// Fx29 - LD F, Vx  
    /// Set I = location of sprite for digit Vx.
    fn op_fx29(&mut self, x: usize) {
        self.i = self.register[x] as usize * 5;
    }

    /// Fx33 - LD B, Vx  
    /// Store BCD representation of Vx in memory locations I, I+1, and I+2.
    fn op_fx33(&mut self, x: usize) {
        let value = self.register[x];

        self.memory[self.i] = value / 100;
        self.memory[self.i + 1] = (value % 100) / 10;
        self.memory[self.i + 2] = value % 10;
    }

    /// Fx55 - LD [I], Vx  
    /// Store registers V0 through Vx in memory starting at location I.
    fn op_fx55(&mut self, x: usize) {
        self.memory[self.i..=self.i + x].copy_from_slice(&self.register[0..=x]);

        if !self.load_store_quirk {
            self.i = (self.i + x + 1) & 0xFFFF;
        }
    }

    /// Fx65 - LD Vx, [I]  
    /// Read registers V0 through Vx from memory starting at location I.
    fn op_fx65(&mut self, x: usize) {
        self.register[0..=x].copy_from_slice(&self.memory[self.i..=self.i + x]);

        if !self.load_store_quirk {
            self.i = (self.i + x + 1) & 0xFFFF;
        }
    }
}

/// SCHIP opcodes
impl CPU {
    /// 00Cn - SCD nibble  
    /// Scroll display N lines down.
    fn op_00cn(&mut self, n: u8) {
        let (rows, cols) = self.get_height_width();

        // Get the number of rows that are retained.
        let retained = rows as u8 - n;

        // Get the index + 1 of the last pixel that is retained.
        let last_index = cols as usize * retained as usize;

        // Memove the retained pixels.
        self.vram
            .copy_within(0..last_index, cols as usize * n as usize);

        // Clear the upper pixels.
        self.vram[0..(cols as usize * n as usize)]
            .iter_mut()
            .for_each(|x| *x = 0);
    }

    /// 00FB - SCR  
    /// Scroll display 4 pixels right.
    fn op_00fb(&mut self) {
        let (rows, cols) = self.get_height_width();

        for row in 0..rows {
            let start = cols as usize * row as usize;

            self.vram
                .copy_within(start..(start + cols as usize - 4), start + 4);

            self.vram[start..start + 4].iter_mut().for_each(|x| *x = 0);
        }
    }

    /// 00FC - SCL  
    /// Scroll display 4 pixels left.
    fn op_00fc(&mut self) {
        let (rows, cols) = self.get_height_width();

        for row in 0..rows {
            let start = cols as usize * row as usize;
            let end = start + cols as usize;

            self.vram.copy_within((start + 4)..end, start);

            self.vram[(end - 4)..end].iter_mut().for_each(|x| *x = 0);
        }
    }

    /// 00FD - EXIT  
    /// Exit CHIP interpreter.
    fn op_00fd(&mut self) {
        self.is_halted = true;
    }

    /// 00FE - LOW  
    /// Disable extended screen mode.
    fn op_00fe(&mut self) {
        self.is_highres = false;
        self.op_00e0();
    }

    /// 00FF - HIGH  
    /// Enable extended screen mode for full-screen graphics.
    fn op_00ff(&mut self) {
        self.is_highres = true;
        self.op_00e0();
    }

    /// Fx30 - LD HF, Vx  
    /// Point I to 10-byte font sprite for VX (0..F)
    fn op_fx30(&mut self, x: usize) {
        self.i = (self.register[x] as usize * 10) + 80;
    }

    /// Fx75 - LD R, Vx  
    /// Store V0..VX in RPL user flags (X <= 7)
    fn op_fx75(&mut self, x: usize) {
        self.flag_regs[0..=x].copy_from_slice(&self.register[0..=x]);
    }

    /// Fx85 - LD Vx, R  
    /// Read V0..VX from RPL user flags (X <= 7)
    fn op_fx85(&mut self, x: usize) {
        self.register[0..=x].copy_from_slice(&self.flag_regs[0..=x]);
    }
}
