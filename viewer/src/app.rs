use std::cell::RefCell;
use std::rc::Rc;

use web_sys::{self, WebGlRenderingContext};

use crate::render::LineGraph;
use crate::store::*;

pub struct App {
    pub store: Rc<RefCell<Store>>,
    lg: LineGraph,
}

impl App {
    pub fn new() -> App {
        let store = Rc::new(RefCell::new(Store::new()));
        let lg = LineGraph::new();
        App { store, lg }
    }

    pub fn render(&self, gl: &WebGlRenderingContext, state: &State) {
        gl.viewport(
            0,
            0,
            state.canvas_dimensions.width as i32,
            state.canvas_dimensions.height as i32,
        );

        gl.clear_color(0.0, 0.0, 0.0, 1.0);
        gl.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);

        self.lg.render(gl);
    }
}
