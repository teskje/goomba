use crate::bits::BitsExt;
use crate::mmu::Mmu;
use crate::state::State;

#[derive(Clone, Copy, Debug, Default)]
#[derive(serde::Serialize, serde::Deserialize)]
pub(super) struct Slot {
    pub(super) index: u16,
    pub(super) y: u8,
    pub(super) x: u8,
}

#[derive(Clone, Debug, Default)]
#[derive(serde::Serialize, serde::Deserialize)]
pub(super) struct Attributes {
    pub(super) y: u8,
    pub(super) _x: u8,
    pub(super) tile_id: u8,
    pub(super) flags: Flags,
}

#[derive(Clone, Copy, Debug, Default)]
#[derive(serde::Serialize, serde::Deserialize)]
pub(super) struct Flags(u8);

impl Flags {
    pub fn palette(&self) -> bool {
        self.0.bit(4)
    }

    pub fn x_flip(&self) -> bool {
        self.0.bit(5)
    }

    pub fn y_flip(&self) -> bool {
        self.0.bit(6)
    }

    pub fn bg_over_obj(&self) -> bool {
        self.0.bit(7)
    }
}

pub(super) struct Scanner<'a> {
    s: &'a mut State,
}

impl<'a> Scanner<'a> {
    pub(super) fn new(state: &'a mut State) -> Self {
        Self { s: state }
    }

    pub(super) fn step(&mut self) {
        if self.s.ppu.object_slots.len() == 10 {
            return;
        }

        match self.s.ppu.oam_scan.stash.take() {
            None => self.step_y(),
            Some(y) => self.step_x(y),
        }
    }

    fn step_y(&mut self) {
        let index = self.s.ppu.oam_scan.index;
        let y = read_y(self.s, index);
        self.s.ppu.oam_scan.stash = Some(y);
    }

    fn step_x(&mut self, y: u8) {
        let index = self.s.ppu.oam_scan.index;
        let x = read_x(self.s, index);

        let r = &self.s.ppu.registers;
        let obj_height = if r.lcdc.obj_size() { 16 } else { 8 };
        let max_y = r.ly + 16;
        let min_y = max_y - obj_height + 1;

        if y >= min_y && y <= max_y {
            self.s.ppu.object_slots.push(Slot { index, y, x });
        }

        self.s.ppu.oam_scan.index += 1;
    }
}

#[derive(Debug, Default)]
#[derive(serde::Serialize, serde::Deserialize)]
pub(super) struct Scan {
    index: u16,
    stash: Option<u8>,
}

fn object_address(index: u16) -> u16 {
    0xfe00 + index * 4
}

fn read_y(state: &mut State, index: u16) -> u8 {
    let addr = object_address(index);
    Mmu::new(state).read(addr)
}

fn read_x(state: &mut State, index: u16) -> u8 {
    let addr = object_address(index) + 1;
    Mmu::new(state).read(addr)
}

pub(super) fn read_tile_id(state: &mut State, index: u16) -> u8 {
    let addr = object_address(index) + 2;
    Mmu::new(state).read(addr)
}

pub(super) fn read_flags(state: &mut State, index: u16) -> Flags {
    let addr = object_address(index) + 3;
    Flags(Mmu::new(state).read(addr))
}
