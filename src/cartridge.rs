use anyhow::{bail, Result};
use structview::View;

const KB: usize = 1024;

#[derive(Clone, Copy, View)]
#[repr(C)]
pub struct Header {
    entry_point: [u8; 4],
    logo: [u8; 48],
    title: [u8; 16],
    new_licensee: [u8; 2],
    sgb_flag: u8,
    cartridge_type: u8,
    rom_size: u8,
    ram_size: u8,
    destination: u8,
    old_licensee: u8,
    version: u8,
    header_checksum: u8,
    global_checksum: [u8; 2],
}

impl Header {
    pub fn parse(data: &[u8]) -> Result<&Header> {
        let header = Header::view(data)?;

        let checksum = compute_header_checksum(data);
        if checksum != header.header_checksum {
            bail!("invalid header checksum");
        }

        Ok(header)
    }

    pub fn mapper_type(&self) -> MapperType {
        use MapperType::*;

        match self.cartridge_type {
            0x00 => None,
            0x01 | 0x02 | 0x03 => Mbc1,
            0x11 | 0x12 | 0x13 => Mbc3,
            code => Unsupported(code),
        }
    }

    pub fn rom_size(&self) -> Result<usize> {
        let size = match self.rom_size {
            s @ 0x00..=0x08 => 32 * KB * (1 << s),
            s => bail!("invalid ROM size: {s:#x}"),
        };
        Ok(size)
    }

    pub fn ram_size(&self) -> Result<usize> {
        let size = match self.ram_size {
            0x00 => 0x00,
            0x02 => 8 * KB,
            0x03 => 32 * KB,
            0x04 => 128 * KB,
            0x05 => 64 * KB,
            s => bail!("invalid RAM size: {s:#x}"),
        };
        Ok(size)
    }
}

fn compute_header_checksum(data: &[u8]) -> u8 {
    let mut checksum: u8 = 0;
    for byte in &data[0x34..=0x4c] {
        checksum = checksum.wrapping_sub(*byte).wrapping_sub(1);
    }
    checksum
}

pub enum MapperType {
    None,
    Mbc1,
    Mbc3,
    Unsupported(u8),
}
