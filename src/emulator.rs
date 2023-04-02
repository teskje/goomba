use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use anyhow::Result;
use log::{error, info};

use crate::cpu::Cpu;
use crate::dma::Dma;
use crate::frame::Frame;
use crate::joypad::Joypad;
use crate::ppu::Ppu;
use crate::state::State;
use crate::timer::Timer;
use crate::{input, Config};

pub fn run(
    config: Config,
    input_rx: mpsc::Receiver<input::Event>,
    frame_tx: mpsc::SyncSender<Frame>,
) -> Result<()> {
    Emulator::load(config, input_rx, frame_tx)?.run()
}

struct Emulator {
    state: State,
    input_rx: mpsc::Receiver<input::Event>,
    frame_tx: mpsc::SyncSender<Frame>,
    save_path: PathBuf,
}

impl Emulator {
    fn load(
        config: Config,
        input_rx: mpsc::Receiver<input::Event>,
        frame_tx: mpsc::SyncSender<Frame>,
    ) -> Result<Self> {
        let mut state = State::load_from(&config.path)?;
        if config.print_serial {
            state.mmu.enable_serial_printing();
        }

        let save_path = config.path.with_extension("gmba");

        Ok(Self {
            state,
            input_rx,
            frame_tx,
            save_path,
        })
    }

    fn run(&mut self) -> Result<()> {
        let fps = 60;
        let frame_duration = Duration::from_secs(1) / fps;

        let mut frame_end = Instant::now() + frame_duration;
        loop {
            self.handle_input();

            let frame = self.render_frame()?;
            if self.frame_tx.send(frame).is_err() {
                info!("frame receiver has hung up; shutting down");
                break;
            }

            let time_left = frame_end - Instant::now();
            frame_end += frame_duration;
            thread::sleep(time_left);
        }

        Ok(())
    }

    fn handle_input(&mut self) {
        for event in self.input_rx.try_iter() {
            match event {
                input::Event::ButtonPress(btn) => Joypad::new(&mut self.state).press_button(btn),
                input::Event::ButtonRelease(btn) => {
                    Joypad::new(&mut self.state).release_button(btn)
                }
                input::Event::Save => self.save_game(),
            }
        }
    }

    fn render_frame(&mut self) -> Result<Frame> {
        let s = &mut self.state;
        loop {
            Timer::new(s).step();
            Cpu::new(s).step()?;
            Dma::new(s).step();

            let mut ppu = Ppu::new(s);
            (0..4).for_each(|_| ppu.step());

            if let Some(frame) = ppu.take_frame() {
                return Ok(frame);
            }
        }
    }

    fn save_game(&self) {
        if let Err(error) = self.state.store_save(&self.save_path) {
            error!("cannot save game: {error}");
        }
    }
}
