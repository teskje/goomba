use std::fs::File;
use std::io::{Read, Seek, Write};
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
pub struct State {
    pub mmu: MmuState,
    pub timer: TimerState,
    pub joypad: JoypadState,
    pub cpu: CpuState,
    pub ppu: PpuState,
    pub dma: DmaState,
}

impl State {
    pub fn load_from(path: &Path) -> Result<Self> {
        let mut file = File::open(path).with_context(|| format!("opening {path:?}"))?;

        let mut tag = [0; SAVESTATE_TAG.len()];
        file.read_exact(&mut tag).ok();

        if tag == SAVESTATE_TAG {
            Self::load_save(file).with_context(|| format!("loading savestate from {path:?}"))
        } else {
            file.rewind()?;
            Self::load_cartridge(file).with_context(|| format!("loading cartridge from {path:?}"))
        }
    }

    fn load_cartridge(mut file: File) -> Result<Self> {
        let mut rom = Vec::new();
        file.read_to_end(&mut rom)?;

        let mmu_state = mmu::load_cartridge(rom)?;

        Ok(Self {
            mmu: mmu_state,
            timer: Default::default(),
            joypad: Default::default(),
            cpu: Default::default(),
            ppu: Default::default(),
            dma: Default::default(),
        })
    }

    fn load_save(file: File) -> Result<Self> {
        rmp_serde::decode::from_read(&file).map_err(Into::into)
    }

    pub fn store_save(&self, path: &Path) -> Result<()> {
        let mut file = File::create(path).with_context(|| format!("creating {path:?}"))?;

        file.write_all(SAVESTATE_TAG)
            .with_context(|| format!("writing tag to {path:?}"))?;

        rmp_serde::encode::write_named(&mut file, self)
            .with_context(|| format!("writing state to {path:?}"))
    }
}
