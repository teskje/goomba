use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use anyhow::{Context, Result};

use crate::cpu::CpuState;
use crate::dma::DmaState;
use crate::joypad::JoypadState;
use crate::mmu::{self, MmuState};
use crate::ppu::PpuState;
use crate::timer::TimerState;

const SAVESTATE_TAG: &[u8] = b"goomba:savestate\n";

#[derive(Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct State {
    pub mmu: MmuState,
    pub timer: TimerState,
    pub joypad: JoypadState,
    pub cpu: CpuState,
    pub ppu: PpuState,
    pub dma: DmaState,
}

impl State {
    pub fn load(rom_or_save: Vec<u8>, ram: Option<Vec<u8>>) -> Result<Self> {
        if rom_or_save.starts_with(SAVESTATE_TAG) {
            Self::load_save(&rom_or_save).context("loading savestate")
        } else {
            Self::load_cartridge(rom_or_save, ram).context("loading cartridge")
        }
    }

    fn load_cartridge(rom: Vec<u8>, ram: Option<Vec<u8>>) -> Result<Self> {
        let mmu_state = mmu::load_cartridge(rom, ram)?;

        Ok(Self {
            mmu: mmu_state,
            timer: Default::default(),
            joypad: Default::default(),
            cpu: Default::default(),
            ppu: Default::default(),
            dma: Default::default(),
        })
    }

    fn load_save(save: &[u8]) -> Result<Self> {
        let save = &save[SAVESTATE_TAG.len()..];
        rmp_serde::decode::from_slice(save).map_err(Into::into)
    }

    pub fn store_save(&self, path: &Path) -> Result<()> {
        let mut file = File::create(path).with_context(|| format!("creating {path:?}"))?;

        file.write_all(SAVESTATE_TAG)
            .with_context(|| format!("writing tag to {path:?}"))?;

        rmp_serde::encode::write_named(&mut file, self)
            .with_context(|| format!("writing state to {path:?}"))
    }

    pub fn store_ram(&self, path: &Path) -> Result<()> {
        let tmp_path = path.with_extension("ram.tmp");
        let mut file = File::create(&tmp_path).with_context(|| format!("creating {path:?}"))?;
        self.dump_ram(&mut file)
            .with_context(|| format!("writing RAM to {path:?}"))?;

        fs::rename(&tmp_path, path).with_context(|| format!("renaming {tmp_path:?} to {path:?}"))
    }

    pub fn dump_ram<W: Write>(&self, w: W) -> Result<()> {
        self.mmu.dump_ram(w)
    }
}
