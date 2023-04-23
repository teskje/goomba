use crate::bits::BitsExt;
use crate::frame::Frame;

use super::fetcher::Fetch;
use super::{object, Mode};

#[derive(Debug, Default)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct PpuState {
    pub(super) registers: Registers,
    pub(super) mode: Mode,
    pub(super) line_dots: u16,
    pub(super) draw_x: u8,
    pub(super) window_y: u8,
    pub(super) inside_window: bool,
    pub(super) object_slots: Vec<object::Slot>,
    pub(super) frame: Frame,
    pub(super) oam_scan: object::Scan,
    pub(super) fetch: Fetch,
    pub(super) stat_interrupt: bool,
}

impl PpuState {
    pub fn read_lcdc(&self) -> u8 {
        self.registers.lcdc.0
    }

    pub fn write_lcdc(&mut self, value: u8) {
        self.registers.lcdc = Lcdc(value);
    }

    pub fn read_stat(&self) -> u8 {
        self.registers.stat
    }

    pub fn write_stat(&mut self, value: u8) {
        let value = value & 0x78 | self.registers.stat & 0x07;
        self.registers.stat = value;
    }

    pub fn read_scy(&self) -> u8 {
        self.registers.scy
    }

    pub fn write_scy(&mut self, value: u8) {
        self.registers.scy = value;
    }

    pub fn read_scx(&self) -> u8 {
        self.registers.scx
    }

    pub fn write_scx(&mut self, value: u8) {
        self.registers.scx = value;
    }

    pub fn read_ly(&self) -> u8 {
        self.registers.ly
    }

    pub fn read_lyc(&self) -> u8 {
        self.registers.lyc
    }

    pub fn write_lyc(&mut self, value: u8) {
        self.registers.lyc = value;
    }

    pub fn read_bgp(&self) -> u8 {
        self.registers.bgp
    }

    pub fn write_bgp(&mut self, value: u8) {
        self.registers.bgp = value;
    }

    pub fn read_obp0(&self) -> u8 {
        self.registers.obp0
    }

    pub fn write_obp0(&mut self, value: u8) {
        self.registers.obp0 = value;
    }

    pub fn read_obp1(&self) -> u8 {
        self.registers.obp1
    }

    pub fn write_obp1(&mut self, value: u8) {
        self.registers.obp1 = value;
    }

    pub fn read_wy(&self) -> u8 {
        self.registers.wy
    }

    pub fn write_wy(&mut self, value: u8) {
        self.registers.wy = value;
    }

    pub fn read_wx(&self) -> u8 {
        self.registers.wx
    }

    pub fn write_wx(&mut self, value: u8) {
        self.registers.wx = value;
    }
}

#[derive(Debug, Default)]
#[derive(serde::Serialize, serde::Deserialize)]
pub(super) struct Registers {
    pub(super) lcdc: Lcdc,
    pub(super) stat: u8,
    pub(super) scy: u8,
    pub(super) scx: u8,
    pub(super) ly: u8,
    pub(super) lyc: u8,
    pub(super) bgp: u8,
    pub(super) obp0: u8,
    pub(super) obp1: u8,
    pub(super) wy: u8,
    pub(super) wx: u8,
}

impl Registers {
    pub(super) fn refresh_stat_lycly(&mut self) {
        let flag = self.lyc == self.ly;
        self.stat &= 0xfb;
        self.stat |= u8::from(flag) << 2;
    }

    pub(super) fn mode_interrupt(&self) -> bool {
        let mode = self.stat & 0x03;
        match mode {
            0 => self.stat.bit(3),
            1 => self.stat.bit(4),
            2 => self.stat.bit(5),
            3 => false,
            _ => unreachable!(),
        }
    }

    pub(super) fn lyc_interrupt(&self) -> bool {
        let lycly = self.stat.bit(2);
        let enabled = self.stat.bit(6);
        lycly && enabled
    }
}

#[derive(Clone, Copy, Debug, Default)]
#[derive(serde::Serialize, serde::Deserialize)]
pub(super) struct Lcdc(u8);

impl Lcdc {
    pub fn bg_enable(&self) -> bool {
        self.0.bit(0)
    }

    pub fn obj_enable(&self) -> bool {
        self.0.bit(1)
    }

    pub fn obj_size(&self) -> bool {
        self.0.bit(2)
    }

    pub fn bg_tile_map(&self) -> bool {
        self.0.bit(3)
    }

    pub fn bg_tile_data(&self) -> bool {
        self.0.bit(4)
    }

    pub fn win_enable(&self) -> bool {
        self.0.bit(5)
    }

    pub fn win_tile_map(&self) -> bool {
        self.0.bit(6)
    }

    pub fn ppu_enable(&self) -> bool {
        self.0.bit(7)
    }
}
