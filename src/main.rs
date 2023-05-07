use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};

use emulator::Emulator;

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
    env_logger::init();

    let args: Args = argh::from_env();

    let rom_or_save = fs::read(&args.path).with_context(|| format!("opening {:?}", args.path))?;

    let ram = match &args.ram_path {
        Some(p) if p.exists() => {
            let ram = fs::read(p).with_context(|| format!("opening {p:?}"))?;
            Some(ram)
        }
        _ => None,
    };

    let emu = Emulator::load(rom_or_save, ram)?;

    gui::run(emu)
}
