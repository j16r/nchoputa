use std::ops::Deref;

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

pub struct Dimensions {
    pub width: u32,
    pub height: u32,
}

pub enum Msg {
    WindowResized(Dimensions),
}
