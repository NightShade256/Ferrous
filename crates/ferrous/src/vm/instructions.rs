use super::Vm;

impl Vm {
    /// Clear the screen.
    #[inline]
    pub(super) fn op_00e0(&mut self) {
        self.vram.fill(0x00);
    }

    /// Jump to address `NNN`.
    #[inline]
    pub(super) fn op_1nnn(&mut self, nnn: u16) {
        self.pc = nnn;
    }

    /// Skip the following instruction if the value of register `VX` equals `NN`.
    #[inline]
    pub(super) fn op_3xnn(&mut self, x: u8, nn: u8) {
        if self.reg[x as usize] == nn {
            self.pc = self.pc.wrapping_add(2);
        }
    }

    /// Skip the following instruction if the value of register `VX` is not equal to `NN`.
    #[inline]
    pub(super) fn op_4xnn(&mut self, x: u8, nn: u8) {
        if self.reg[x as usize] != nn {
            self.pc = self.pc.wrapping_add(2);
        }
    }

    /// Skip the following instruction if the value of register `VX` is equal to the value
    /// of register `VY`.
    #[inline]
    pub(super) fn op_5xy0(&mut self, x: u8, y: u8) {
        if self.reg[x as usize] == self.reg[y as usize] {
            self.pc = self.pc.wrapping_add(2);
        }
    }

    /// Store number `NN` in register `VX`.
    #[inline]
    pub(super) fn op_6xnn(&mut self, x: u8, nn: u8) {
        self.reg[x as usize] = nn;
    }

    /// Add the value `NN` to register `VX`.
    #[inline]
    pub(super) fn op_7xnn(&mut self, x: u8, nn: u8) {
        self.reg[x as usize] = self.reg[x as usize].wrapping_add(nn);
    }

    /// Store memory address `NNN` in register `I`.
    #[inline]
    pub(super) fn op_annn(&mut self, nnn: u16) {
        self.id = nnn;
    }
}
