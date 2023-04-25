use std::mem;

use crate::bits::BitsExt;
use crate::frame::Color;
use crate::mmu::Mmu;
use crate::state::State;

use super::fifo::Fifo;
use super::object;

pub(super) struct Fetcher<'a> {
    s: &'a mut State,
}

impl<'a> Fetcher<'a> {
    pub(super) fn new(state: &'a mut State) -> Self {
        Self { s: state }
    }

    pub(super) fn step(&mut self) -> Option<Color> {
        self.s.ppu.fetch.step = match mem::take(&mut self.s.ppu.fetch.step) {
            Step::Bg(step) => BgFetcher::new(self.s).step(step),
            Step::Obj(step) => ObjFetcher::new(self.s).step(step),
        };

        self.step_object_fifo();
        self.pop_pixel()
    }

    fn step_object_fifo(&mut self) {
        if self.s.ppu.fetch.obj_fifo.is_full() {
            if !self.waiting_for_objects() {
                self.s.ppu.fetch.obj_fifo.unlock();
            }
            return;
        }

        self.s.ppu.fetch.obj_fifo.fill();
        self.s.ppu.fetch.obj_x += 1;

        if self.s.ppu.registers.lcdc.obj_enable() {
            let mut objects = self.find_hit_objects();
            objects.reverse();
            self.s.ppu.fetch.pending_objects = objects;
        }

        if self.waiting_for_objects() {
            self.s.ppu.fetch.obj_fifo.lock();
        }
    }

    fn waiting_for_objects(&self) -> bool {
        let fetch = &self.s.ppu.fetch;
        !fetch.pending_objects.is_empty() || matches!(fetch.step, Step::Obj(_))
    }

    fn find_hit_objects(&self) -> Vec<object::Slot> {
        let x = self.s.ppu.fetch.obj_x;
        let slots = &self.s.ppu.object_slots;
        slots.iter().filter(|obj| obj.x == x).copied().collect()
    }

    fn pop_pixel(&mut self) -> Option<Color> {
        let bg_px = self.s.ppu.fetch.bg_fifo.pop();
        let obj_px = self.s.ppu.fetch.obj_fifo.pop();

        match (bg_px, obj_px) {
            (Some(bg), Some(obj)) => Some(self.mix_pixels(bg, obj)),
            (Some(bg), None) => {
                self.s.ppu.fetch.bg_fifo.put_back(bg);
                None
            }
            (None, Some(obj)) => {
                self.s.ppu.fetch.obj_fifo.put_back(obj);
                None
            }
            (None, None) => None,
        }
    }

    fn mix_pixels(&self, bg_px: ColorIdx, obj_px: ObjPixel) -> Color {
        fn translate_color(palette: u8, idx: ColorIdx) -> Color {
            let bits = match idx {
                ColorIdx::C0 => palette.bits(0..=1),
                ColorIdx::C1 => palette.bits(2..=3),
                ColorIdx::C2 => palette.bits(4..=5),
                ColorIdx::C3 => palette.bits(6..=7),
            };
            let colors = [Color::White, Color::Light, Color::Dark, Color::Black];
            colors[usize::from(bits)]
        }

        let r = &self.s.ppu.registers;
        let obj_wins = match (bg_px, obj_px.color, r.lcdc.bg_enable()) {
            (_, ColorIdx::C0, _) => false,
            (_, _, false) => true,
            (ColorIdx::C0, _, true) => true,
            (_, _, true) => !obj_px.bg_over_obj,
        };

        if obj_wins {
            let obp = if obj_px.palette1 { r.obp1 } else { r.obp0 };
            translate_color(obp, obj_px.color)
        } else if r.lcdc.bg_enable() {
            translate_color(r.bgp, bg_px)
        } else {
            Color::White
        }
    }
}

#[derive(Debug, Default)]
#[derive(serde::Serialize, serde::Deserialize)]
pub(super) struct Fetch {
    bg_x: u8,
    bg_fifo: Fifo<ColorIdx>,
    obj_x: u8,
    obj_fifo: Fifo<ObjPixel>,
    pending_objects: Vec<object::Slot>,
    step: Step,
}

impl Fetch {
    pub(super) fn new(scx: u8) -> Self {
        Self {
            bg_fifo: Fifo::with_discard(scx & 7),
            obj_fifo: Fifo::with_discard(7),
            ..Default::default()
        }
    }

    pub(super) fn enable_window(&mut self) {
        self.bg_x = 0;
        self.bg_fifo = Default::default();
        self.step = Default::default();
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[derive(serde::Serialize, serde::Deserialize)]
enum ColorIdx {
    #[default]
    C0,
    C1,
    C2,
    C3,
}

#[derive(Debug, Default)]
#[derive(serde::Serialize, serde::Deserialize)]
struct ObjPixel {
    color: ColorIdx,
    palette1: bool,
    bg_over_obj: bool,
}

#[derive(Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
enum Step {
    Bg(BgStep),
    Obj(ObjStep),
}

impl Default for Step {
    fn default() -> Self {
        Self::Bg(Default::default())
    }
}

#[derive(Debug, Default)]
#[derive(serde::Serialize, serde::Deserialize)]
enum BgStep {
    #[default]
    LocateTileId,
    FetchTileId {
        addr: u16,
    },
    LocateTileRowLow {
        id: u8,
    },
    FetchTileRowLow {
        addr: u16,
        id: u8,
    },
    LocateTileRowHigh {
        id: u8,
        low: u8,
    },
    FetchTileRowHigh {
        addr: u16,
        low: u8,
    },
    PushPixels {
        low: u8,
        high: u8,
    },
}

struct BgFetcher<'a> {
    s: &'a mut State,
}

impl<'a> BgFetcher<'a> {
    fn new(state: &'a mut State) -> Self {
        Self { s: state }
    }

    fn step(&mut self, step: BgStep) -> Step {
        use BgStep::*;

        let next_step = match step {
            LocateTileId => {
                let addr = self.tile_id_addr();
                FetchTileId { addr }
            }
            FetchTileId { addr } => {
                let id = Mmu::new(self.s).read(addr);
                LocateTileRowLow { id }
            }
            LocateTileRowLow { id } => {
                let addr = self.tile_row_addr(id, false);
                FetchTileRowLow { addr, id }
            }
            FetchTileRowLow { addr, id } => {
                let low = Mmu::new(self.s).read(addr);
                LocateTileRowHigh { id, low }
            }
            LocateTileRowHigh { id, low } => {
                let addr = self.tile_row_addr(id, true);
                FetchTileRowHigh { addr, low }
            }
            FetchTileRowHigh { addr, low } => {
                let high = Mmu::new(self.s).read(addr);
                PushPixels { low, high }
            }
            PushPixels { low, high } => {
                return self.push_pixels(low, high);
            }
        };
        Step::Bg(next_step)
    }

    fn tile_id_addr(&mut self) -> u16 {
        let x = self.s.ppu.fetch.bg_x;
        let r = &self.s.ppu.registers;

        let (y, x, high_base) = match self.s.ppu.inside_window {
            false => (
                r.ly.wrapping_add(r.scy),
                x.wrapping_add(r.scx),
                r.lcdc.bg_tile_map(),
            ),
            true => (self.s.ppu.window_y, x, r.lcdc.win_tile_map()),
        };

        let base = if high_base { 0x9c00 } else { 0x9800 };
        let y_offset = u16::from(y >> 3);
        let x_offset = u16::from(x >> 3);
        base + (y_offset << 5) + x_offset
    }

    fn tile_row_addr(&mut self, id: u8, high: bool) -> u16 {
        let r = &self.s.ppu.registers;

        let y = match self.s.ppu.inside_window {
            false => r.ly.wrapping_add(r.scy),
            true => self.s.ppu.window_y,
        };

        let base = match (r.lcdc.bg_tile_data(), id.bit(7)) {
            (false, false) => 0x9000,
            (false, true) | (true, _) => 0x8000,
        };
        let tile_offset = u16::from(id);
        let row_offset = u16::from(y % 8);
        let high_offset = u16::from(high);
        base + (tile_offset << 4) + (row_offset << 1) + high_offset
    }

    fn push_pixels(&mut self, low: u8, high: u8) -> Step {
        let fetch = &mut self.s.ppu.fetch;

        if !fetch.bg_fifo.is_empty() {
            if let Some(slot) = fetch.pending_objects.pop() {
                return Step::Obj(ObjStep::FetchFlags { slot });
            }
            return Step::Bg(BgStep::PushPixels { low, high });
        }

        let colors = merge_tile_row(low, high);
        fetch.bg_fifo.extend(colors);
        fetch.bg_x += 8;

        Step::Bg(Default::default())
    }
}

#[derive(Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
enum ObjStep {
    FetchFlags {
        slot: object::Slot,
    },
    FetchTileId {
        slot: object::Slot,
        flags: object::Flags,
    },
    LocateTileRowLow {
        obj: object::Attributes,
    },
    FetchTileRowLow {
        addr: u16,
        obj: object::Attributes,
    },
    LocateTileRowHigh {
        obj: object::Attributes,
        low: u8,
    },
    FetchTileRowHigh {
        addr: u16,
        obj: object::Attributes,
        low: u8,
    },
    PushPixels {
        obj: object::Attributes,
        low: u8,
        high: u8,
    },
}

struct ObjFetcher<'a> {
    s: &'a mut State,
}

impl<'a> ObjFetcher<'a> {
    fn new(state: &'a mut State) -> Self {
        Self { s: state }
    }

    fn step(&mut self, step: ObjStep) -> Step {
        use ObjStep::*;

        let next_step = match step {
            FetchFlags { slot } => {
                let flags = object::read_flags(self.s, slot.index);
                FetchTileId { slot, flags }
            }
            FetchTileId { slot, flags } => {
                let tile_id = self.fetch_tile_id(slot, flags);
                LocateTileRowLow {
                    obj: object::Attributes {
                        y: slot.y,
                        _x: slot.x,
                        tile_id,
                        flags,
                    },
                }
            }
            LocateTileRowLow { obj } => {
                let addr = self.tile_row_addr(&obj, false);
                FetchTileRowLow { addr, obj }
            }
            FetchTileRowLow { addr, obj } => {
                let low = self.fetch_tile_row(addr, &obj);
                LocateTileRowHigh { obj, low }
            }
            LocateTileRowHigh { obj, low } => {
                let addr = self.tile_row_addr(&obj, true);
                FetchTileRowHigh { addr, obj, low }
            }
            FetchTileRowHigh { addr, obj, low } => {
                let high = self.fetch_tile_row(addr, &obj);
                PushPixels { obj, low, high }
            }
            PushPixels { obj, low, high } => {
                self.push_pixels(obj, low, high);
                return Step::Bg(Default::default());
            }
        };
        Step::Obj(next_step)
    }

    fn fetch_tile_id(&mut self, slot: object::Slot, flags: object::Flags) -> u8 {
        let tile_id = object::read_tile_id(self.s, slot.index);

        let r = &self.s.ppu.registers;
        let high_objects = r.lcdc.obj_size();
        if high_objects {
            let obj_y = (r.ly + 16).checked_sub(slot.y).unwrap();
            match (obj_y < 8, flags.y_flip()) {
                (true, false) | (false, true) => tile_id & 0xfe,
                (true, true) | (false, false) => tile_id | 0x01,
            }
        } else {
            tile_id
        }
    }

    fn tile_row_addr(&self, obj: &object::Attributes, high: bool) -> u16 {
        let r = &self.s.ppu.registers;

        let base = 0x8000;
        let tile_offset = u16::from(obj.tile_id);
        let mut row_offset = u16::from((r.ly + 16).checked_sub(obj.y).unwrap() % 8);
        if obj.flags.y_flip() {
            row_offset = 7 - row_offset;
        }
        let high_offset = u16::from(high);
        base + (tile_offset << 4) + (row_offset << 1) + high_offset
    }

    fn fetch_tile_row(&mut self, addr: u16, obj: &object::Attributes) -> u8 {
        let row = Mmu::new(self.s).read(addr);
        if obj.flags.x_flip() {
            row.reverse_bits()
        } else {
            row
        }
    }

    fn push_pixels(&mut self, obj: object::Attributes, low: u8, high: u8) {
        let colors = merge_tile_row(low, high);
        for (px, new) in self.s.ppu.fetch.obj_fifo.iter_mut().zip(colors.into_iter()) {
            if px.color == ColorIdx::C0 {
                *px = ObjPixel {
                    color: new,
                    palette1: obj.flags.palette(),
                    bg_over_obj: obj.flags.bg_over_obj(),
                }
            }
        }
    }
}

fn merge_tile_row(low: u8, high: u8) -> [ColorIdx; 8] {
    let mut colors = [ColorIdx::C0; 8];
    for i in 0..8 {
        let bl = low.bit(i);
        let bh = high.bit(i);
        let idx = 7 - i as usize;
        colors[idx] = match (bh, bl) {
            (false, false) => ColorIdx::C0,
            (false, true) => ColorIdx::C1,
            (true, false) => ColorIdx::C2,
            (true, true) => ColorIdx::C3,
        };
    }
    colors
}
