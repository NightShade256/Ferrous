#[cfg(feature = "savestates")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "savestates", derive(Deserialize, Serialize))]
pub(crate) struct Vm {
    /// 4 kiB of WRAM, with the initial 0x200 bytes being reserved
    /// for use by the interpreter.
    wram: Vec<u8>,

    /// 32 kiB of VRAM, with enough space to store a 128 by 64
    /// framebuffer for the screen.
    vram: Vec<u8>,

    /// The address of the next instruction to be executed.
    pc: u16,

    /// The address of the next empty stack slot.
    sp: u16,
}

impl Vm {
    /// Create a new `Vm` instance.
    pub fn new() -> Self {
        Self {
            wram: vec![0x00; 0x1000],
            vram: vec![0x00; 0x8000],
            pc: 0x0000,
            sp: 0x0000,
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
}

impl Vm {
    fn op_00e0(&mut self) {
        self.vram.fill(0x00);
    }
}
