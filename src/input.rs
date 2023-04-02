#[derive(Clone, Copy, Debug)]
pub enum Event {
    ButtonPress(Button),
    ButtonRelease(Button),
    Save,
}

#[derive(Clone, Copy, Debug)]
pub enum Button {
    Up,
    Down,
    Left,
    Right,
    A,
    B,
    Start,
    Select,
}
