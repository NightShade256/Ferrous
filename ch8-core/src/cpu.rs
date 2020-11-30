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
//! of a Chip 8 interpreter.

use crate::font::FONT_SPRITES;

/// Implementation of a Chip-8 interpreter.
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

    /// Sixteen general purpose registers.
    /// Conventionally named as V0 to VF.
    /// VF is a special register, that is used as a flag.
    pub register: [u8; 0x10],

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
    pub vram: Vec<u8>,

    /// Keypad Representation; Conveys whether a key is pressed (true) or not pressed
    /// (false) currently.
    pub keypad: [bool; 0x10],
}

// General Methods
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

        Self {
            memory,
            register: [0; 0x10],
            pc: 0x200, // All programs start from 0x200.
            sp: 0,
            i: 0,
            dt: 0,
            st: 0,
            vram: vec![0; 64 * 32],
            keypad: [false; 0x10],
        }
    }

    /// Reset the interpreter to its initial state.
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

        self.register = [0; 0x10];

        self.pc = 0x200;
        self.sp = 0;
        self.i = 0;
        self.dt = 0;
        self.st = 0;

        self.vram.iter_mut().for_each(|x| *x = 0);
        self.keypad = [false; 0x10];
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
            return Err("ROM\'s length is larger than the permitted 3584 bytes.".to_string());
        }

        // Copy the ROM buffer.
        self.memory[0x200..0x200 + buffer.len()].copy_from_slice(&buffer);

        Ok(())
    }
}
