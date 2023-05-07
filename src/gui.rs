use anyhow::Result;
use log::{error, info};
use pixels::{Pixels, SurfaceTexture};
use rfd::FileDialog;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

use emulator::{Button, Emulator, Frame};

const CODE_CLOSE: i32 = 0;
const CODE_ERROR: i32 = 1;

pub fn run(emu: Emulator) -> Result<()> {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop)?;
    let size = window.inner_size();
    let surface = SurfaceTexture::new(size.width, size.height, &window);
    let pixels = Pixels::new(Frame::WIDTH, Frame::HEIGHT, surface)?;

    let mut handler = Handler::new(emu, pixels);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = match handler.handle(event) {
            Ok(()) => ControlFlow::Poll,
            Err(code) => ControlFlow::ExitWithCode(code),
        };
    });
}

struct Handler {
    emulator: Emulator,
    pixels: Pixels,
    input: WinitInputHelper,
}

impl Handler {
    fn new(emulator: Emulator, pixels: Pixels) -> Self {
        Self {
            emulator,
            pixels,
            input: WinitInputHelper::new(),
        }
    }

    fn handle(&mut self, event: Event<()>) -> Result<(), i32> {
        if self.input.update(&event) {
            self.handle_close_request()?;
            self.handle_resize()?;
            self.handle_keypresses()?;
            self.render_frame()?;
        }

        if event == Event::LoopDestroyed {
            info!("window closed; shutting down");
            self.save_ram();
        }

        Ok(())
    }

    fn handle_close_request(&self) -> Result<(), i32> {
        if self.input.close_requested() {
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

    fn handle_keypresses(&mut self) -> Result<(), i32> {
        const BUTTON_KEYCODES: [(Button, VirtualKeyCode); 8] = [
            (Button::Up, VirtualKeyCode::Up),
            (Button::Down, VirtualKeyCode::Down),
            (Button::Left, VirtualKeyCode::Left),
            (Button::Right, VirtualKeyCode::Right),
            (Button::A, VirtualKeyCode::X),
            (Button::B, VirtualKeyCode::Z),
            (Button::Start, VirtualKeyCode::Return),
            (Button::Select, VirtualKeyCode::Back),
        ];

        for (button, keycode) in BUTTON_KEYCODES {
            if self.input.key_pressed(keycode) {
                self.emulator.press_button(button);
            }
            if self.input.key_released(keycode) {
                self.emulator.release_button(button);
            }
        }

        if self.input.key_pressed(VirtualKeyCode::S) && self.input.held_control() {
            self.save_state();
        }

        Ok(())
    }

    fn render_frame(&mut self) -> Result<(), i32> {
        let frame = self.emulator.render_frame().map_err(|error| {
            error!("emulator error: {error:#}");
            CODE_ERROR
        })?;

        frame
            .write_into(self.pixels.get_frame_mut())
            .expect("frame buffer has the correct size");

        self.pixels.render().map_err(|error| {
            error!("render error: {error}");
            CODE_ERROR
        })
    }

    fn save_ram(&self) {
        let path = FileDialog::new()
            .set_title("Save cartridge RAM?")
            .add_filter("RAM dump", &["gb-ram"])
            .save_file();

        if let Some(path) = path {
            self.emulator.save_ram(&path);
        }
    }

    fn save_state(&self) {
        let path = FileDialog::new()
            .set_title("Pick a savestate file")
            .add_filter("savestate", &["gb-save"])
            .save_file();

        if let Some(path) = path {
            self.emulator.save_state(&path);
        }
    }
}
