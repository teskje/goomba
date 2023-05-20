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
    game: Option<Game>,
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
            game: None,
        }
    }

    pub fn take_ram_save(&mut self) -> Option<RamSave> {
        let Some(game) = &mut self.game else { return None };

        game.ram_save.take()
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
            Resize(size) => self.resize(size)?,
            LoadGame { name, rom, ram } => self.load_game(name, rom, ram)?,
            SaveRam => self.save_ram()?,
            Button(evt) => self.register_button_event(evt),
        }

        Ok(())
    }

    fn handle_button_event(&mut self) {
        let Some(game) = &mut self.game else { return };
        let Some(event) = game.button_events.pop_front() else { return };

        use ButtonEvent::*;

        match event {
            Pressed(btn) => game.emulator.press_button(btn),
            Released(btn) => game.emulator.release_button(btn),
        }
    }

    fn resize(&mut self, size: LogicalSize<u32>) -> Result<()> {
        self.window.set_inner_size(size);

        let phys_size = self.window.inner_size();
        self.pixels
            .resize_surface(phys_size.width, phys_size.height)
            .context("resize error")
    }

    fn load_game(&mut self, name: String, rom: Vec<u8>, ram: Option<Vec<u8>>) -> Result<()> {
        let emulator = Emulator::load(rom, ram)?;
        let game = Game {
            name,
            emulator,
            ram_save: None,
            button_events: Default::default(),
        };
        self.game = Some(game);
        Ok(())
    }

    fn save_ram(&mut self) -> Result<()> {
        let Some(game) = &mut self.game else { return Ok(()) };

        let ram = game.emulator.dump_ram()?;
        let name = format!("{}.gb-ram", game.name);
        game.ram_save = Some(RamSave { name, ram });
        Ok(())
    }

    fn register_button_event(&mut self, event: ButtonEvent) {
        let Some(game) = &mut self.game else { return };

        game.button_events.push_back(event);
    }

    fn render_frame(&mut self) -> Result<()> {
        let frame = match &mut self.game {
            Some(game) => game.emulator.render_frame()?,
            None => Frame::default(),
        };

        frame
            .write_into(self.pixels.get_frame_mut())
            .expect("frame buffer has the correct size");

        self.pixels.render().context("render error")
    }
}

struct Game {
    name: String,
    emulator: Emulator,
    /// Holds the `RamSave` from the last save request.
    ram_save: Option<RamSave>,
    /// A queue containing received button events.
    ///
    /// We don't handle button events directly but defer them to ensure only a single button event
    /// is handled during each frame. This ensures that the game sees all events, even if
    /// successive once occur within the same frame.
    button_events: VecDeque<ButtonEvent>,
}

pub struct RamSave {
    pub name: String,
    pub ram: Vec<u8>,
}

#[derive(Debug)]
pub enum GuiEvent {
    Resize(LogicalSize<u32>),
    LoadGame {
        name: String,
        rom: Vec<u8>,
        ram: Option<Vec<u8>>,
    },
    SaveRam,
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
