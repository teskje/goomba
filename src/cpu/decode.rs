use code::{Cnd, DstW, Inst, SrcW};
use log::trace;

use super::execute::Op;
use super::Cpu;

impl Cpu<'_> {
    pub(super) fn decode(&mut self, inst: Inst) {
        trace!("{inst}");

        debug_assert!(self.s.cpu.stash.is_empty(), "stash leak");

        macro_rules! ops {
            ( $($x:expr),+ ) => {
                self.push_ops([$($x),+])
            };
        }

        use Op::*;
        match inst {
            Inst::Ld(d, s) => ops!(Read(s), Write(d)),
            Inst::Ldi(d, s) => ops!(Read(s), Write(d), IncHl),
            Inst::Ldd(d, s) => ops!(Read(s), Write(d), DecHl),
            Inst::Ldw(d, s) if reg_to_reg(d, s) => ops!(ReadW(s), WriteW(d), Wait),
            Inst::Ldw(d, s) => ops!(ReadW(s), WriteW(d)),
            Inst::LdSPOff(s) => ops!(Read(s), LdSPOff, Wait),
            Inst::Push(r) => ops!(ReadW(r.into()), PushW, Wait),
            Inst::Pop(r) => ops!(PopW, WriteW(r.into())),
            Inst::Add(s) => ops!(Read(s), Add),
            Inst::AddHL(r) => ops!(AddHL(r), Wait),
            Inst::AddSP(s) => ops!(Read(s), AddSP, Wait, Wait),
            Inst::Adc(s) => ops!(Read(s), Adc),
            Inst::Sub(s) => ops!(Read(s), Sub),
            Inst::Sbc(s) => ops!(Read(s), Sbc),
            Inst::And(s) => ops!(Read(s), And),
            Inst::Or(s) => ops!(Read(s), Or),
            Inst::Xor(s) => ops!(Read(s), Xor),
            Inst::Inc(d) => ops!(Read(d.into()), Inc, Write(d)),
            Inst::Incw(d) => ops!(ReadW(d.into()), IncW, WriteW(d), Wait),
            Inst::Dec(d) => ops!(Read(d.into()), Dec, Write(d)),
            Inst::Decw(d) => ops!(ReadW(d.into()), DecW, WriteW(d), Wait),
            Inst::Cp(s) => ops!(Read(s), Cp),
            Inst::Daa => ops!(Daa),
            Inst::Cpl => ops!(Cpl),
            Inst::Jr(c, t) => ops!(Read(t), JumpR(c)),
            Inst::Jp(c, t) => ops!(ReadW(t), Jump(c)),
            Inst::JpHL => ops!(JumpHL),
            Inst::Call(c, t) => ops!(ReadW(t), Call(c)),
            Inst::Ret(c) => ops!(Return(c)),
            Inst::Reti => ops!(Return(Cnd::None), Ei),
            Inst::Rst(v) => ops!(Reset(v), PushW),
            Inst::Rl(d) => ops!(Read(d.into()), Rl, Write(d)),
            Inst::RlA => ops!(RlA),
            Inst::Rlc(d) => ops!(Read(d.into()), Rlc, Write(d)),
            Inst::RlcA => ops!(RlcA),
            Inst::Rr(d) => ops!(Read(d.into()), Rr, Write(d)),
            Inst::RrA => ops!(RrA),
            Inst::Rrc(d) => ops!(Read(d.into()), Rrc, Write(d)),
            Inst::RrcA => ops!(RrcA),
            Inst::Sla(d) => ops!(Read(d.into()), Sla, Write(d)),
            Inst::Sra(d) => ops!(Read(d.into()), Sra, Write(d)),
            Inst::Srl(d) => ops!(Read(d.into()), Srl, Write(d)),
            Inst::Swap(d) => ops!(Read(d.into()), Swap, Write(d)),
            Inst::Bit(b, d) => ops!(Read(d.into()), Bit(b)),
            Inst::Res(b, d) => ops!(Read(d.into()), Res(b), Write(d)),
            Inst::Set(b, d) => ops!(Read(d.into()), Set(b), Write(d)),
            Inst::Nop => (),
            Inst::Halt => ops!(Halt),
            Inst::Stop => todo!("stop"),
            Inst::Scf => ops!(Scf),
            Inst::Ccf => ops!(Ccf),
            Inst::Di => ops!(Di),
            Inst::Ei => ops!(Ei),
        }
    }
}

fn reg_to_reg(dst: DstW, src: SrcW) -> bool {
    matches!(dst, DstW::Reg(_)) && matches!(src, SrcW::Reg(_))
}
