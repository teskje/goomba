use std::path::PathBuf;

use anyhow::Result;

use emulator::{Config, Emulator};

mod gui;

/// An emulator for the classic GameBoy.
#[derive(argh::FromArgs)]
struct Args {
    /// path of the cartridge or savegame file to execute
    #[argh(positional)]
    path: PathBuf,
    /// path of persistent cartridge RAM
    #[argh(option)]
    ram_path: Option<PathBuf>,
}

fn main() -> Result<()> {
    let args: Args = argh::from_env();

    env_logger::init();

    let emu = Emulator::load(Config {
        path: args.path,
        ram_path: args.ram_path,
    })?;

    gui::run(emu)
}
