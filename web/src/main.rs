mod app;
mod gui;
mod web;

fn main() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    console_log::init_with_level(log::Level::Info).expect("error initializing logging");

    wasm_bindgen_futures::spawn_local(gui::run());
}

