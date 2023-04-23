use std::path::PathBuf;

use anyhow::Result;
use log::{error, info};

use crate::cpu::Cpu;
use crate::dma::Dma;
use crate::frame::Frame;
use crate::joypad::{Button, Joypad};
use crate::ppu::Ppu;
use crate::state::State;
use crate::timer::Timer;
use crate::Config;

pub struct Emulator {
    state: State,
    savestate_path: PathBuf,
    ram_path: Option<PathBuf>,
}

impl Emulator {
    pub fn load(config: Config) -> Result<Self> {
        let base_path = config.path;
        let ram_path = config.ram_path;
        let savestate_path = base_path.with_extension("gmba");

        let mut state = State::load_from(&base_path, ram_path.as_deref())?;
        if config.print_serial {
            state.mmu.enable_serial_printing();
        }

        Ok(Self {
            state,
            savestate_path,
            ram_path,
        })
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

    pub fn save_state(&self) {
        let path = &self.savestate_path;
        match self.state.store_save(path) {
            Ok(()) => info!("saved state to {path:?}"),
            Err(e) => error!("cannot save state: {e:#}"),
        }
    }

    pub fn save_ram(&self) {
        let Some(path) = &self.ram_path else { return };
        match self.state.store_ram(path) {
            Ok(()) => info!("saved RAM to {path:?}"),
            Err(e) => error!("cannot save RAM: {e:#}"),
        }
    }
}
