use std::io::{self, Write};

use anyhow::{bail, Context, Result};
use log::{trace, warn};

use crate::bits::BitsExt;
use crate::cartridge::{self, MapperType};
use crate::state::State;

use self::mapper::Mapper;
use self::memory::Memory;

mod mapper;
mod memory;

const KB: usize = 1024;

const WORK_RAM_SIZE: usize = 8 * KB;
const HIGH_RAM_SIZE: usize = 127;
const VIDEO_RAM_SIZE: usize = 8 * KB;
const OAM_SIZE: usize = 160;

pub fn load_cartridge(rom: Vec<u8>, ram: Option<Vec<u8>>) -> Result<MmuState> {
    let header = cartridge::Header::parse(&rom[0x100..]).context("reading cartridge header")?;
    let rom_size = header.rom_size()?;
    let ram_size = header.ram_size()?;
    let mapper_type = header.mapper_type();

    let rom = Memory::from(rom);
    let ram = match ram {
        Some(buf) => Memory::from(buf),
        None => Memory::with_size(ram_size),
    };

    if usize::from(rom_size) != rom.len() {
        bail!(
            "ROM size mismatch (expected {:#x}, got {:#x})",
            rom_size,
            rom.len()
        );
    }
    if usize::from(ram_size) != ram.len() {
        bail!(
            "RAM size mismatch (expected {:#x}, got {:#x})",
            ram_size,
            ram.len()
        );
    }

    let mapper = match mapper_type {
        MapperType::None => mapper::load_rom_only(rom)?,
        MapperType::Mbc1 => mapper::load_mbc1(rom, ram)?,
        MapperType::Mbc3 => todo!("MBC3"),
        MapperType::Unsupported(code) => bail!("unsupported cartridge type: {code:#x}"),
    };

    Ok(MmuState::new(mapper))
}

pub struct Mmu<'a> {
    s: &'a mut State,
}

impl<'a> Mmu<'a> {
    pub fn new(state: &'a mut State) -> Self {
        Self { s: state }
    }

    pub fn read(&self, addr: u16) -> u8 {
        let data = match addr {
            0x0000..=0x7fff => self.s.mmu.mapper.read_rom(addr),
            0x8000..=0x9fff => self.s.mmu.video_ram[addr - 0x8000],
            0xa000..=0xbfff => self.s.mmu.mapper.read_ram(addr - 0xa000),
            0xc000..=0xdfff => self.s.mmu.work_ram[addr - 0xc000],
            0xfe00..=0xfe9f => self.s.mmu.oam[addr - 0xfe00],
            0xff00 => self.s.joypad.read_p1(),
            0xff04 => self.s.timer.read_div(),
            0xff05 => self.s.timer.read_tima(),
            0xff06 => self.s.timer.read_tma(),
            0xff07 => self.s.timer.read_tac(),
            0xff0f => self.s.cpu.interrupts.read_flag(),
            0xff10..=0xff3f => 0xff, // audio registers
            0xff40 => self.s.ppu.read_lcdc(),
            0xff41 => self.s.ppu.read_stat(),
            0xff42 => self.s.ppu.read_scy(),
            0xff43 => self.s.ppu.read_scx(),
            0xff44 => self.s.ppu.read_ly(),
            0xff45 => self.s.ppu.read_lyc(),
            0xff46 => self.s.dma.read(),
            0xff47 => self.s.ppu.read_bgp(),
            0xff48 => self.s.ppu.read_obp0(),
            0xff49 => self.s.ppu.read_obp1(),
            0xff4a => self.s.ppu.read_wy(),
            0xff4b => self.s.ppu.read_wx(),
            0xff50 => 0xff, // boot ROM enable
            0xff80..=0xfffe => self.s.mmu.high_ram[addr - 0xff80],
            0xffff => self.s.cpu.interrupts.read_enable(),
            _ => {
                warn!("invalid memory read addr: {addr:#x}");
                0xff
            }
        };

        trace!("read: {addr:#04x} -> {data:#02x}");
        data
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        trace!("write: {addr:#04x} <- {value:#02x}");

        match addr {
            0x0000..=0x7fff => self.s.mmu.mapper.write_rom(addr, value),
            0x8000..=0x9fff => self.s.mmu.video_ram[addr - 0x8000] = value,
            0xa000..=0xbfff => self.s.mmu.mapper.write_ram(addr - 0xa000, value),
            0xc000..=0xdfff => self.s.mmu.work_ram[addr - 0xc000] = value,
            0xfe00..=0xfe9f => self.s.mmu.oam[addr - 0xfe00] = value,
            0xff00 => self.s.joypad.write_p1(value),
            0xff01 => self.write_sb(value),
            0xff02 => self.write_sc(value),
            0xff04 => self.s.timer.write_div(value),
            0xff05 => self.s.timer.write_tima(value),
            0xff06 => self.s.timer.write_tma(value),
            0xff07 => self.s.timer.write_tac(value),
            0xff0f => self.s.cpu.interrupts.write_flag(value),
            0xff10..=0xff3f => (), // audio registers
            0xff40 => self.s.ppu.write_lcdc(value),
            0xff41 => self.s.ppu.write_stat(value),
            0xff42 => self.s.ppu.write_scy(value),
            0xff43 => self.s.ppu.write_scx(value),
            0xff45 => self.s.ppu.write_lyc(value),
            0xff46 => self.s.dma.write(value),
            0xff47 => self.s.ppu.write_bgp(value),
            0xff48 => self.s.ppu.write_obp0(value),
            0xff49 => self.s.ppu.write_obp1(value),
            0xff4a => self.s.ppu.write_wy(value),
            0xff4b => self.s.ppu.write_wx(value),
            0xff50 => (), // boot ROM enable
            0xff80..=0xfffe => self.s.mmu.high_ram[addr - 0xff80] = value,
            0xffff => self.s.cpu.interrupts.write_enable(value),
            _ => warn!("unknown write address: {addr:#x}"),
        };
    }

    fn write_sb(&mut self, value: u8) {
        if let Some(out) = &mut self.s.mmu.serial_out {
            *out = value;
        }
    }

    fn write_sc(&mut self, value: u8) {
        if value.bit(7) {
            self.s.mmu.serial_out.map(|b| {
                print!("{}", char::from(b));
                io::stdout().flush().unwrap();
            });
        }
    }
}

#[derive(Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct MmuState {
    mapper: Mapper,
    work_ram: Memory,
    high_ram: Memory,
    video_ram: Memory,
    oam: Memory,
    serial_out: Option<u8>,
}

impl MmuState {
    fn new(mapper: Mapper) -> Self {
        Self {
            mapper,
            work_ram: Memory::with_size(WORK_RAM_SIZE),
            high_ram: Memory::with_size(HIGH_RAM_SIZE),
            video_ram: Memory::with_size(VIDEO_RAM_SIZE),
            oam: Memory::with_size(OAM_SIZE),
            serial_out: None,
        }
    }

    pub fn enable_serial_printing(&mut self) {
        self.serial_out = Some(0);
    }

    pub fn store_ram<W: Write>(&self, w: W) -> Result<()> {
        self.mapper.store_ram(w)
    }
}
