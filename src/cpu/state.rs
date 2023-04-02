use crate::bits::BitsExt;

use super::execute::Op;
use super::interrupt::Interrupt;

#[derive(Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct CpuState {
    pub interrupts: InterruptState,

    pub(super) registers: Registers,
    pub(super) pc: u16,
    pub(super) ime: bool,
    pub(super) halt: bool,

    pub(super) todo: Vec<Op>,
    pub(super) stash: Vec<u8>,
}

impl Default for CpuState {
    fn default() -> Self {
        Self {
            interrupts: Default::default(),
            registers: Default::default(),
            pc: 0x0100,
            ime: false,
            halt: false,
            todo: Default::default(),
            stash: Default::default(),
        }
    }
}

#[derive(Debug, Default)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct InterruptState {
    pub(super) enable_bits: u8,
    pub(super) flag_bits: u8,
}

impl InterruptState {
    pub fn read_enable(&self) -> u8 {
        self.enable_bits
    }

    pub fn write_enable(&mut self, value: u8) {
        self.enable_bits = value;
    }

    pub fn read_flag(&self) -> u8 {
        self.flag_bits | 0xe0
    }

    pub fn write_flag(&mut self, value: u8) {
        self.flag_bits = value;
    }

    pub fn set_flag(&mut self, irpt: Interrupt) {
        let bit = irpt.bit();
        self.flag_bits.set_bit(bit);
    }

    pub(super) fn reset_flag(&mut self, irpt: Interrupt) {
        let bit = irpt.bit();
        self.flag_bits.reset_bit(bit);
    }
}

#[derive(Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub(super) struct Registers {
    pub(super) a: u8,
    pub(super) b: u8,
    pub(super) c: u8,
    pub(super) d: u8,
    pub(super) e: u8,
    pub(super) h: u8,
    pub(super) l: u8,
    pub(super) sp: u16,
    pub(super) flags: Flags,
}

impl Registers {
    pub(super) fn af(&self) -> u16 {
        u16::from_be_bytes([self.a, self.flags.to_u8()])
    }

    pub(super) fn bc(&self) -> u16 {
        u16::from_be_bytes([self.b, self.c])
    }

    pub(super) fn de(&self) -> u16 {
        u16::from_be_bytes([self.d, self.e])
    }

    pub(super) fn hl(&self) -> u16 {
        u16::from_be_bytes([self.h, self.l])
    }

    pub(super) fn set_af(&mut self, data: u16) {
        let [a, f] = data.to_be_bytes();
        self.a = a;
        self.flags = Flags::from_u8(f);
    }

    pub(super) fn set_bc(&mut self, data: u16) {
        [self.b, self.c] = data.to_be_bytes();
    }

    pub(super) fn set_de(&mut self, data: u16) {
        [self.d, self.e] = data.to_be_bytes();
    }

    pub(super) fn set_hl(&mut self, data: u16) {
        [self.h, self.l] = data.to_be_bytes();
    }

    pub(super) fn inc_hl(&mut self) {
        self.set_hl(self.hl().wrapping_add(1));
    }

    pub(super) fn dec_hl(&mut self) {
        self.set_hl(self.hl().wrapping_sub(1));
    }

    pub(super) fn read(&self, reg: code::Reg) -> u8 {
        use code::Reg::*;
        match reg {
            A => self.a,
            B => self.b,
            C => self.c,
            D => self.d,
            E => self.e,
            H => self.h,
            L => self.l,
        }
    }

    pub(super) fn read_wide(&self, reg: code::RegW) -> u16 {
        use code::RegW::*;
        match reg {
            AF => self.af(),
            BC => self.bc(),
            DE => self.de(),
            HL => self.hl(),
            SP => self.sp,
        }
    }

    pub(super) fn write(&mut self, reg: code::Reg, data: u8) {
        use code::Reg::*;
        match reg {
            A => self.a = data,
            B => self.b = data,
            C => self.c = data,
            D => self.d = data,
            E => self.e = data,
            H => self.h = data,
            L => self.l = data,
        };
    }

    pub(super) fn write_wide(&mut self, reg: code::RegW, data: u16) {
        use code::RegW::*;
        match reg {
            AF => self.set_af(data),
            BC => self.set_bc(data),
            DE => self.set_de(data),
            HL => self.set_hl(data),
            SP => self.sp = data,
        };
    }
}

impl Default for Registers {
    fn default() -> Self {
        // Initial register state after the boot ROM (which we don't bother emulating), as
        // documented in [https://gbdev.io/pandocs/Power_Up_Sequence.html].
        Self {
            a: 0x01,
            b: 0x00,
            c: 0x13,
            d: 0x00,
            e: 0xd8,
            h: 0x01,
            l: 0x4d,
            sp: 0xfffe,
            flags: Flags {
                z: true,
                n: false,
                h: false,
                c: false,
            },
        }
    }
}

#[derive(Debug, Default)]
#[derive(serde::Serialize, serde::Deserialize)]
pub(super) struct Flags {
    pub z: bool,
    pub n: bool,
    pub h: bool,
    pub c: bool,
}

impl Flags {
    pub(super) fn to_u8(&self) -> u8 {
        let mut x = 0;
        let bits = [(self.z, 7), (self.n, 6), (self.h, 5), (self.c, 4)];
        for (flag, pos) in bits {
            if flag {
                x.set_bit(pos);
            }
        }
        x
    }

    pub(super) fn from_u8(x: u8) -> Self {
        Self {
            z: x.bit(7),
            n: x.bit(6),
            h: x.bit(5),
            c: x.bit(4),
        }
    }
}
