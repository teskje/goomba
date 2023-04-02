use std::cmp;

use anyhow::{bail, Result};
use log::warn;

use crate::mmu::KB;
use crate::mmu::bank::Bank;

const ROM_BANK_SIZE: usize = 16 * KB;
const RAM_BANK_SIZE: usize = 8 * KB;

#[derive(Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub(in crate::mmu) struct Mapper {
    rom: Vec<Bank<ROM_BANK_SIZE>>,
    ram: Vec<Bank<RAM_BANK_SIZE>>,
    rom_bank_nr: u8,
}

impl Mapper {
    pub(super) fn read_rom(&self, addr: u16) -> u8 {
        if addr < 0x4000 {
            self.rom[0][addr]
        } else {
            self.read_high_rom(addr - 0x4000)
        }
    }

    fn read_high_rom(&self, addr: u16) -> u8 {
        let bank_nr = usize::from(self.rom_bank_nr);
        let bank = self.rom.get(bank_nr);
        bank.and_then(|b| b.get(addr)).unwrap_or_else(|| {
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
        let bank = self.ram.get(0);
        bank.and_then(|b| b.get(addr)).unwrap_or_else(|| {
            warn!("invalid RAM read addr: {addr:#x}");
            0xff
        })
    }

    pub(super) fn write_ram(&mut self, addr: u16, value: u8) {
        let bank = self.ram.get_mut(0);
        match bank.and_then(|b| b.get_mut(addr)) {
            Some(v) => *v = value,
            None => warn!("invalid RAM write addr: {addr:#x}"),
        }
    }
}

pub(super) fn load(rom: Vec<u8>, ram_size: usize) -> Result<Mapper> {
    let rom = match Bank::split_from(&rom) {
        Ok(banks) if (1..=32).contains(&banks.len()) => banks,
        _ => bail!("invalid ROM size: {:#x}", rom.len()),
    };

    let ram = match ram_size {
        0 => vec![],
        RAM_BANK_SIZE => vec![Default::default()],
        _ => bail!("invalid RAM size: {ram_size:#x}"),
    };

    Ok(Mapper {
        rom,
        ram,
        rom_bank_nr: 1,
    })
}
