use std::io::Write;

use anyhow::{Context, Result};

use log::warn;

use super::memory::Memory;

mod mbc1;
mod rom_only;

#[derive(Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub(super) enum Mapper {
    RomOnly(rom_only::Mapper),
    Mbc1(mbc1::Mapper),
}

impl Mapper {
    pub(super) fn read_rom(&self, addr: u16) -> u8 {
        match self {
            Self::RomOnly(m) => m.read_rom(addr),
            Self::Mbc1(m) => m.read_rom(addr),
        }
    }

    pub(super) fn write_rom(&mut self, addr: u16, value: u8) {
        match self {
            Self::RomOnly(_) => warn!("ROM write not supported"),
            Self::Mbc1(m) => m.write_rom(addr, value),
        }
    }

    pub(super) fn read_ram(&self, addr: u16) -> u8 {
        match self {
            Self::RomOnly(_) => {
                warn!("RAM read not supported");
                0xff
            }
            Self::Mbc1(m) => m.read_ram(addr),
        }
    }

    pub(super) fn write_ram(&mut self, addr: u16, value: u8) {
        match self {
            Self::RomOnly(_) => warn!("RAM write not supported"),
            Self::Mbc1(m) => m.write_ram(addr, value),
        }
    }

    pub(super) fn dump_ram<W: Write>(&self, w: W) -> Result<()> {
        match self {
            Self::RomOnly(_) => Ok(()),
            Self::Mbc1(m) => m.dump_ram(w),
        }
    }
}

pub(super) fn load_rom_only(rom: Memory) -> Result<Mapper> {
    rom_only::load(rom)
        .map(Mapper::RomOnly)
        .context("loading rom_only mapper")
}

pub(super) fn load_mbc1(rom: Memory, ram: Memory) -> Result<Mapper> {
    mbc1::load(rom, ram)
        .map(Mapper::Mbc1)
        .context("loading mbc1 mapper")
}
