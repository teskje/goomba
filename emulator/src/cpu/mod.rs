use anyhow::{bail, Result};

use crate::mmu::Mmu;
use crate::state::State;

use self::execute::Op;

mod decode;
mod execute;
mod interrupt;
mod state;

pub(crate) use self::interrupt::Interrupt;
pub(crate) use self::state::{CpuState, InterruptState};

pub(crate) struct Cpu<'a> {
    s: &'a mut State,
    must_yield: bool,
}

impl<'a> Cpu<'a> {
    pub fn new(state: &'a mut State) -> Self {
        Self {
            s: state,
            must_yield: false,
        }
    }

    pub fn step(&mut self) -> Result<()> {
        while let Some(op) = self.next_op() {
            self.execute(op)?;
        }
        if self.must_yield {
            return Ok(());
        };

        self.handle_interrupt();
        if self.must_yield || self.s.cpu.halt {
            Ok(())
        } else {
            self.fetch_decode()
        }
    }

    fn must_yield(&mut self) {
        debug_assert!(!self.must_yield, "double yield");
        self.must_yield = true;
    }

    fn next_op(&mut self) -> Option<Op> {
        if self.must_yield {
            None
        } else {
            self.s.cpu.todo.pop()
        }
    }

    fn push_op(&mut self, op: Op) {
        self.s.cpu.todo.push(op);
    }

    fn push_ops<I>(&mut self, ops: I)
    where
        I: IntoIterator<Item = Op>,
        I::IntoIter: DoubleEndedIterator,
    {
        self.s.cpu.todo.extend(ops.into_iter().rev());
    }

    fn fetch_decode(&mut self) -> Result<()> {
        let byte = self.read_program();
        let ops = &mut self.s.cpu.stash;
        ops.push(byte);

        match code::decode(ops) {
            Ok(inst) => {
                self.s.cpu.stash.clear();
                self.decode(inst);
            }
            Err(code::Error::TooFewBytes) => {
                self.push_op(Op::FetchDecode);
            }
            Err(error) => bail!("instruction decode error: {error}"),
        }
        Ok(())
    }

    fn read_program(&mut self) -> u8 {
        let pc = self.s.cpu.pc;
        let value = self.read_memory(pc);
        self.s.cpu.pc += 1;
        value
    }

    fn read_memory(&mut self, addr: u16) -> u8 {
        self.must_yield();
        Mmu::new(self.s).read(addr)
    }

    fn write_memory(&mut self, addr: u16, value: u8) {
        self.must_yield();
        Mmu::new(self.s).write(addr, value)
    }
}
