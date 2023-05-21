use std::sync::mpsc::{self, Sender, TryRecvError};

use anyhow::bail;
use emulator::Button;
use futures::future::OptionFuture;
use js_sys::{Array, Uint8Array};
use log::error;
use wasm_bindgen::JsCast;
use web_sys::{
    Blob, Element, Event, File, HtmlAnchorElement, HtmlInputElement, KeyboardEvent, PointerEvent,
    Url,
};
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

        if let Some(save) = app.take_ram_save() {
            save_file(&save.ram, &save.name);
        }

        app.handle_winit_event(event)
    };

    event_loop.run(move |event, _, control_flow| {
        if let Err(error) = try_handle(event) {
            error!("{error:#?}");
            control_flow.set_exit_with_code(1);
        } else {
            control_flow.set_poll();
        }
    });
}

fn attach_canvas(window: &Window) {
    let canvas = Element::from(window.canvas());
    get_lcd()
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
    web::add_event_listener(&get_load_input(), "change", {
        let tx = event_tx.clone();
        move |e| on_load_input_change(e, tx.clone())
    });
    web::add_event_listener(&get_save_button(), "click", {
        let tx = event_tx.clone();
        move |e| on_save_button_click(e, tx.clone())
    });

    // joypad buttons
    for (btn, elem) in get_joypad_buttons() {
        web::add_event_listener(&elem, "pointerdown", {
            let tx = event_tx.clone();
            move |e| on_joypad_button_press(e.unchecked_into(), btn, tx.clone())
        });
        web::add_event_listener(&elem, "pointerup", {
            let tx = event_tx.clone();
            move |e| on_joypad_button_release(e.unchecked_into(), btn, tx.clone())
        });
        web::add_event_listener(&elem, "pointerenter", {
            let tx = event_tx.clone();
            move |e| on_button_enter(e.unchecked_into(), btn, tx.clone())
        });
        web::add_event_listener(&elem, "pointerleave", {
            let tx = event_tx.clone();
            move |e| on_joypad_button_release(e.unchecked_into(), btn, tx.clone())
        });
    }
}

fn on_window_resize(_e: Event, tx: Sender<GuiEvent>) {
    let event = GuiEvent::Resize(lcd_size());
    tx.send(event).unwrap();
}

fn on_key_press(e: KeyboardEvent, tx: Sender<GuiEvent>) {
    if let Some(event) = GuiEvent::for_key_press(&e.key()) {
        tx.send(event).unwrap();
    }
}

fn on_key_release(e: KeyboardEvent, tx: Sender<GuiEvent>) {
    if let Some(event) = GuiEvent::for_key_release(&e.key()) {
        tx.send(event).unwrap();
    }
}

fn on_joypad_button_press(e: PointerEvent, btn: Button, tx: Sender<GuiEvent>) {
    let target = e.target().unwrap();
    let elem: &Element = target.unchecked_ref();
    elem.release_pointer_capture(e.pointer_id()).unwrap();

    let event = GuiEvent::button_pressed(btn);
    tx.send(event).unwrap();
}

fn on_joypad_button_release(_e: PointerEvent, btn: Button, tx: Sender<GuiEvent>) {
    let event = GuiEvent::button_released(btn);
    tx.send(event).unwrap();
}

fn on_button_enter(e: PointerEvent, btn: Button, tx: Sender<GuiEvent>) {
    if e.pointer_type() == "touch" {
        on_joypad_button_press(e, btn, tx);
    }
}

fn on_load_input_change(_e: Event, tx: Sender<GuiEvent>) {
    let input = get_load_input();
    let files = input.files().unwrap();

    let mut rom = None;
    let mut ram = None;
    let mut i = 0;
    while let Some(file) = files.item(i) {
        let name = file.name();
        if name.ends_with(".gb") {
            rom = Some(file);
        } else if name.ends_with(".gb-ram") {
            ram = Some(file);
        }
        i += 1;
    }

    if let Some(rom) = rom {
        load_rom_and_ram(rom, ram, tx);
    } else {
        web::window().alert_with_message("No ROM provided").unwrap();
    }
}

fn load_rom_and_ram(rom: File, ram: Option<File>, tx: Sender<GuiEvent>) {
    let filename = rom.name();
    let name = match filename.strip_suffix(".gb") {
        Some(name) => name.into(),
        None => filename,
    };

    let read_rom = read_file(rom);
    let read_ram: OptionFuture<_> = ram.map(|f| read_file(f)).into();

    wasm_bindgen_futures::spawn_local(async move {
        let (rom, ram) = futures::join!(read_rom, read_ram);
        let event = GuiEvent::LoadGame { name, rom, ram };
        tx.send(event).unwrap();
    });
}

async fn read_file(file: File) -> Vec<u8> {
    let promise = file.array_buffer();
    let fut = wasm_bindgen_futures::JsFuture::from(promise);
    let result = fut.await.unwrap();

    let array = Uint8Array::new(&result);
    array.to_vec()
}

fn save_file(data: &[u8], name: &str) {
    let array = Array::new();
    array.push(&Uint8Array::from(data));
    let blob = Blob::new_with_u8_array_sequence(&array).unwrap();
    let url = Url::create_object_url_with_blob(&blob).unwrap();

    let a = web::document().create_element("a").unwrap();
    let a: HtmlAnchorElement = a.unchecked_into();
    a.set_href(&url);
    a.set_download(name);
    a.click();
}

fn on_save_button_click(_e: Event, tx: Sender<GuiEvent>) {
    let event = GuiEvent::SaveRam;
    tx.send(event).unwrap();
}

fn get_lcd() -> Element {
    web::get_element_by_id("lcd")
}

fn get_load_input() -> HtmlInputElement {
    web::get_element_by_id("input-load")
}

fn get_save_button() -> Element {
    web::get_element_by_id("button-save")
}

fn get_joypad_buttons() -> Vec<(Button, Element)> {
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
        .map(|(btn, id)| (btn, web::get_element_by_id(id)))
        .collect()
}

fn lcd_size() -> LogicalSize<u32> {
    let lcd = get_lcd();
    LogicalSize {
        width: lcd.client_width() as u32,
        height: lcd.client_height() as u32,
    }
}
