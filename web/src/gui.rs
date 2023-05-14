use std::sync::mpsc::{self, Sender, TryRecvError};

use anyhow::bail;
use js_sys::Uint8Array;
use log::error;
use wasm_bindgen::JsCast;
use web_sys::FileReader;
use winit::dpi::LogicalSize;
use winit::event_loop::EventLoop;
use winit::platform::web::{WindowBuilderExtWebSys, WindowExtWebSys};
use winit::window::{Window, WindowBuilder};

use crate::app::{App, GuiEvent};
use crate::web;

pub async fn run() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(lcd_size())
        .with_prevent_default(false)
        .with_focusable(false)
        .build(&event_loop)
        .expect("error building window");

    attach_canvas(&window);

    let mut app = App::create(window).await;

    let (event_tx, event_rx) = mpsc::channel();
    register_event_listeners(event_tx);

    let mut try_handle = move |event: winit::event::Event<()>| {
        match event_rx.try_recv() {
            Ok(event) => app.handle_gui_event(event)?,
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => bail!("event_tx disconnected"),
        }

        app.handle_winit_event(event)
    };

    event_loop.run(move |event, _, control_flow| {
        if let Err(error) = try_handle(event) {
            error!("{error}");
            control_flow.set_exit_with_code(1);
        } else {
            control_flow.set_poll();
        }
    });
}

fn attach_canvas(window: &Window) {
    let canvas = web_sys::Element::from(window.canvas());
    lcd_element()
        .append_child(&canvas)
        .expect("error appending canvas");
}

fn register_event_listeners(event_tx: Sender<GuiEvent>) {
    // resizing
    web::add_event_listener(&web::window(), "resize", {
        let tx = event_tx.clone();
        move |e| on_window_resized(e, tx.clone())
    });

    // key presses
    web::add_event_listener(&web::window(), "keydown", {
        let tx = event_tx.clone();
        move |e| on_key_pressed(e, tx.clone())
    });
    web::add_event_listener(&web::window(), "keyup", {
        let tx = event_tx.clone();
        move |e| on_key_released(e, tx.clone())
    });

    // menu buttons
    web::add_event_listener(&rom_input_element(), "change", {
        let tx = event_tx.clone();
        move |e| on_rom_input_changed(e, tx.clone())
    });
}

fn on_window_resized(_e: web_sys::Event, tx: Sender<GuiEvent>) {
    let event = GuiEvent::Resized(lcd_size());
    tx.send(event).unwrap();
}

fn on_key_pressed(e: web_sys::Event, tx: Sender<GuiEvent>) {
    let kbe = e.unchecked_ref::<web_sys::KeyboardEvent>();
    let key = kbe.key();
    if let Some(event) = GuiEvent::for_key_press(&key) {
        tx.send(event).unwrap();
    }
}

fn on_key_released(e: web_sys::Event, tx: Sender<GuiEvent>) {
    let kbe = e.unchecked_ref::<web_sys::KeyboardEvent>();
    let key = kbe.key();
    if let Some(event) = GuiEvent::for_key_release(&key) {
        tx.send(event).unwrap();
    }
}

fn on_rom_input_changed(_e: web_sys::Event, tx: Sender<GuiEvent>) {
    let input = rom_input_element();
    let files = input.files().unwrap();
    let Some(file) = files.item(0) else { return };

    let reader = FileReader::new().unwrap();
    reader.read_as_array_buffer(&file).unwrap();

    let onload = web::js_function(move |event| {
        let reader: FileReader = event.target().unwrap().unchecked_into();
        let result = reader.result().unwrap();
        let array = Uint8Array::new(&result);
        let data = array.to_vec();
        let event = GuiEvent::FileLoaded(data);
        tx.send(event).unwrap();
    });
    reader.set_onload(Some(&onload));
}

fn lcd_element() -> web_sys::HtmlDivElement {
    web::get_element_by_id("lcd").expect("lcd element missing")
}

fn rom_input_element() -> web_sys::HtmlInputElement {
    web::get_element_by_id("input-rom").expect("rom input missing")
}

fn lcd_size() -> LogicalSize<u32> {
    let lcd = lcd_element();
    LogicalSize {
        width: lcd.client_width() as u32,
        height: lcd.client_height() as u32,
    }
}
