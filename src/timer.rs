use log::trace;

use crate::bits::BitsExt;
use crate::cpu::{Interrupt, InterruptState};
use crate::state::State;

pub struct Timer<'a> {
    t: &'a mut TimerState,
    interrupts: &'a mut InterruptState,
}

impl<'a> Timer<'a> {
    pub fn new(state: &'a mut State) -> Self {
        Self {
            t: &mut state.timer,
            interrupts: &mut state.cpu.interrupts,
        }
    }

    pub fn step(&mut self) {
        let counter = self.t.counter.wrapping_add(4);
        self.t.counter = counter;

        let tac = self.t.tac;
        if tac.enable() {
            let should_tick = match tac.clock_select() {
                0b00 => counter.bits(0..=9) == 0,
                0b01 => counter.bits(0..=3) == 0,
                0b10 => counter.bits(0..=5) == 0,
                0b11 => counter.bits(0..=7) == 0,
                _ => unreachable!(),
            };

            if should_tick {
                self.tick_timer();
            }
        }
    }

    fn tick_timer(&mut self) {
        let tima = self.t.tima.wrapping_add(1);
        self.t.tima = tima;
        trace!("tick: {tima:#x}");

        if tima == 0 {
            self.interrupts.set_flag(Interrupt::Timer);
            self.t.tima = self.t.tma;
        }
    }
}

#[derive(Debug, Default)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct TimerState {
    counter: u16,
    tima: u8,
    tma: u8,
    tac: Tac,
}

impl TimerState {
    pub fn read_div(&self) -> u8 {
        (self.counter >> 8) as u8
    }

    pub fn write_div(&mut self, _value: u8) {
        self.counter = 0;
    }

    pub fn read_tima(&self) -> u8 {
        self.tima
    }

    pub fn write_tima(&mut self, value: u8) {
        self.tima = value;
    }

    pub fn read_tma(&self) -> u8 {
        self.tma
    }

    pub fn write_tma(&mut self, value: u8) {
        self.tma = value;
    }

    pub fn read_tac(&self) -> u8 {
        self.tac.0
    }

    pub fn write_tac(&mut self, value: u8) {
        self.tac = Tac(value.bits(0..=2));
    }
}

#[derive(Clone, Copy, Debug, Default)]
#[derive(serde::Serialize, serde::Deserialize)]
struct Tac(u8);

impl Tac {
    fn enable(&self) -> bool {
        self.0.bit(2)
    }

    fn clock_select(&self) -> u8 {
        self.0.bits(0..=1)
    }
}
