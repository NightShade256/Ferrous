#[cfg(feature = "savestates")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "savestates", derive(Deserialize, Serialize))]
pub(crate) struct Vm {
    /// The address of the sprite that has to be drawn with the DRW
    /// CPU instruction.
    id: u16,

    /// Indicates whether the interpreter is currently in high resolution
    /// mode.
    is_highres: bool,

    /// The address of the next instruction to be executed.
    pc: u16,

    /// 16 byte-wide general purpose registers. The 15th register `VF`
    /// is used as a flag for overflow and more.
    reg: [u8; 0x10],

    /// The address of the next empty stack slot.
    sp: u16,

    /// 32 kiB of VRAM, with enough space to store a 128 by 64
    /// framebuffer for the screen.
    vram: Vec<u8>,

    /// 4 kiB of WRAM, with the initial 0x200 bytes being reserved
    /// for use by the interpreter.
    wram: Vec<u8>,
}

impl Vm {
    /// Create a new `Vm` instance.
    pub fn new() -> Self {
        Self {
            id: 0x0000,
            is_highres: false,
            pc: 0x0000,
            reg: [0x00; 0x10],
            sp: 0x0000,
            vram: vec![0x00; 0x2000],
            wram: vec![0x00; 0x1000],
        }
    }

    /// Load the given ROM into main memory for execution.
    pub fn load_rom(&mut self, rom: Vec<u8>) -> Result<(), &'static str> {
        if rom.len() <= 3584 {
            self.wram[0x200..(0x200 + rom.len())].copy_from_slice(&rom);
            Ok(())
        } else {
            Err("ROM length is greater than the permissible 3584 bytes")
        }
    }

    /// Get the size of the display as a width and height tuple.
    pub fn get_display_size(&self) -> (u32, u32) {
        if self.is_highres {
            (128, 64)
        } else {
            (64, 32)
        }
    }
}

impl Vm {
    /// Clear the screen.
    fn op_00e0(&mut self) {
        self.vram.fill(0x00);
    }

    /// Jump to address `NNN`.
    fn op_1nnn(&mut self, nnn: u16) {
        self.pc = nnn;
    }

    /// Skip the following instruction if the value of register `VX` equals `NN`.
    fn op_3xnn(&mut self, x: u8, nn: u8) {
        if self.reg[x as usize] == nn {
            self.pc = self.pc.wrapping_add(2);
        }
    }

    /// Skip the following instruction if the value of register `VX` is not equal to `NN`.
    fn op_4xnn(&mut self, x: u8, nn: u8) {
        if self.reg[x as usize] != nn {
            self.pc = self.pc.wrapping_add(2);
        }
    }

    /// Skip the following instruction if the value of register `VX` is equal to the value
    /// of register `VY`.
    fn op_5xy0(&mut self, x: u8, y: u8) {
        if self.reg[x as usize] == self.reg[y as usize] {
            self.pc = self.pc.wrapping_add(2);
        }
    }

    /// Store number `NN` in register `VX`.
    fn op_6xnn(&mut self, x: u8, nn: u8) {
        self.reg[x as usize] = nn;
    }

    /// Add the value `NN` to register `VX`.
    fn op_7xnn(&mut self, x: u8, nn: u8) {
        self.reg[x as usize] = self.reg[x as usize].wrapping_add(nn);
    }

    /// Store memory address `NNN` in register `I`.
    fn op_annn(&mut self, nnn: u16) {
        self.id = nnn;
    }
}
