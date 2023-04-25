use crate::mmu::Mmu;
use crate::state::State;

pub(crate) struct Dma<'a> {
    s: &'a mut State,
}

impl<'a> Dma<'a> {
    pub fn new(state: &'a mut State) -> Self {
        Self { s: state }
    }

    pub fn step(&mut self) {
        if self.s.dma.progress.is_some() {
            self.advance();
        } else if self.s.dma.triggered {
            self.start();
        }
    }

    fn start(&mut self) {
        let base = u16::from(self.s.dma.source_addr_high) << 8;
        self.s.dma.progress = Some(Progress { base, offset: 0 });
        self.advance();
    }

    fn advance(&mut self) {
        let progress = self.s.dma.progress.unwrap();
        let src = progress.base + progress.offset;
        let dst = 0xfe00 + progress.offset;

        let mut mmu = Mmu::new(self.s);
        let value = mmu.read(src);
        mmu.write(dst, value);

        let progress = &mut self.s.dma.progress.as_mut().unwrap();
        progress.offset += 1;
        if progress.offset == 160 {
            self.stop();
        }
    }

    fn stop(&mut self) {
        self.s.dma.progress = None;
        self.s.dma.triggered = false;
    }
}

#[derive(Debug, Default)]
#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct DmaState {
    source_addr_high: u8,
    triggered: bool,
    progress: Option<Progress>,
}

impl DmaState {
    pub fn read(&self) -> u8 {
        self.source_addr_high
    }

    pub fn write(&mut self, value: u8) {
        self.source_addr_high = value;
        self.triggered = true;
    }
}

#[derive(Clone, Copy, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
struct Progress {
    base: u16,
    offset: u16,
}
