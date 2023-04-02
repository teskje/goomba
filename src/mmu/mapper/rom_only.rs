use anyhow::{bail, Result};

use log::warn;

use crate::mmu::bank::Bank;
use crate::mmu::KB;

const ROM_SIZE: usize = 32 * KB;

#[derive(Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub(in crate::mmu) struct Mapper {
    rom: Bank<ROM_SIZE>,
}

impl Mapper {
    pub(super) fn read_rom(&self, addr: u16) -> u8 {
        self.rom.get(addr).unwrap_or_else(|| {
            warn!("invalid ROM read addr: {addr:#x}");
            0xff
        })
    }
}

pub(super) fn load(rom: Vec<u8>) -> Result<Mapper> {
    match rom.try_into() {
        Ok(rom) => Ok(Mapper { rom }),
        Err(v) => bail!("invalid ROM size: {:#x}", v.len()),
    }
}
