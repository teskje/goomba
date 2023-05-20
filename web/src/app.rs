use std::collections::VecDeque;

use anyhow::{Context, Result};
use emulator::{Button, Emulator, Frame};
use pixels::wgpu::Color;
use pixels::{Pixels, PixelsBuilder, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::Event as WinitEvent;
use winit::window::Window;

pub struct App {
    window: Window,
    pixels: Pixels,
    emulator: Option<Emulator>,
    /// A queue containing received button events.
    ///
    /// We don't handle button events directly but defer them to ensure only a single button event
    /// is handled during each frame. This ensures that the game sees all events, even if
    /// successive once occur within the same frame.
    button_events: VecDeque<ButtonEvent>,
}

impl App {
    pub async fn create(window: Window) -> Self {
        let size = window.inner_size();
        let surface = SurfaceTexture::new(size.width, size.height, &window);
        let pixels = PixelsBuilder::new(Frame::WIDTH, Frame::HEIGHT, surface)
            .clear_color(Color::TRANSPARENT)
            .build_async()
            .await
            .expect("error creating Pixels");

        Self {
            window,
            pixels,
            emulator: None,
            button_events: Default::default(),
        }
    }

    pub fn handle_winit_event(&mut self, event: WinitEvent<()>) -> Result<()> {
        if event == WinitEvent::MainEventsCleared {
            self.handle_button_event();
            self.render_frame()?;
        }

        Ok(())
    }

    pub fn handle_gui_event(&mut self, event: GuiEvent) -> Result<()> {
        use GuiEvent::*;
        match event {
            Resized(size) => self.resize(size)?,
            FileLoaded(data) => self.load_emulator(data)?,
            Button(evt) => self.register_button_event(evt),
        }

        Ok(())
    }

    fn handle_button_event(&mut self) {
        let Some(emu) = &mut self.emulator else { return };
        let Some(event) = self.button_events.pop_front() else { return };

        use ButtonEvent::*;

        match event {
            Pressed(btn) => emu.press_button(btn),
            Released(btn) => emu.release_button(btn),
        }
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

    fn register_button_event(&mut self, event: ButtonEvent) {
        if self.emulator.is_some() {
            self.button_events.push_back(event);
        }
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
}

#[derive(Debug)]
pub enum GuiEvent {
    Resized(LogicalSize<u32>),
    FileLoaded(Vec<u8>),
    Button(ButtonEvent),
}

#[derive(Debug)]
pub enum ButtonEvent {
    Pressed(Button),
    Released(Button),
}

impl GuiEvent {
    pub fn button_pressed(btn: Button) -> Self {
        Self::Button(ButtonEvent::Pressed(btn))
    }

    pub fn button_released(btn: Button) -> Self {
        Self::Button(ButtonEvent::Released(btn))
    }

    pub fn for_key_press(key: &str) -> Option<Self> {
        key_to_button(key).map(Self::button_pressed)
    }

    pub fn for_key_release(key: &str) -> Option<Self> {
        key_to_button(key).map(Self::button_released)
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
