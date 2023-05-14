use anyhow::{Context, Result};
use emulator::{Button, Emulator, Frame};
use pixels::{Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::Event as WinitEvent;
use winit::window::Window;

pub struct App {
    window: Window,
    pixels: Pixels,
    emulator: Option<Emulator>,
}

impl App {
    pub async fn create(window: Window) -> Self {
        let size = window.inner_size();
        let surface = SurfaceTexture::new(size.width, size.height, &window);
        let pixels = Pixels::new_async(Frame::WIDTH, Frame::HEIGHT, surface)
            .await
            .expect("error creating Pixels");

        Self {
            window,
            pixels,
            emulator: None,
        }
    }

    pub fn handle_winit_event(&mut self, event: WinitEvent<()>) -> Result<()> {
        if event == WinitEvent::MainEventsCleared {
            self.render_frame()?;
        }

        Ok(())
    }

    pub fn handle_gui_event(&mut self, event: GuiEvent) -> Result<()> {
        use GuiEvent::*;
        match event {
            Resized(size) => self.resize(size)?,
            FileLoaded(data) => self.load_emulator(data)?,
            ButtonPressed(button) => self.press_button(button),
            ButtonReleased(button) => self.release_button(button),
        }

        Ok(())
    }

    fn resize(&mut self, size: LogicalSize<u32>) -> Result<()> {
        self.window.set_inner_size(size);

        let phys_size = self.window.inner_size();
        self.pixels
            .resize_surface(phys_size.width, phys_size.height)
            .context("resize error")
    }

    fn load_emulator(&mut self, data: Vec<u8>) -> Result<()> {
        let emu = Emulator::load(data, None)?;
        self.emulator = Some(emu);
        Ok(())
    }

    fn render_frame(&mut self) -> Result<()> {
        let frame = match &mut self.emulator {
            Some(emu) => emu.render_frame()?,
            None => Frame::default(),
        };

        frame
            .write_into(self.pixels.get_frame_mut())
            .expect("frame buffer has the correct size");

        self.pixels.render().context("render error")
    }

    fn press_button(&mut self, button: Button) {
        if let Some(emu) = &mut self.emulator {
            emu.press_button(button);
        }
    }

    fn release_button(&mut self, button: Button) {
        if let Some(emu) = &mut self.emulator {
            emu.release_button(button);
        }
    }
}

pub enum GuiEvent {
    Resized(LogicalSize<u32>),
    FileLoaded(Vec<u8>),
    ButtonPressed(Button),
    ButtonReleased(Button),
}

impl GuiEvent {
    pub fn for_key_press(key: &str) -> Option<Self> {
        key_to_button(key).map(Self::ButtonPressed)
    }

    pub fn for_key_release(key: &str) -> Option<Self> {
        key_to_button(key).map(Self::ButtonReleased)
    }
}

fn key_to_button(key: &str) -> Option<Button> {
    let button = match key {
        "ArrowUp" => Button::Up,
        "ArrowDown" => Button::Down,
        "ArrowLeft" => Button::Left,
        "ArrowRight" => Button::Right,
        "x" => Button::A,
        "z" => Button::B,
        "Enter" => Button::Start,
        "Backspace" => Button::Select,
        _ => return None,
    };
    Some(button)
}
