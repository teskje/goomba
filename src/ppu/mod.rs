use std::mem;

use crate::cpu::Interrupt;
use crate::frame::{self, Frame};
use crate::state::State;

mod fetcher;
mod fifo;
mod object;
mod state;

use self::fetcher::{Fetch, Fetcher};
pub use self::state::PpuState;

const FRAME_LINES: u32 = 154;
const LINE_DOTS: u16 = 456;

pub struct Ppu<'a> {
    s: &'a mut State,
    ready_frame: Option<Frame>,
}

impl<'a> Ppu<'a> {
    pub fn new(state: &'a mut State) -> Self {
        Self {
            s: state,
            ready_frame: None,
        }
    }

    pub fn step(&mut self) {
        match self.s.ppu.mode {
            Mode::OamScan => self.step_oam_scan(),
            Mode::Draw => self.step_draw(),
            Mode::HBlank | Mode::VBlank | Mode::PpuOff => (),
        }

        self.update_stat();
        self.end_dot();
    }

    fn end_dot(&mut self) {
        self.s.ppu.line_dots += 1;

        if self.s.ppu.line_dots == LINE_DOTS {
            self.end_line();
        }
    }

    fn end_line(&mut self) {
        self.s.ppu.line_dots = 0;
        self.s.ppu.registers.ly += 1;
        if self.s.ppu.inside_window {
            self.s.ppu.window_y += 1;
            self.s.ppu.inside_window = false;
        }

        let y = u32::from(self.s.ppu.registers.ly);
        let mode = self.s.ppu.mode;
        if y < frame::HEIGHT && mode == Mode::HBlank {
            self.enter_oam_scan();
        } else if y == frame::HEIGHT && mode == Mode::HBlank {
            self.enter_vblank();
        } else if y == FRAME_LINES {
            self.end_frame();
        }
    }

    fn end_frame(&mut self) {
        let frame = mem::take(&mut self.s.ppu.frame);
        self.ready_frame = Some(frame);

        self.s.ppu.registers.ly = 0;
        self.s.ppu.window_y = 0;
        if self.s.ppu.registers.lcdc.ppu_enable() {
            self.enter_oam_scan();
        } else {
            self.set_mode(Mode::PpuOff);
        };
    }

    pub fn take_frame(&mut self) -> Option<Frame> {
        self.ready_frame.take()
    }

    fn set_mode(&mut self, mode: Mode) {
        self.s.ppu.mode = mode;

        let stat = &mut self.s.ppu.registers.stat;
        *stat &= 0xfc;
        *stat |= mode.stat();
    }

    fn enter_oam_scan(&mut self) {
        self.set_mode(Mode::OamScan);
        self.s.ppu.object_slots = Default::default();
        self.s.ppu.oam_scan = Default::default();
    }

    fn enter_draw(&mut self) {
        self.set_mode(Mode::Draw);
        self.s.ppu.draw_x = 0;
        self.s.ppu.fetch = Fetch::new(self.s.ppu.registers.scx);
    }

    fn enter_vblank(&mut self) {
        self.set_mode(Mode::VBlank);
        self.s.cpu.interrupts.set_flag(Interrupt::VBlank);
    }

    fn step_oam_scan(&mut self) {
        object::Scanner::new(self.s).step();

        if self.s.ppu.line_dots == 79 {
            self.enter_draw();
        }
    }

    fn step_draw(&mut self) {
        self.maybe_enable_window();

        if let Some(px) = Fetcher::new(self.s).step() {
            self.s.ppu.frame.push_pixel(px);
            self.s.ppu.draw_x += 1;
        }

        if u32::from(self.s.ppu.draw_x) == frame::WIDTH {
            self.set_mode(Mode::HBlank);
        }
    }

    fn maybe_enable_window(&mut self) {
        if self.s.ppu.inside_window {
            return;
        }

        let r = &self.s.ppu.registers;
        let wy_triggered = r.wy <= r.ly;
        let wx_triggered = r.wx <= self.s.ppu.draw_x + 7;

        if r.lcdc.bg_enable() && r.lcdc.win_enable() && wy_triggered && wx_triggered {
            self.s.ppu.inside_window = true;
            self.s.ppu.fetch.enable_window();
        }
    }

    fn update_stat(&mut self) {
        let regs = &mut self.s.ppu.registers;
        regs.refresh_stat_lycly();

        let prev_high = self.s.ppu.stat_interrupt;
        let new_high = regs.mode_interrupt() || regs.lyc_interrupt();
        if !prev_high && new_high {
            self.s.cpu.interrupts.set_flag(Interrupt::LcdStat);
        }
        self.s.ppu.stat_interrupt = new_high;
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[derive(serde::Serialize, serde::Deserialize)]
enum Mode {
    #[default]
    PpuOff,
    OamScan,
    Draw,
    HBlank,
    VBlank,
}

impl Mode {
    fn stat(&self) -> u8 {
        match self {
            Mode::PpuOff | Mode::HBlank => 0,
            Mode::VBlank => 1,
            Mode::OamScan => 2,
            Mode::Draw => 3,
        }
    }
}
