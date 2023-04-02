//! Reference:
//!   * https://gbdev.io/gb-opcodes/optables/
//!   * https://gbdev.io/pandocs/CPU_Instruction_Set.html

use crate::inst::*;
use crate::Error;

pub fn decode(op: &[u8]) -> Result<Inst, Error> {
    match op {
        [] => Err(Error::TooFewBytes),
        [a] => decode1(*a),
        [a, b] => decode2(*a, *b),
        [a, b, c] => decode3(*a, *b, *c),
        _ => Err(Error::TooManyBytes),
    }
}

fn decode1(op: u8) -> Result<Inst, Error> {
    use Inst::*;
    use Reg::*;
    use RegW::*;

    let dst = Dst::from_code(op >> 3 & 0b111);
    let src = Src::from_code(op & 0b111);

    let inst = match op {
        0x00 => Nop,
        0x02 => Ld(Dst::reg_ref(BC), Src::Reg(A)),
        0x03 => Incw(DstW::Reg(BC)),
        0x04 => Inc(Dst::Reg(B)),
        0x05 => Dec(Dst::Reg(B)),
        0x07 => RlcA,
        0x09 => AddHL(BC),
        0x0a => Ld(Dst::Reg(A), Src::reg_ref(BC)),
        0x0b => Decw(DstW::Reg(BC)),
        0x0c => Inc(Dst::Reg(C)),
        0x0d => Dec(Dst::Reg(C)),
        0x0f => RrcA,
        0x10 => Stop,
        0x12 => Ld(Dst::reg_ref(DE), Src::Reg(A)),
        0x13 => Incw(DstW::Reg(DE)),
        0x14 => Inc(Dst::Reg(D)),
        0x15 => Dec(Dst::Reg(D)),
        0x17 => RlA,
        0x19 => AddHL(DE),
        0x1a => Ld(Dst::Reg(A), Src::reg_ref(DE)),
        0x1b => Decw(DstW::Reg(DE)),
        0x1c => Inc(Dst::Reg(E)),
        0x1d => Dec(Dst::Reg(E)),
        0x1f => RrA,
        0x22 => Ldi(Dst::reg_ref(HL), Src::Reg(A)),
        0x23 => Incw(DstW::Reg(HL)),
        0x24 => Inc(Dst::Reg(H)),
        0x25 => Dec(Dst::Reg(H)),
        0x27 => Daa,
        0x29 => AddHL(HL),
        0x2a => Ldi(Dst::Reg(A), Src::reg_ref(HL)),
        0x2b => Decw(DstW::Reg(HL)),
        0x2c => Inc(Dst::Reg(L)),
        0x2d => Dec(Dst::Reg(L)),
        0x2f => Cpl,
        0x32 => Ldd(Dst::reg_ref(HL), Src::Reg(A)),
        0x33 => Incw(DstW::Reg(SP)),
        0x34 => Inc(Dst::reg_ref(HL)),
        0x35 => Dec(Dst::reg_ref(HL)),
        0x37 => Scf,
        0x39 => AddHL(SP),
        0x3a => Ldd(Dst::Reg(A), Src::reg_ref(HL)),
        0x3b => Decw(DstW::Reg(SP)),
        0x3c => Inc(Dst::Reg(A)),
        0x3d => Dec(Dst::Reg(A)),
        0x3f => Ccf,
        0x76 => Halt,
        0x40..=0x7f => Ld(dst, src),
        0x80..=0x87 => Add(src),
        0x88..=0x8f => Adc(src),
        0x90..=0x97 => Sub(src),
        0x98..=0x9f => Sbc(src),
        0xa0..=0xa7 => And(src),
        0xa8..=0xaf => Xor(src),
        0xb0..=0xb7 => Or(src),
        0xb8..=0xbf => Cp(src),
        0xc0 => Ret(Cnd::NZ),
        0xc1 => Pop(BC),
        0xc5 => Push(BC),
        0xc7 => Rst(0x00),
        0xc8 => Ret(Cnd::Z),
        0xc9 => Ret(Cnd::None),
        0xcf => Rst(0x08),
        0xd0 => Ret(Cnd::NC),
        0xd1 => Pop(DE),
        0xd5 => Push(DE),
        0xd7 => Rst(0x10),
        0xd8 => Ret(Cnd::C),
        0xd9 => Reti,
        0xdf => Rst(0x18),
        0xe1 => Pop(HL),
        0xe2 => Ld(Dst::Mem(Ref::RegH(C)), Src::Reg(A)),
        0xe5 => Push(HL),
        0xe7 => Rst(0x20),
        0xe9 => JpHL,
        0xef => Rst(0x28),
        0xf1 => Pop(AF),
        0xf2 => Ld(Dst::Reg(A), Src::Mem(Ref::RegH(C))),
        0xf3 => Di,
        0xf5 => Push(AF),
        0xf7 => Rst(0x30),
        0xf9 => Ldw(DstW::Reg(SP), SrcW::Reg(HL)),
        0xfb => Ei,
        0xff => Rst(0x38),
        0xd3 | 0xdb | 0xdd | 0xe3 | 0xe4 | 0xeb | 0xec | 0xed | 0xf4 | 0xfc | 0xfd => {
            return Err(Error::InvalidOpcode(op))
        }
        _ => return Err(Error::TooFewBytes),
    };
    Ok(inst)
}

fn decode2(op1: u8, op2: u8) -> Result<Inst, Error> {
    use Inst::*;
    use Reg::*;
    use RegW::*;

    let src = Src::Imm(op2);

    let inst = match op1 {
        0x06 => Ld(Dst::Reg(B), src),
        0x0e => Ld(Dst::Reg(C), src),
        0x16 => Ld(Dst::Reg(D), src),
        0x18 => Jr(Cnd::None, src),
        0x1e => Ld(Dst::Reg(E), src),
        0x20 => Jr(Cnd::NZ, src),
        0x26 => Ld(Dst::Reg(H), src),
        0x28 => Jr(Cnd::Z, src),
        0x2e => Ld(Dst::Reg(L), src),
        0x30 => Jr(Cnd::NC, src),
        0x36 => Ld(Dst::reg_ref(HL), src),
        0x38 => Jr(Cnd::C, src),
        0x3e => Ld(Dst::Reg(A), src),
        0xc6 => Add(src),
        0xcb => decode_prefixed(op2),
        0xce => Adc(src),
        0xd6 => Sub(src),
        0xde => Sbc(src),
        0xe0 => Ld(Dst::Mem(Ref::ImmH(op2)), Src::Reg(A)),
        0xe6 => And(src),
        0xe8 => AddSP(src),
        0xee => Xor(src),
        0xf0 => Ld(Dst::Reg(A), Src::Mem(Ref::ImmH(op2))),
        0xf6 => Or(src),
        0xf8 => LdSPOff(src),
        0xfe => Cp(src),
        0x01 | 0x08 | 0x11 | 0x21 | 0x31 | 0xc2 | 0xc3 | 0xc4 | 0xca | 0xcc | 0xcd | 0xd2
        | 0xd4 | 0xda | 0xdc | 0xea | 0xfa => return Err(Error::TooFewBytes),
        _ => return Err(Error::TooManyBytes),
    };
    Ok(inst)
}

fn decode3(op1: u8, op2: u8, op3: u8) -> Result<Inst, Error> {
    use Inst::*;
    use Reg::*;
    use RegW::*;

    let imm = u16::from_le_bytes([op2, op3]);
    let src = SrcW::Imm(imm);

    let inst = match op1 {
        0x01 => Ldw(DstW::Reg(BC), src),
        0x08 => Ldw(DstW::imm_ref(imm), SrcW::Reg(SP)),
        0x11 => Ldw(DstW::Reg(DE), src),
        0x21 => Ldw(DstW::Reg(HL), src),
        0x31 => Ldw(DstW::Reg(SP), src),
        0xc2 => Jp(Cnd::NZ, src),
        0xc3 => Jp(Cnd::None, src),
        0xc4 => Call(Cnd::NZ, src),
        0xca => Jp(Cnd::Z, src),
        0xcc => Call(Cnd::Z, src),
        0xcd => Call(Cnd::None, src),
        0xd2 => Jp(Cnd::NC, src),
        0xd4 => Call(Cnd::NC, src),
        0xda => Jp(Cnd::C, src),
        0xdc => Call(Cnd::C, src),
        0xea => Ld(Dst::imm_ref(imm), Src::Reg(A)),
        0xfa => Ld(Dst::Reg(A), Src::imm_ref(imm)),
        _ => return Err(Error::TooManyBytes),
    };
    Ok(inst)
}

fn decode_prefixed(op: u8) -> Inst {
    use Inst::*;

    let bit = op >> 3 & 0b111;
    let dst = Dst::from_code(op & 0b111);

    match op {
        0x00..=0x07 => Rlc(dst),
        0x08..=0x0f => Rrc(dst),
        0x10..=0x17 => Rl(dst),
        0x18..=0x1f => Rr(dst),
        0x20..=0x27 => Sla(dst),
        0x28..=0x2f => Sra(dst),
        0x30..=0x37 => Swap(dst),
        0x38..=0x3f => Srl(dst),
        0x40..=0x7f => Bit(bit, dst),
        0x80..=0xbf => Res(bit, dst),
        0xc0..=0xff => Set(bit, dst),
    }
}

impl Dst {
    fn from_code(c: u8) -> Self {
        match c {
            0x00 => Self::Reg(Reg::B),
            0x01 => Self::Reg(Reg::C),
            0x02 => Self::Reg(Reg::D),
            0x03 => Self::Reg(Reg::E),
            0x04 => Self::Reg(Reg::H),
            0x05 => Self::Reg(Reg::L),
            0x06 => Self::Mem(Ref::Reg(RegW::HL)),
            0x07 => Self::Reg(Reg::A),
            _ => panic!("invalid dst code: {c}"),
        }
    }
}

impl Src {
    fn from_code(c: u8) -> Self {
        match c {
            0x00 => Self::Reg(Reg::B),
            0x01 => Self::Reg(Reg::C),
            0x02 => Self::Reg(Reg::D),
            0x03 => Self::Reg(Reg::E),
            0x04 => Self::Reg(Reg::H),
            0x05 => Self::Reg(Reg::L),
            0x06 => Self::Mem(Ref::Reg(RegW::HL)),
            0x07 => Self::Reg(Reg::A),
            _ => panic!("invalid src code: {c}"),
        }
    }
}
