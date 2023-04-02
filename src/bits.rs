use std::ops::RangeInclusive;

pub trait BitsExt: Sized {
    fn bit(&self, i: u8) -> bool;
    fn bits(&self, r: RangeInclusive<u8>) -> Self;
    fn set_bit(&mut self, i: u8);
    fn reset_bit(&mut self, i: u8);
}

impl BitsExt for u8 {
    fn bit(&self, i: u8) -> bool {
        self & (1 << i) != 0
    }

    fn bits(&self, r: RangeInclusive<u8>) -> Self {
        let (s, e) = r.into_inner();
        let mask = ((1_u16 << e + 1) - 1) as u8;
        (self & mask) >> s
    }

    fn set_bit(&mut self, i: u8) {
        *self |= 1 << i;
    }

    fn reset_bit(&mut self, i: u8) {
        *self &= !(1 << i);
    }
}

impl BitsExt for u16 {
    fn bit(&self, i: u8) -> bool {
        self & (1 << i) != 0
    }

    fn bits(&self, r: RangeInclusive<u8>) -> Self {
        let (s, e) = r.into_inner();
        let mask = ((1_u32 << e + 1) - 1) as u16;
        (self & mask) >> s
    }

    fn set_bit(&mut self, i: u8) {
        *self |= 1 << i;
    }

    fn reset_bit(&mut self, i: u8) {
        *self &= !(1 << i);
    }
}

impl BitsExt for i16 {
    fn bit(&self, i: u8) -> bool {
        self & (1 << i) != 0
    }

    fn bits(&self, r: RangeInclusive<u8>) -> Self {
        let (s, e) = r.into_inner();
        let mask = ((1_i32 << e + 1) - 1) as i16;
        (self & mask) >> s
    }

    fn set_bit(&mut self, i: u8) {
        *self |= 1 << i;
    }

    fn reset_bit(&mut self, i: u8) {
        *self &= !(1 << i);
    }
}
