use std::ops::Deref;

use tracing::debug;

pub struct Store {
    pub state: StateWrapper,
}

impl Store {
    pub fn new() -> Store {
        Store {
            state: StateWrapper(State::new()),
        }
    }

    pub fn msg(&mut self, msg: &Msg) {
        match msg {
            _ => self.state.msg(msg),
        }
    }
}

#[derive(Debug)]
pub struct State {
    pub canvas_dimensions: Dimensions,
}

impl State {
    fn new() -> State {
        State {
            canvas_dimensions: Dimensions {
                width: 0,
                height: 0,
            },
        }
    }

    pub fn msg(&mut self, msg: &Msg) {
        match msg {
            Msg::WindowResized(ref dimensions) => {
                self.canvas_dimensions.width = dimensions.width;
                self.canvas_dimensions.height = dimensions.height;
            },
            Msg::MouseMoved((ref coordinates, ref button)) => {
                debug!("mouse coordinates {:?}", coordinates);
                debug!("mouse button {:?}", button);
            }
        }
    }
}

pub struct StateWrapper(State);

impl Deref for StateWrapper {
    type Target = State;

    fn deref(&self) -> &State {
        &self.0
    }
}

impl StateWrapper {
    pub fn msg(&mut self, msg: &Msg) {
        &self.0.msg(msg);
    }
}

#[derive(Debug)]
pub struct Dimensions {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug)]
pub struct Coordinates {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug)]
pub enum MouseButton {
    Main = 0,
    Auxiliary = 1,
    Secondary = 2,
    Fourth = 3,
    Fifth = 4,
}

impl From<i16> for MouseButton {
    fn from(input: i16) -> Self {
        match input {
            0 => MouseButton::Main,
            1 => MouseButton::Auxiliary,
            2 => MouseButton::Secondary,
            3 => MouseButton::Fourth,
            4 => MouseButton::Fifth,
            _ => unreachable!(),
        }
    }
}

pub enum Msg {
    WindowResized(Dimensions),
    MouseMoved((Coordinates, MouseButton)),
}
