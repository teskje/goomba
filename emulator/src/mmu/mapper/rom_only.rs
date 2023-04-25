use anyhow::{bail, Result};

use log::warn;

use crate::mmu::memory::Memory;
use crate::mmu::KB;

const ROM_SIZE: usize = 32 * KB;

#[derive(Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub(in crate::mmu) struct Mapper {
    rom: Memory,
}

impl Mapper {
    pub(super) fn read_rom(&self, addr: u16) -> u8 {
        self.rom.get(addr).unwrap_or_else(|| {
            warn!("invalid ROM read addr: {addr:#x}");
            0xff
        })
    }
}

pub(super) fn load(rom: Memory) -> Result<Mapper> {
    if rom.len() != ROM_SIZE {
        bail!("invalid ROM size: {:#x}", rom.len());
    }
    Ok(Mapper { rom })
}
