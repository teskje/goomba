use std::path::PathBuf;

use anyhow::Result;
use emulator::Emulator;

mod bits;
mod cartridge;
mod cpu;
mod dma;
mod emulator;
mod frame;
mod gui;
mod joypad;
mod mmu;
mod ppu;
mod state;
mod timer;

/// An emulator for the classic GameBoy.
#[derive(argh::FromArgs)]
pub struct Config {
    /// path of the cartridge or savegame file to execute
    #[argh(positional)]
    path: PathBuf,
    /// print serial output
    #[argh(switch)]
    print_serial: bool,
}

fn main() -> Result<()> {
    let config: Config = argh::from_env();

    env_logger::init();

    let emu = Emulator::load(config)?;
    gui::run(emu)
}
