use anyhow::Result;
use code::{Cnd, Dst, DstW, Ref, RegW, Src, SrcW};

use crate::bits::BitsExt;

use super::state::Flags;
use super::Cpu;

#[derive(Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub(super) enum Op {
    FetchDecode,
    Read(Src),
    Write(Dst),
    ReadW(SrcW),
    WriteW(DstW),
    LdSPOff,
    Push,
    Pop,
    PushW,
    PopW,
    Add,
    AddHL(RegW),
    AddSP,
    Adc,
    Sub,
    Sbc,
    And,
    Or,
    Xor,
    Cp,
    Daa,
    Cpl,
    Inc,
    Dec,
    IncW,
    DecW,
    IncHl,
    DecHl,
    Jump(Cnd),
    JumpR(Cnd),
    JumpHL,
    Call(Cnd),
    Return(Cnd),
    Reset(u8),
    Rl,
    RlA,
    Rlc,
    RlcA,
    Rr,
    RrA,
    Rrc,
    RrcA,
    Sla,
    Sra,
    Srl,
    Swap,
    Bit(u8),
    Res(u8),
    Set(u8),
    Halt,
    Scf,
    Ccf,
    Di,
    Ei,
    Wait,
}

impl Cpu<'_> {
    pub(super) fn execute(&mut self, op: Op) -> Result<()> {
        use Op::*;
        match op {
            FetchDecode => self.fetch_decode()?,
            Read(src) => self.exec_read(src),
            Write(dst) => self.exec_write(dst),
            ReadW(src) => self.exec_read_wide(src),
            WriteW(dst) => self.exec_write_wide(dst),
            LdSPOff => self.exec_ld_sp_off(),
            Push => self.exec_push(),
            Pop => self.exec_pop(),
            PushW => self.push_ops(vec![Push, Push]),
            PopW => self.push_ops(vec![Pop, Pop]),
            Add => self.exec_add(),
            AddHL(reg) => self.exec_add_hl(reg),
            AddSP => self.exec_add_sp(),
            Adc => self.exec_adc(),
            Sub => self.exec_sub(),
            Sbc => self.exec_sbc(),
            And => self.exec_and(),
            Or => self.exec_or(),
            Xor => self.exec_xor(),
            Cp => self.exec_cp(),
            Daa => self.exec_daa(),
            Cpl => self.exec_cpl(),
            Inc => self.exec_inc(),
            Dec => self.exec_dec(),
            IncW => self.exec_inc_wide(),
            DecW => self.exec_dec_wide(),
            IncHl => self.s.cpu.registers.inc_hl(),
            DecHl => self.s.cpu.registers.dec_hl(),
            Jump(cnd) => self.exec_jump(cnd),
            JumpR(cnd) => self.exec_jump_relative(cnd),
            JumpHL => self.exec_jump_hl(),
            Call(cnd) => self.exec_call(cnd),
            Return(cnd) => self.exec_return(cnd),
            Reset(vec) => self.exec_reset(vec),
            Rl => self.exec_rl(),
            RlA => self.exec_rl_a(),
            Rlc => self.exec_rlc(),
            RlcA => self.exec_rlc_a(),
            Rr => self.exec_rr(),
            RrA => self.exec_rr_a(),
            Rrc => self.exec_rrc(),
            RrcA => self.exec_rrc_a(),
            Sla => self.exec_sla(),
            Sra => self.exec_sra(),
            Srl => self.exec_srl(),
            Swap => self.exec_swap(),
            Bit(bit) => self.exec_bit(bit),
            Res(bit) => self.exec_res(bit),
            Set(bit) => self.exec_set(bit),
            Halt => self.s.cpu.halt = true,
            Scf => self.exec_scf(),
            Ccf => self.exec_ccf(),
            Di => self.s.cpu.ime = false,
            Ei => self.s.cpu.ime = true,
            Wait => self.must_yield(),
        }

        Ok(())
    }

    fn exec_read(&mut self, src: Src) {
        use Src::*;
        let value = match src {
            Reg(reg) => self.s.cpu.registers.read(reg),
            Mem(ref_) => {
                let addr = self.resolve_ref(ref_);
                self.read_memory(addr)
            }
            Imm(imm) => imm,
        };

        self.stash(value);
    }

    fn exec_write(&mut self, dst: Dst) {
        let value = self.unstash();

        use Dst::*;
        match dst {
            Reg(reg) => self.s.cpu.registers.write(reg, value),
            Mem(ref_) => {
                let addr = self.resolve_ref(ref_);
                self.write_memory(addr, value);
            }
        }
    }

    fn exec_read_wide(&mut self, src: SrcW) {
        match src {
            SrcW::Reg(reg) => {
                let value = self.s.cpu.registers.read_wide(reg);
                self.stash_wide(value);
            }
            SrcW::Mem(ref_) => {
                let addr = self.resolve_ref(ref_);
                let value = self.read_memory(addr);
                self.stash(value);

                let src2 = Src::Mem(Ref::Imm(addr + 1));
                self.push_op(Op::Read(src2));
            }
            SrcW::Imm(imm) => self.stash_wide(imm),
        }
    }

    fn exec_write_wide(&mut self, dst: DstW) {
        match dst {
            DstW::Reg(reg) => {
                let value = self.unstash_wide();
                self.s.cpu.registers.write_wide(reg, value);
            }
            DstW::Mem(ref_) => {
                let value = self.unstash();
                let addr = self.resolve_ref(ref_);
                self.write_memory(addr + 1, value);

                let dst2 = Dst::Mem(Ref::Imm(addr));
                self.push_op(Op::Write(dst2));
            }
        }
    }

    fn exec_ld_sp_off(&mut self) {
        let sp = self.s.cpu.registers.sp;
        let offset = i16::from(self.unstash() as i8);
        let value = sp.wrapping_add_signed(offset);

        self.s.cpu.registers.set_hl(value);
        self.s.cpu.registers.flags = Flags {
            z: false,
            n: false,
            h: (sp & 0x0f) + (offset as u16 & 0x0f) > 0x0f,
            c: (sp & 0xff) + (offset as u16 & 0xff) > 0xff,
        };
    }

    fn exec_push(&mut self) {
        let value = self.unstash();
        let addr = self.s.cpu.registers.sp.wrapping_sub(1);
        self.write_memory(addr, value);
        self.s.cpu.registers.sp = addr;
    }

    fn exec_pop(&mut self) {
        let addr = self.s.cpu.registers.sp;
        let value = self.read_memory(addr);
        self.s.cpu.registers.sp = addr.wrapping_add(1);
        self.stash(value);
    }

    fn exec_add(&mut self) {
        let a = self.s.cpu.registers.a;
        let b = self.unstash();
        let result = a.wrapping_add(b);

        self.s.cpu.registers.a = result;
        self.s.cpu.registers.flags = Flags {
            z: result == 0,
            n: false,
            h: (a & 0x0f) + (b & 0x0f) > 0x0f,
            c: u16::from(a) + u16::from(b) > 0xff,
        };
    }

    fn exec_add_hl(&mut self, reg: RegW) {
        let a = self.s.cpu.registers.hl();
        let b = self.s.cpu.registers.read_wide(reg);
        let result = a.wrapping_add(b);

        self.s.cpu.registers.set_hl(result);
        let flags = &mut self.s.cpu.registers.flags;
        flags.n = false;
        flags.h = (a & 0x0fff) + (b & 0x0fff) > 0x0fff;
        flags.c = u32::from(a) + u32::from(b) > 0xffff;
    }

    fn exec_add_sp(&mut self) {
        let a = self.s.cpu.registers.sp;
        let b = i16::from(self.unstash() as i8);
        let result = a.wrapping_add_signed(b);

        self.s.cpu.registers.sp = result;
        self.s.cpu.registers.flags = Flags {
            z: false,
            n: false,
            h: (a & 0x0f) + (b as u16 & 0x0f) > 0x0f,
            c: (a & 0xff) + (b as u16 & 0xff) > 0xff,
        };
    }

    fn exec_adc(&mut self) {
        let a = self.s.cpu.registers.a;
        let b = self.unstash();
        let c = u8::from(self.s.cpu.registers.flags.c);
        let result = a.wrapping_add(b).wrapping_add(c);

        self.s.cpu.registers.a = result;
        self.s.cpu.registers.flags = Flags {
            z: result == 0,
            n: false,
            h: (a & 0x0f) + (b & 0x0f) + c > 0x0f,
            c: u16::from(a) + u16::from(b) + u16::from(c) > 0xff,
        };
    }

    fn exec_sub(&mut self) {
        let a = self.s.cpu.registers.a;
        let b = self.unstash();
        let result = a.wrapping_sub(b);

        self.s.cpu.registers.a = result;
        self.s.cpu.registers.flags = Flags {
            z: result == 0,
            n: true,
            h: (a & 0x0f) < (b & 0x0f),
            c: a < b,
        };
    }

    fn exec_sbc(&mut self) {
        let a = self.s.cpu.registers.a;
        let b = self.unstash();
        let c = u8::from(self.s.cpu.registers.flags.c);
        let result = a.wrapping_sub(b).wrapping_sub(c);

        self.s.cpu.registers.a = result;
        self.s.cpu.registers.flags = Flags {
            z: result == 0,
            n: true,
            h: (a & 0x0f) < (b & 0x0f) + c,
            c: u16::from(a) < u16::from(b) + u16::from(c),
        };
    }

    fn exec_and(&mut self) {
        let a = self.s.cpu.registers.a;
        let b = self.unstash();
        let result = a & b;

        self.s.cpu.registers.a = result;
        self.s.cpu.registers.flags = Flags {
            z: result == 0,
            h: true,
            ..Default::default()
        };
    }

    fn exec_or(&mut self) {
        let a = self.s.cpu.registers.a;
        let b = self.unstash();
        let result = a | b;

        self.s.cpu.registers.a = result;
        self.s.cpu.registers.flags = Flags {
            z: result == 0,
            ..Default::default()
        };
    }

    fn exec_xor(&mut self) {
        let a = self.s.cpu.registers.a;
        let b = self.unstash();
        let result = a ^ b;

        self.s.cpu.registers.a = result;
        self.s.cpu.registers.flags = Flags {
            z: result == 0,
            ..Default::default()
        };
    }

    fn exec_cp(&mut self) {
        let a = self.s.cpu.registers.a;
        let b = self.unstash();
        let result = a.wrapping_sub(b);

        self.s.cpu.registers.flags = Flags {
            z: result == 0,
            n: true,
            h: (a & 0x0f) < (b & 0x0f),
            c: a < b,
        };
    }

    fn exec_daa(&mut self) {
        let mut x = i16::from(self.s.cpu.registers.a);
        let flags = &self.s.cpu.registers.flags;

        if flags.n {
            if flags.h {
                x -= 0x06;
                x &= 0xff;
            }
            if flags.c {
                x -= 0x60;
            }
        } else {
            if flags.h || x & 0x0f > 0x09 {
                x += 0x06;
            }
            if flags.c || x > 0x9f {
                x += 0x60;
            }
        }

        let result = x as u8;

        self.s.cpu.registers.a = result;
        self.s.cpu.registers.flags.z = result == 0;
        self.s.cpu.registers.flags.h = false;
        if x.bit(8) {
            self.s.cpu.registers.flags.c = true;
        }
    }

    fn exec_cpl(&mut self) {
        self.s.cpu.registers.a ^= 0xff;
        self.s.cpu.registers.flags.n = true;
        self.s.cpu.registers.flags.h = true;
    }

    fn exec_inc(&mut self) {
        let value = self.unstash();
        let result = value.wrapping_add(1);

        let flags = &mut self.s.cpu.registers.flags;
        flags.z = result == 0;
        flags.n = false;
        flags.h = result & 0x0f == 0;

        self.stash(result);
    }

    fn exec_dec(&mut self) {
        let value = self.unstash();
        let result = value.wrapping_sub(1);

        let flags = &mut self.s.cpu.registers.flags;
        flags.z = result == 0;
        flags.n = true;
        flags.h = result & 0x0f == 0x0f;

        self.stash(result);
    }

    fn exec_inc_wide(&mut self) {
        let value = self.unstash_wide();
        let result = value.wrapping_add(1);

        self.stash_wide(result);
    }

    fn exec_dec_wide(&mut self) {
        let value = self.unstash_wide();
        let result = value.wrapping_sub(1);

        self.stash_wide(result);
    }

    fn exec_jump(&mut self, cnd: Cnd) {
        let addr = self.unstash_wide();
        if self.test(cnd) {
            self.s.cpu.pc = addr;
            self.must_yield();
        }
    }

    fn exec_jump_relative(&mut self, cnd: Cnd) {
        let offset = i16::from(self.unstash() as i8);
        if self.test(cnd) {
            self.s.cpu.pc = self.s.cpu.pc.wrapping_add_signed(offset);
            self.must_yield();
        }
    }

    fn exec_jump_hl(&mut self) {
        self.s.cpu.pc = self.s.cpu.registers.hl();
    }

    fn exec_call(&mut self, cnd: Cnd) {
        let addr = self.unstash_wide();
        if self.test(cnd) {
            let pc = self.s.cpu.pc;
            self.s.cpu.pc = addr;
            self.must_yield();

            self.stash_wide(pc);
            self.push_op(Op::PushW);
        }
    }

    fn exec_return(&mut self, cnd: Cnd) {
        if self.test(cnd) {
            if cnd != Cnd::None {
                self.must_yield();
            }
            self.push_ops(vec![Op::PopW, Op::Jump(Cnd::None)]);
        } else {
            self.must_yield();
        }
    }

    fn exec_reset(&mut self, vec: u8) {
        let pc = self.s.cpu.pc;
        self.s.cpu.pc = u16::from(vec);

        self.stash_wide(pc);
        self.must_yield();
    }

    fn exec_rl(&mut self) {
        let x = self.unstash();
        let c = u8::from(self.s.cpu.registers.flags.c);
        let result = (x << 1) | c;

        self.s.cpu.registers.flags = Flags {
            z: result == 0,
            c: x.bit(7),
            ..Default::default()
        };

        self.stash(result);
    }

    fn exec_rl_a(&mut self) {
        let a = self.s.cpu.registers.a;
        let c = u8::from(self.s.cpu.registers.flags.c);
        let result = (a << 1) | c;

        self.s.cpu.registers.a = result;
        self.s.cpu.registers.flags = Flags {
            c: a.bit(7),
            ..Default::default()
        };
    }

    fn exec_rlc(&mut self) {
        let x = self.unstash();
        let result = (x << 1) | (x >> 7);

        self.s.cpu.registers.flags = Flags {
            z: result == 0,
            c: x.bit(7),
            ..Default::default()
        };

        self.stash(result);
    }

    fn exec_rlc_a(&mut self) {
        let a = self.s.cpu.registers.a;
        let result = (a << 1) | (a >> 7);

        self.s.cpu.registers.a = result;
        self.s.cpu.registers.flags = Flags {
            c: a.bit(7),
            ..Default::default()
        };
    }

    fn exec_rr(&mut self) {
        let x = self.unstash();
        let c = u8::from(self.s.cpu.registers.flags.c);
        let result = (x >> 1) | (c << 7);

        self.s.cpu.registers.flags = Flags {
            z: result == 0,
            c: x.bit(0),
            ..Default::default()
        };

        self.stash(result);
    }

    fn exec_rr_a(&mut self) {
        let a = self.s.cpu.registers.a;
        let c = u8::from(self.s.cpu.registers.flags.c);
        let result = (a >> 1) | (c << 7);

        self.s.cpu.registers.a = result;
        self.s.cpu.registers.flags = Flags {
            c: a.bit(0),
            ..Default::default()
        };
    }

    fn exec_rrc(&mut self) {
        let x = self.unstash();
        let result = (x >> 1) | (x << 7);

        self.s.cpu.registers.flags = Flags {
            z: result == 0,
            c: x.bit(0),
            ..Default::default()
        };

        self.stash(result);
    }

    fn exec_rrc_a(&mut self) {
        let a = self.s.cpu.registers.a;
        let result = (a >> 1) | (a << 7);

        self.s.cpu.registers.a = result;
        self.s.cpu.registers.flags = Flags {
            c: a.bit(0),
            ..Default::default()
        };
    }

    fn exec_sla(&mut self) {
        let x = self.unstash();
        let result = x << 1;

        self.s.cpu.registers.flags = Flags {
            z: result == 0,
            c: x.bit(7),
            ..Default::default()
        };

        self.stash(result);
    }

    fn exec_sra(&mut self) {
        let x = self.unstash();
        let result = (x >> 1) | (x & 0x80);

        self.s.cpu.registers.flags = Flags {
            z: result == 0,
            c: x.bit(0),
            ..Default::default()
        };

        self.stash(result);
    }

    fn exec_srl(&mut self) {
        let x = self.unstash();
        let result = x >> 1;

        self.s.cpu.registers.flags = Flags {
            z: result == 0,
            c: x.bit(0),
            ..Default::default()
        };

        self.stash(result);
    }

    fn exec_swap(&mut self) {
        let x = self.unstash();
        let result = (x << 4) | (x >> 4);

        self.s.cpu.registers.flags = Flags {
            z: result == 0,
            ..Default::default()
        };

        self.stash(result);
    }

    fn exec_bit(&mut self, bit: u8) {
        let x = self.unstash();
        let zero = !x.bit(bit);

        self.s.cpu.registers.flags.z = zero;
        self.s.cpu.registers.flags.n = false;
        self.s.cpu.registers.flags.h = true;
    }

    fn exec_res(&mut self, bit: u8) {
        let mut x = self.unstash();
        x.reset_bit(bit);
        self.stash(x);
    }

    fn exec_set(&mut self, bit: u8) {
        let mut x = self.unstash();
        x.set_bit(bit);
        self.stash(x);
    }

    fn exec_scf(&mut self) {
        self.s.cpu.registers.flags.n = false;
        self.s.cpu.registers.flags.h = false;
        self.s.cpu.registers.flags.c = true;
    }

    fn exec_ccf(&mut self) {
        self.s.cpu.registers.flags.n = false;
        self.s.cpu.registers.flags.h = false;
        self.s.cpu.registers.flags.c = !self.s.cpu.registers.flags.c;
    }

    fn stash(&mut self, value: u8) {
        self.s.cpu.stash.push(value);
    }

    fn stash_wide(&mut self, value: u16) {
        let [a, b] = value.to_le_bytes();
        self.stash(a);
        self.stash(b);
    }

    fn unstash(&mut self) -> u8 {
        self.s.cpu.stash.pop().unwrap()
    }

    fn unstash_wide(&mut self) -> u16 {
        let b = self.unstash();
        let a = self.unstash();
        u16::from_le_bytes([a, b])
    }

    fn resolve_ref(&self, ref_: Ref) -> u16 {
        use Ref::*;
        match ref_ {
            Reg(reg) => self.s.cpu.registers.read_wide(reg),
            Imm(imm) => imm,
            RegH(reg) => {
                let offset = self.s.cpu.registers.read(reg);
                u16::from(offset) + 0xff00
            }
            ImmH(offset) => u16::from(offset) + 0xff00,
        }
    }

    fn test(&mut self, cnd: Cnd) -> bool {
        let flags = &self.s.cpu.registers.flags;
        match cnd {
            Cnd::None => true,
            Cnd::Z => flags.z,
            Cnd::NZ => !flags.z,
            Cnd::C => flags.c,
            Cnd::NC => !flags.c,
        }
    }
}
