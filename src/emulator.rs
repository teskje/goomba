use std::path::PathBuf;

use anyhow::Result;
use log::error;

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
    save_path: PathBuf,
}

impl Emulator {
    pub fn load(config: Config) -> Result<Self> {
        let mut state = State::load_from(&config.path)?;
        if config.print_serial {
            state.mmu.enable_serial_printing();
        }

        let save_path = config.path.with_extension("gmba");

        Ok(Self { state, save_path })
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

    pub fn save_game(&self) {
        if let Err(error) = self.state.store_save(&self.save_path) {
            error!("cannot save game: {error}");
        }
    }
}
