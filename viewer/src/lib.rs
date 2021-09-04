use console_error_panic_hook;
use std::rc::Rc;
use tracing::info;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{self, WebGl2RenderingContext as GL};

mod app;
mod render;
mod store;
mod shader;

use crate::app::App;
use crate::store::{Dimensions, Coordinates, MouseButton, Msg};

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

fn register_mouse_move_handler(app: Rc<App>) -> Result<(), JsValue> {
    let handler = move |event: web_sys::MouseEvent| {
        app.store.borrow_mut().msg(&Msg::MouseMoved((Coordinates {
            x: event.offset_x(),
            y: event.offset_y(),
        },
        event.button().into())));
    };

    let closure = Closure::wrap(Box::new(handler) as Box<dyn FnMut(_)>);
    window().add_event_listener_with_callback("mouseup", closure.as_ref().unchecked_ref())?;
    window().add_event_listener_with_callback("mousedown", closure.as_ref().unchecked_ref())?;
    window().add_event_listener_with_callback("mousemove", closure.as_ref().unchecked_ref())?;
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
            .dyn_into()
            .expect("failed converting canvas element to js-sys HtmlCanvasElement");

        let gl: GL = canvas
            .get_context("webgl2")
            .expect("get context webgl2 error")
            .unwrap()
            .dyn_into()
            .unwrap();

        let app = Rc::new(App::new(&gl));
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
        register_mouse_move_handler(Rc::clone(&self.app))?;
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
