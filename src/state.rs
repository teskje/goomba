use std::fs::{self, File};
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
    pub fn load_from(base_path: &Path, ram_path: Option<&Path>) -> Result<Self> {
        let mut base_file =
            File::open(base_path).with_context(|| format!("opening {base_path:?}"))?;

        let ram_file = match ram_path {
            Some(p) if p.exists() => {
                let file = File::open(p).with_context(|| format!("opening {p:?}"))?;
                Some(file)
            }
            _ => None,
        };

        let mut tag = [0; SAVESTATE_TAG.len()];
        base_file.read_exact(&mut tag).ok();

        if tag == SAVESTATE_TAG {
            Self::load_save(base_file)
                .with_context(|| format!("loading savestate from {base_path:?}"))
        } else {
            base_file.rewind()?;
            Self::load_cartridge(base_file, ram_file).with_context(|| {
                format!("loading cartridge from rom={base_path:?}, ram={ram_path:?}")
            })
        }
    }

    fn load_cartridge(rom_file: File, ram_file: Option<File>) -> Result<Self> {
        let rom = read_file(rom_file)?;
        let ram = match ram_file {
            Some(f) => Some(read_file(f)?),
            None => None,
        };

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

    pub fn store_ram(&self, path: &Path) -> Result<()> {
        let tmp_path = path.with_extension("ram.tmp");
        let mut file = File::create(&tmp_path).with_context(|| format!("creating {path:?}"))?;
        self.mmu
            .store_ram(&mut file)
            .with_context(|| format!("writing RAM to {path:?}"))?;

        fs::rename(&tmp_path, path).with_context(|| format!("renaming {tmp_path:?} to {path:?}"))
    }
}

fn read_file(mut file: File) -> Result<Vec<u8>> {
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    Ok(buf)
}
