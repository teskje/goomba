use crate::bits::BitsExt;
use crate::state::State;

pub(crate) struct Joypad<'a> {
    j: &'a mut JoypadState,
}

impl<'a> Joypad<'a> {
    pub fn new(state: &'a mut State) -> Self {
        Self {
            j: &mut state.joypad,
        }
    }

    pub fn press_button(&mut self, button: Button) {
        *self.j.buttons.get_mut(button) = true;
    }

    pub fn release_button(&mut self, button: Button) {
        *self.j.buttons.get_mut(button) = false;
    }
}

#[derive(Debug, Default)]
#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct JoypadState {
    buttons: Buttons,
    select: Select,
}

impl JoypadState {
    pub fn read_p1(&self) -> u8 {
        let mut value = 0x00;
        if self.select.direction {
            value |= u8::from(self.buttons.right);
            value |= u8::from(self.buttons.left) << 1;
            value |= u8::from(self.buttons.up) << 2;
            value |= u8::from(self.buttons.down) << 3;
        }
        if self.select.action {
            value |= u8::from(self.buttons.a);
            value |= u8::from(self.buttons.b) << 1;
            value |= u8::from(self.buttons.select) << 2;
            value |= u8::from(self.buttons.start) << 3;
        }

        !value
    }

    pub fn write_p1(&mut self, value: u8) {
        self.select.direction = !value.bit(4);
        self.select.action = !value.bit(5);
    }
}

#[derive(Debug, Default)]
#[derive(serde::Serialize, serde::Deserialize)]
struct Buttons {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
    a: bool,
    b: bool,
    start: bool,
    select: bool,
}

impl Buttons {
    fn get_mut(&mut self, button: Button) -> &mut bool {
        use Button::*;
        match button {
            Up => &mut self.up,
            Down => &mut self.down,
            Left => &mut self.left,
            Right => &mut self.right,
            A => &mut self.a,
            B => &mut self.b,
            Start => &mut self.start,
            Select => &mut self.select,
        }
    }
}

#[derive(Clone, Debug, Default)]
#[derive(serde::Serialize, serde::Deserialize)]
struct Select {
    direction: bool,
    action: bool,
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
