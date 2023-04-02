use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;

use anyhow::Result;

mod bits;
mod cartridge;
mod cpu;
mod dma;
mod emulator;
mod frame;
mod gui;
mod input;
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

    let (input_tx, input_rx) = mpsc::channel();
    let (frame_tx, frame_rx) = mpsc::sync_channel(1);

    thread::Builder::new()
        .name("emulator".into())
        .spawn(|| emulator::run(config, input_rx, frame_tx).map_err(|e| panic!("{e}")))?;

    gui::run(frame_rx, input_tx)
}
