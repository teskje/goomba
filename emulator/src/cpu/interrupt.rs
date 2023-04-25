use code::{Cnd, SrcW};

use crate::bits::BitsExt;

use super::execute::Op;
use super::Cpu;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Interrupt {
    VBlank,
    LcdStat,
    Timer,
    Serial,
    Joypad,
}

impl Interrupt {
    const VALUES: [Self; 5] = [
        Self::VBlank,
        Self::LcdStat,
        Self::Timer,
        Self::Serial,
        Self::Joypad,
    ];

    pub(super) fn bit(&self) -> u8 {
        match self {
            Self::VBlank => 0,
            Self::LcdStat => 1,
            Self::Timer => 2,
            Self::Serial => 3,
            Self::Joypad => 4,
        }
    }

    fn vector(&self) -> u16 {
        use Interrupt::*;
        match self {
            VBlank => 0x0040,
            LcdStat => 0x0048,
            Timer => 0x0050,
            Serial => 0x0058,
            Joypad => 0x0060,
        }
    }
}

impl Cpu<'_> {
    pub(super) fn handle_interrupt(&mut self) {
        let Some(irpt) = self.pending_interrupt() else { return };

        self.s.cpu.halt = false;

        if self.s.cpu.ime {
            self.clear_interrupt(irpt);
            self.s.cpu.ime = false;

            let target = SrcW::Imm(irpt.vector());
            self.push_ops([Op::ReadW(target), Op::Call(Cnd::None)]);

            self.must_yield();
        }
    }

    fn pending_interrupt(&self) -> Option<Interrupt> {
        let interrupts = &self.s.cpu.interrupts;
        let pending_bits = interrupts.flag_bits & interrupts.enable_bits;
        let pending_bits = pending_bits.bits(0..=4);

        Interrupt::VALUES
            .into_iter()
            .find(|irpt| pending_bits.bit(irpt.bit()))
    }

    fn clear_interrupt(&mut self, irpt: Interrupt) {
        self.s.cpu.interrupts.reset_flag(irpt);
    }
}
