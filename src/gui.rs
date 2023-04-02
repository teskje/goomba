use std::sync::mpsc;

use anyhow::Result;
use log::{error, info};
use pixels::{Pixels, SurfaceTexture};
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

use crate::frame::{self, Frame};
use crate::input::{self, Button};

const CODE_CLOSE: i32 = 0;
const CODE_ERROR: i32 = 1;

pub fn run(frame_rx: mpsc::Receiver<Frame>, input_tx: mpsc::Sender<input::Event>) -> Result<()> {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop)?;
    let size = window.inner_size();
    let surface = SurfaceTexture::new(size.width, size.height, &window);
    let pixels = Pixels::new(frame::WIDTH, frame::HEIGHT, surface)?;

    let mut handler = Handler::new(pixels, frame_rx, input_tx);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = match handler.handle(event) {
            Ok(()) => ControlFlow::Poll,
            Err(code) => ControlFlow::ExitWithCode(code),
        };
    });
}

struct Handler {
    pixels: Pixels,
    frame_rx: mpsc::Receiver<Frame>,
    input_tx: mpsc::Sender<input::Event>,
    input: WinitInputHelper,
}

impl Handler {
    fn new(
        pixels: Pixels,
        frame_rx: mpsc::Receiver<Frame>,
        input_tx: mpsc::Sender<input::Event>,
    ) -> Self {
        Self {
            pixels,
            frame_rx,
            input_tx,
            input: WinitInputHelper::new(),
        }
    }

    fn handle(&mut self, event: Event<()>) -> Result<(), i32> {
        if self.input.update(&event) {
            self.handle_close()?;
            self.handle_resize()?;
            self.handle_keypresses()?;
            self.render_frame()?;
        }

        Ok(())
    }

    fn handle_close(&self) -> Result<(), i32> {
        if self.input.close_requested() {
            info!("window close requested; shutting down");
            Err(CODE_CLOSE)
        } else {
            Ok(())
        }
    }

    fn handle_resize(&mut self) -> Result<(), i32> {
        let Some(size) = self.input.window_resized() else { return Ok(()) };

        self.pixels
            .resize_surface(size.width, size.height)
            .map_err(|error| {
                error!("resize error: {error}");
                CODE_ERROR
            })
    }

    fn handle_keypresses(&self) -> Result<(), i32> {
        for (button, keycode) in Button::KEYCODES {
            if self.input.key_pressed(keycode) {
                self.send_input(input::Event::ButtonPress(button))?;
            }
            if self.input.key_released(keycode) {
                self.send_input(input::Event::ButtonRelease(button))?;
            }
        }

        if self.input.key_pressed(VirtualKeyCode::S) && self.input.held_control() {
            self.send_input(input::Event::Save)?;
        }

        Ok(())
    }

    fn send_input(&self, event: input::Event) -> Result<(), i32> {
        self.input_tx.send(event).map_err(|_| {
            error!("input receiver has hung up");
            CODE_ERROR
        })
    }

    fn render_frame(&mut self) -> Result<(), i32> {
        let Ok(frame) = self.frame_rx.recv() else {
            error!("frame writer has hung up");
            return Err(CODE_ERROR);
        };

        frame
            .write_into(self.pixels.get_frame_mut())
            .expect("frame buffer has the correct size");

        self.pixels.render().map_err(|error| {
            error!("render error: {error}");
            CODE_ERROR
        })
    }
}

impl Button {
    const KEYCODES: [(Self, VirtualKeyCode); 8] = [
        (Self::Up, VirtualKeyCode::Up),
        (Self::Down, VirtualKeyCode::Down),
        (Self::Left, VirtualKeyCode::Left),
        (Self::Right, VirtualKeyCode::Right),
        (Self::A, VirtualKeyCode::X),
        (Self::B, VirtualKeyCode::Z),
        (Self::Start, VirtualKeyCode::Return),
        (Self::Select, VirtualKeyCode::Back),
    ];
}
