use std::io::Write;

use anyhow::{bail, Result};

pub const WIDTH: u32 = 160;
pub const HEIGHT: u32 = 144;

const PIXEL_COUNT: usize = (WIDTH * HEIGHT) as usize;
const RGBA_WHITE: [u8; 4] = [0xff, 0xff, 0xff, 0xff];

#[derive(Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Frame {
    pixels: Vec<Color>,
}

impl Frame {
    pub fn new() -> Self {
        Self {
            pixels: Vec::with_capacity(PIXEL_COUNT),
        }
    }

    pub fn push_pixel(&mut self, color: Color) {
        self.pixels.push(color);
    }

    pub fn write_into(&self, mut buffer: &mut [u8]) -> Result<()> {
        if buffer.len() != PIXEL_COUNT * 4 {
            bail!("invalid buffer size: {}", buffer.len());
        }

        let mut pixels = self.pixels.iter();
        for _ in 0..PIXEL_COUNT {
            let bytes = pixels.next().map(Color::rgba).unwrap_or(RGBA_WHITE);
            buffer.write_all(&bytes).unwrap();
        }

        debug_assert!(pixels.next().is_none());
        Ok(())
    }
}

impl Default for Frame {
    fn default() -> Self {
        Frame::new()
    }
}

#[derive(Clone, Copy, Debug, Default)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum Color {
    #[default]
    White,
    Light,
    Dark,
    Black,
}

impl Color {
    fn rgba(&self) -> [u8; 4] {
        use Color::*;
        match self {
            White => [0xe0, 0xe0, 0xe0, 0xff],
            Light => [0xa0, 0xa0, 0xa0, 0xff],
            Dark => [0x60, 0x60, 0x60, 0xff],
            Black => [0x20, 0x20, 0x20, 0xff],
        }
    }
}
