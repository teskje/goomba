use std::fmt;

use crate::inst::*;

impl fmt::Display for Inst {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Inst::*;
        match self {
            Ld(d, s) => write!(f, "ld {d}, {s}"),
            Ldi(d, s) => write!(f, "ldi {d}, {s}"),
            Ldd(d, s) => write!(f, "ldd {d}, {s}"),
            Ldw(d, s) => write!(f, "ld {d}, {s}"),
            LdSPOff(s) => write!(f, "ld hl, sp + {s}"),
            Push(r) => write!(f, "push {r}"),
            Pop(r) => write!(f, "pop {r}"),
            Add(s) => write!(f, "add a, {s}"),
            AddHL(r) => write!(f, "add hl, {r}"),
            AddSP(s) => write!(f, "add sp, {s}"),
            Adc(s) => write!(f, "adc a, {s}"),
            Sub(s) => write!(f, "sub a, {s}"),
            Sbc(s) => write!(f, "sbc a, {s}"),
            And(s) => write!(f, "and a, {s}"),
            Or(s) => write!(f, "or a, {s}"),
            Xor(s) => write!(f, "xor a, {s}"),
            Inc(d) => write!(f, "inc {d}"),
            Incw(r) => write!(f, "inc {r}"),
            Dec(d) => write!(f, "dec {d}"),
            Decw(r) => write!(f, "dec {r}"),
            Cp(s) => write!(f, "cp a, {s}"),
            Daa => f.write_str("daa"),
            Cpl => f.write_str("cpl"),
            Jr(Cnd::None, s) => write!(f, "jr {s}"),
            Jr(c, s) => write!(f, "jr {c}, {s}"),
            Jp(Cnd::None, s) => write!(f, "jp {s}"),
            Jp(c, s) => write!(f, "jp {c}, {s}"),
            JpHL => write!(f, "jp hl"),
            Call(Cnd::None, s) => write!(f, "call {s}"),
            Call(c, s) => write!(f, "call {c}, {s}"),
            Ret(Cnd::None) => f.write_str("ret"),
            Ret(c) => write!(f, "ret {c}"),
            Reti => f.write_str("reti"),
            Rst(s) => write!(f, "rst {s}"),
            Rl(d) => write!(f, "rl {d}"),
            RlA => f.write_str("rla"),
            Rlc(d) => write!(f, "rlc {d}"),
            RlcA => f.write_str("rlca"),
            Rr(d) => write!(f, "rr {d}"),
            RrA => f.write_str("rra"),
            Rrc(d) => write!(f, "rrc {d}"),
            RrcA => f.write_str("rrca"),
            Sla(d) => write!(f, "sla {d}"),
            Sra(d) => write!(f, "sra {d}"),
            Srl(d) => write!(f, "srl {d}"),
            Swap(d) => write!(f, "swap {d}"),
            Bit(b, d) => write!(f, "bit {b}, {d}"),
            Res(b, d) => write!(f, "res {b}, {d}"),
            Set(b, d) => write!(f, "set {b}, {d}"),
            Nop => f.write_str("nop"),
            Halt => f.write_str("halt"),
            Stop => f.write_str("stop"),
            Scf => f.write_str("scf"),
            Ccf => f.write_str("ccf"),
            Di => f.write_str("di"),
            Ei => f.write_str("ei"),
        }
    }
}

impl fmt::Display for Reg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Reg::*;
        match self {
            A => f.write_str("a"),
            B => f.write_str("b"),
            C => f.write_str("c"),
            D => f.write_str("d"),
            E => f.write_str("e"),
            H => f.write_str("h"),
            L => f.write_str("l"),
        }
    }
}

impl fmt::Display for RegW {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use RegW::*;
        match self {
            AF => f.write_str("af"),
            BC => f.write_str("bc"),
            DE => f.write_str("de"),
            HL => f.write_str("hl"),
            SP => f.write_str("sp"),
        }
    }
}

impl fmt::Display for Ref {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Ref::Reg(reg) => write!(f, "[{reg}]"),
            Ref::Imm(imm) => write!(f, "[{imm:#04x}]"),
            Ref::RegH(reg) => write!(f, "[^{reg}]"),
            Ref::ImmH(imm) => write!(f, "[^{imm:#02x}]"),
        }
    }
}

impl fmt::Display for Dst {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Dst::*;
        match self {
            Reg(reg) => fmt::Display::fmt(reg, f),
            Mem(ref_) => fmt::Display::fmt(ref_, f),
        }
    }
}

impl fmt::Display for DstW {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use DstW::*;
        match self {
            Reg(reg) => fmt::Display::fmt(reg, f),
            Mem(ref_) => fmt::Display::fmt(ref_, f),
        }
    }
}

impl fmt::Display for Src {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Src::*;
        match self {
            Reg(reg) => fmt::Display::fmt(reg, f),
            Mem(ref_) => fmt::Display::fmt(ref_, f),
            Imm(imm) => write!(f, "{imm:#02x}"),
        }
    }
}

impl fmt::Display for SrcW {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use SrcW::*;
        match self {
            Reg(reg) => fmt::Display::fmt(reg, f),
            Mem(ref_) => fmt::Display::fmt(ref_, f),
            Imm(imm) => write!(f, "{imm:#04x}"),
        }
    }
}

impl fmt::Display for Cnd {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Cnd::*;
        match self {
            None => f.write_str("_"),
            Z => f.write_str("z"),
            NZ => f.write_str("nz"),
            C => f.write_str("c"),
            NC => f.write_str("nc"),
        }
    }
}
