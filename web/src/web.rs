use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::{Document, Event, EventTarget, Window};

pub fn window() -> Window {
    web_sys::window().unwrap()
}

pub fn document() -> Document {
    window().document().unwrap()
}

pub fn get_element_by_id<E: JsCast>(id: &str) -> E {
    document()
        .get_element_by_id(id)
        .and_then(|e| e.dyn_into().ok())
        .expect(&format!("element missing: #{id}"))
}

pub fn js_function<F>(f: F) -> js_sys::Function
where
    F: FnMut(Event) + 'static,
{
    let closure = Closure::new(f);
    let js_value = closure.into_js_value();
    js_value.unchecked_into()
}

pub fn add_event_listener<F>(target: &EventTarget, event_type: &str, callback: F)
where
    F: FnMut(Event) + 'static,
{
    let js_callback = js_function(callback);
    target
        .add_event_listener_with_callback(event_type, &js_callback)
        .expect("error setting {event_type} callback");
}
