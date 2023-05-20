use std::path::Path;

use anyhow::Result;
use log::{error, info};

use crate::cpu::Cpu;
use crate::dma::Dma;
use crate::joypad::Joypad;
use crate::ppu::Ppu;
use crate::state::State;
use crate::timer::Timer;

mod bits;
mod cartridge;
mod cpu;
mod dma;
mod frame;
mod joypad;
mod mmu;
mod ppu;
mod state;
mod timer;

pub use frame::Frame;
pub use joypad::Button;

pub struct Emulator {
    state: State,
}

impl Emulator {
    pub fn load(rom_or_save: Vec<u8>, ram: Option<Vec<u8>>) -> Result<Self> {
        let state = State::load(rom_or_save, ram)?;

        Ok(Self { state })
    }

    pub fn render_frame(&mut self) -> Result<Frame> {
        let s = &mut self.state;
        loop {
            Timer::new(s).step();
            Cpu::new(s).step()?;
            Dma::new(s).step();

            let mut ppu = Ppu::new(s);
            (0..4).for_each(|_| ppu.step());

            if let Some(frame) = ppu.take_frame() {
                return Ok(frame);
            }
        }
    }

    pub fn press_button(&mut self, button: Button) {
        Joypad::new(&mut self.state).press_button(button);
    }

    pub fn release_button(&mut self, button: Button) {
        Joypad::new(&mut self.state).release_button(button);
    }

    pub fn save_state(&self, path: &Path) {
        match self.state.store_save(path) {
            Ok(()) => info!("saved state to {path:?}"),
            Err(e) => error!("cannot save state: {e:#}"),
        }
    }

    pub fn save_ram(&self, path: &Path) {
        match self.state.store_ram(path) {
            Ok(()) => info!("saved RAM to {path:?}"),
            Err(e) => error!("cannot save RAM: {e:#}"),
        }
    }

    pub fn dump_ram(&self) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        self.state.dump_ram(&mut buf)?;
        Ok(buf)
    }
}
