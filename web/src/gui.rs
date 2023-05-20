use std::sync::mpsc::{self, Sender, TryRecvError};

use anyhow::bail;
use emulator::Button;
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
        move |e| on_window_resize(e, tx.clone())
    });

    // key presses
    web::add_event_listener(&web::window(), "keydown", {
        let tx = event_tx.clone();
        move |e| on_key_press(e.unchecked_into(), tx.clone())
    });
    web::add_event_listener(&web::window(), "keyup", {
        let tx = event_tx.clone();
        move |e| on_key_release(e.unchecked_into(), tx.clone())
    });

    // menu buttons
    web::add_event_listener(&rom_input_element(), "change", {
        let tx = event_tx.clone();
        move |e| on_rom_input_change(e, tx.clone())
    });

    // joypad buttons
    for (btn, elem) in button_elements() {
        web::add_event_listener(&elem, "pointerdown", {
            let tx = event_tx.clone();
            move |e| on_button_press(e.unchecked_into(), btn, tx.clone())
        });
        web::add_event_listener(&elem, "pointerup", {
            let tx = event_tx.clone();
            move |e| on_button_release(e.unchecked_into(), btn, tx.clone())
        });
        web::add_event_listener(&elem, "pointerenter", {
            let tx = event_tx.clone();
            move |e| on_button_enter(e.unchecked_into(), btn, tx.clone())
        });
        web::add_event_listener(&elem, "pointerleave", {
            let tx = event_tx.clone();
            move |e| on_button_release(e.unchecked_into(), btn, tx.clone())
        });
    }
}

fn on_window_resize(_e: web_sys::Event, tx: Sender<GuiEvent>) {
    let event = GuiEvent::Resized(lcd_size());
    tx.send(event).unwrap();
}

fn on_key_press(e: web_sys::KeyboardEvent, tx: Sender<GuiEvent>) {
    if let Some(event) = GuiEvent::for_key_press(&e.key()) {
        tx.send(event).unwrap();
    }
}

fn on_key_release(e: web_sys::KeyboardEvent, tx: Sender<GuiEvent>) {
    if let Some(event) = GuiEvent::for_key_release(&e.key()) {
        tx.send(event).unwrap();
    }
}

fn on_button_press(e: web_sys::PointerEvent, btn: Button, tx: Sender<GuiEvent>) {
    let target = e.target().unwrap();
    let elem: &web_sys::Element = target.unchecked_ref();
    elem.release_pointer_capture(e.pointer_id()).unwrap();

    let event = GuiEvent::button_pressed(btn);
    tx.send(event).unwrap();
}

fn on_button_release(_e: web_sys::PointerEvent, btn: Button, tx: Sender<GuiEvent>) {
    let event = GuiEvent::button_released(btn);
    tx.send(event).unwrap();
}

fn on_button_enter(e: web_sys::PointerEvent, btn: Button, tx: Sender<GuiEvent>) {
    if e.pointer_type() == "touch" {
        on_button_press(e, btn, tx);
    }
}

fn on_rom_input_change(_e: web_sys::Event, tx: Sender<GuiEvent>) {
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
    web::get_element_by_id("input-load").expect("rom input missing")
}

fn button_elements() -> Vec<(Button, web_sys::Element)> {
    let button_ids = [
        (Button::Up, "button-up"),
        (Button::Down, "button-down"),
        (Button::Left, "button-left"),
        (Button::Right, "button-right"),
        (Button::A, "button-a"),
        (Button::B, "button-b"),
        (Button::Start, "button-start"),
        (Button::Select, "button-select"),
    ];

    button_ids
        .into_iter()
        .map(|(btn, id)| {
            let elem = web::get_element_by_id(id).expect("button elementt missing");
            (btn, elem)
        })
        .collect()
}

fn lcd_size() -> LogicalSize<u32> {
    let lcd = lcd_element();
    LogicalSize {
        width: lcd.client_width() as u32,
        height: lcd.client_height() as u32,
    }
}
