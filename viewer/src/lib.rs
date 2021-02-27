use console_error_panic_hook;
use std::rc::Rc;
use tracing::info;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{self, WebGlProgram, WebGl2RenderingContext as GL, WebGlShader};

mod app;
mod render;
mod store;

use crate::app::App;
use crate::store::{Dimensions, Msg};

/// Used to run the application from the web
#[wasm_bindgen]
pub struct Viewer {
    app: Rc<App>,
    gl: Rc<GL>,
    canvas: Rc<web_sys::HtmlCanvasElement>,
}

fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

fn document() -> web_sys::Document {
    window()
        .document()
        .expect("should have a document on window")
}

fn register_resize_handler(app: Rc<App>) -> Result<(), JsValue> {
    let handler = move |_event: web_sys::DomWindowResizeEventDetail| {
        app.store.borrow_mut().msg(&Msg::WindowResized(Dimensions {
            width: window().inner_width().unwrap().as_f64().unwrap() as u32,
            height: window().inner_height().unwrap().as_f64().unwrap() as u32,
        }));
    };

    let closure = Closure::wrap(Box::new(handler) as Box<dyn FnMut(_)>);
    window().add_event_listener_with_callback("resize", closure.as_ref().unchecked_ref())?;
    closure.forget();

    Ok(())
}

#[wasm_bindgen]
impl Viewer {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Viewer {
        console_error_panic_hook::set_once();
        tracing_wasm::set_as_global_default();

        info!("Starting viewer...");

        let canvas_el = document().get_element_by_id("main").unwrap();
        let canvas: web_sys::HtmlCanvasElement = canvas_el
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("failed converting canvas element to js-sys HtmlCanvasElement");

        let gl = canvas
            .get_context("webgl2")
            .expect("get context webgl2 error")
            .unwrap()
            .dyn_into::<GL>()
            .unwrap();

        let app = Rc::new(App::new());
        app.store.borrow_mut().msg(&Msg::WindowResized(Dimensions {
            width: window().inner_width().unwrap().as_f64().unwrap() as u32,
            height: window().inner_height().unwrap().as_f64().unwrap() as u32,
        }));

        Viewer {
            app,
            gl: Rc::new(gl),
            canvas: Rc::new(canvas),
        }
    }

    pub fn start(&mut self) -> Result<(), JsValue> {
        register_resize_handler(Rc::clone(&self.app))?;

        let vert_shader = compile_shader(
            &self.gl,
            GL::VERTEX_SHADER,
            r#"
            attribute vec4 position;
            void main() {
                gl_Position = position;
            }
            "#,
        )?;
        let frag_shader = compile_shader(
            &self.gl,
            GL::FRAGMENT_SHADER,
            r#"
            void main() {
                gl_FragColor = vec4(1.0, 1.0, 1.0, 1.0);
            }
            "#,
        )?;
        let program = link_program(&self.gl, &vert_shader, &frag_shader)?;
        self.gl.use_program(Some(&program));

        Ok(())
    }

    pub fn render(&mut self) {
        let state = &self.app.store.borrow().state;

        if self.canvas.width() != state.canvas_dimensions.width
            || self.canvas.height() != state.canvas_dimensions.height
        {
            self.canvas.set_width(state.canvas_dimensions.width);
            self.canvas.set_height(state.canvas_dimensions.height);
        }

        self.app.render(&self.gl, state);
    }
}

pub fn compile_shader(
    gl: &GL,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, String> {
    let shader = gl
        .create_shader(shader_type)
        .ok_or_else(|| String::from("Unable to create shader object"))?;
    gl.shader_source(&shader, source);
    gl.compile_shader(&shader);

    if gl
        .get_shader_parameter(&shader, GL::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(gl
            .get_shader_info_log(&shader)
            .unwrap_or_else(|| String::from("Unknown error creating shader")))
    }
}

pub fn link_program(
    gl: &GL,
    vert_shader: &WebGlShader,
    frag_shader: &WebGlShader,
) -> Result<WebGlProgram, String> {
    let program = gl
        .create_program()
        .ok_or_else(|| String::from("Unable to create shader object"))?;

    gl.attach_shader(&program, vert_shader);
    gl.attach_shader(&program, frag_shader);
    gl.link_program(&program);

    if gl
        .get_program_parameter(&program, GL::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        Err(gl
            .get_program_info_log(&program)
            .unwrap_or_else(|| String::from("Unknown error creating program object")))
    }
}
