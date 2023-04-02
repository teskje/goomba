#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Inst {
    Ld(Dst, Src),
    Ldi(Dst, Src),
    Ldd(Dst, Src),
    Ldw(DstW, SrcW),
    LdSPOff(Src),
    Push(RegW),
    Pop(RegW),
    Add(Src),
    AddHL(RegW),
    AddSP(Src),
    Adc(Src),
    Sub(Src),
    Sbc(Src),
    And(Src),
    Or(Src),
    Xor(Src),
    Inc(Dst),
    Incw(DstW),
    Dec(Dst),
    Decw(DstW),
    Cp(Src),
    Daa,
    Cpl,
    Jr(Cnd, Src),
    Jp(Cnd, SrcW),
    JpHL,
    Call(Cnd, SrcW),
    Ret(Cnd),
    Reti,
    Rst(u8),
    Rl(Dst),
    RlA,
    Rlc(Dst),
    RlcA,
    Rr(Dst),
    RrA,
    Rrc(Dst),
    RrcA,
    Sla(Dst),
    Sra(Dst),
    Srl(Dst),
    Swap(Dst),
    Bit(u8, Dst),
    Res(u8, Dst),
    Set(u8, Dst),
    Nop,
    Halt,
    Stop,
    Scf,
    Ccf,
    Di,
    Ei,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum Reg {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum RegW {
    AF,
    BC,
    DE,
    HL,
    SP,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum Ref {
    Reg(RegW),
    Imm(u16),
    RegH(Reg),
    ImmH(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum Dst {
    Reg(Reg),
    Mem(Ref),
}

impl Dst {
    pub(crate) fn reg_ref(reg: RegW) -> Self {
        Self::Mem(Ref::Reg(reg))
    }

    pub(crate) fn imm_ref(imm: u16) -> Self {
        Self::Mem(Ref::Imm(imm))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum DstW {
    Reg(RegW),
    Mem(Ref),
}

impl DstW {
    pub(crate) fn imm_ref(imm: u16) -> Self {
        Self::Mem(Ref::Imm(imm))
    }
}

impl From<RegW> for DstW {
    fn from(reg: RegW) -> Self {
        Self::Reg(reg)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum Src {
    Reg(Reg),
    Mem(Ref),
    Imm(u8),
}

impl Src {
    pub(crate) fn reg_ref(reg: RegW) -> Self {
        Self::Mem(Ref::Reg(reg))
    }

    pub(crate) fn imm_ref(imm: u16) -> Self {
        Self::Mem(Ref::Imm(imm))
    }
}

impl From<Dst> for Src {
    fn from(dst: Dst) -> Self {
        match dst {
            Dst::Reg(reg) => Self::Reg(reg),
            Dst::Mem(ref_) => Self::Mem(ref_),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum SrcW {
    Reg(RegW),
    Mem(Ref),
    Imm(u16),
}

impl From<DstW> for SrcW {
    fn from(dst: DstW) -> Self {
        match dst {
            DstW::Reg(reg) => Self::Reg(reg),
            DstW::Mem(ref_) => Self::Mem(ref_),
        }
    }
}

impl From<RegW> for SrcW {
    fn from(reg: RegW) -> Self {
        Self::Reg(reg)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum Cnd {
    None,
    Z,
    NZ,
    C,
    NC,
}
