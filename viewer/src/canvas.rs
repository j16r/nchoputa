use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::WebGlRenderingContext as GL;
use web_sys::*;

pub static APP_DIV_ID: &'static str = "main";

pub fn create_webgl_context() -> Result<WebGlRenderingContext, JsValue> {
    let canvas = init_canvas()?;

    let gl: WebGlRenderingContext = canvas.get_context("webgl")?.unwrap().dyn_into()?;

    gl.clear_color(1.0, 1.0, 1.0, 1.0);
    gl.enable(GL::DEPTH_TEST);

    Ok(gl)
}

fn init_canvas() -> Result<HtmlCanvasElement, JsValue> {
    let window = window().unwrap();
    let document = window.document().unwrap();

    let canvas: HtmlCanvasElement = document.create_element("canvas").unwrap().dyn_into()?;

    canvas.style().set_property("width", "100%")?;
    canvas.style().set_property("height", "100%")?;

    let app_div: HtmlElement = match document.get_element_by_id(APP_DIV_ID) {
        Some(container) => container.dyn_into()?,
        None => {
            let app_div = document.create_element("div")?;
            app_div.set_id(APP_DIV_ID);
            app_div.dyn_into()?
        }
    };

    app_div.style().set_property("width", "100%")?;
    app_div.style().set_property("height", "100%")?;
    app_div.append_child(&canvas)?;

    Ok(canvas)
}
