use std::cmp;

use anyhow::{bail, Result};
use log::warn;

use crate::mmu::memory::Memory;
use crate::mmu::{memory, KB};

const ROM_BANK_SIZE: u16 = 16 * KB as u16;
const RAM_BANK_SIZE: u16 = 8 * KB as u16;

#[derive(Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub(in crate::mmu) struct Mapper {
    rom: memory::Banked<ROM_BANK_SIZE>,
    ram: memory::Banked<RAM_BANK_SIZE>,
    rom_bank_nr: u8,
}

impl Mapper {
    pub(super) fn read_rom(&self, addr: u16) -> u8 {
        if addr < 0x4000 {
            self.rom[(0, addr)]
        } else {
            self.read_high_rom(addr - 0x4000)
        }
    }

    fn read_high_rom(&self, addr: u16) -> u8 {
        self.rom.get(self.rom_bank_nr, addr).unwrap_or_else(|| {
            warn!("invalid ROM read addr: {addr:#x}");
            0xff
        })
    }

    pub(super) fn write_rom(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..=0x1fff => (), // RAM enable
            0x2000..=0x3fff => {
                let nr = cmp::max(value & 0x1f, 1);
                self.rom_bank_nr = nr;
            }
            0x4000..=0x5fff => (), // RAM bank number
            0x6000..=0x7fff => (), // banking mode select
            _ => warn!("invalid RAM write addr: {addr:#x}"),
        }
    }

    pub(super) fn read_ram(&self, addr: u16) -> u8 {
        self.ram.get(0, addr).unwrap_or_else(|| {
            warn!("invalid RAM read addr: {addr:#x}");
            0xff
        })
    }

    pub(super) fn write_ram(&mut self, addr: u16, value: u8) {
        match self.ram.get_mut(0, addr) {
            Some(v) => *v = value,
            None => warn!("invalid RAM write addr: {addr:#x}"),
        }
    }
}

pub(super) fn load(rom: Memory, ram: Memory) -> Result<Mapper> {
    let rom_size = rom.len();
    let rom = match memory::Banked::try_from(rom) {
        Ok(b) if (1..=32).contains(&b.banks()) => b,
        _ => bail!("invalid ROM size: {rom_size:#x}"),
    };

    let ram_size = ram.len();
    let ram = match memory::Banked::try_from(ram) {
        Ok(b) if (0..=1).contains(&b.banks()) => b,
        _ => bail!("invalid RAM size: {ram_size:#x}"),
    };

    Ok(Mapper {
        rom,
        ram,
        rom_bank_nr: 1,
    })
}
