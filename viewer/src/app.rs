use std::cell::RefCell;
use std::rc::Rc;

use web_sys::{self, WebGl2RenderingContext as GL};

use crate::render::LineGraph;
use crate::store::*;
use crate::shader::ShaderSystem;

pub struct App {
    pub store: Rc<RefCell<Store>>,
    lg: LineGraph,
    shaders: ShaderSystem,
}

impl App {
    pub fn new(gl: &GL) -> App {
        let store = Rc::new(RefCell::new(Store::new()));
        let lg = LineGraph::new();
        let shaders = ShaderSystem::new(gl);
        App { shaders, store, lg }
    }

    pub fn render(&self, gl: &GL, state: &State) {
        gl.viewport(
            0,
            0,
            state.canvas_dimensions.width as i32,
            state.canvas_dimensions.height as i32,
        );

        gl.clear_color(0.0, 0.5, 0.0, 1.0);
        gl.clear(GL::COLOR_BUFFER_BIT);

        self.lg.render(gl, &self.shaders);
    }
}
