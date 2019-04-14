extern crate wasm_bindgen;
use self::canvas::*;
use self::render::*;
use console_error_panic_hook;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::*;

mod canvas;
mod render;

#[wasm_bindgen]
pub struct WebClient {
    gl: Rc<WebGlRenderingContext>,
    renderer: WebRenderer,
}


const VERTEX_SHADER_PROGRAM : &'static str= r#"
attribute vec4 aVertexPosition;

uniform mat4 uModelViewMatrix;
uniform mat4 uProjectionMatrix;

void main() {
    gl_Position = uProjectionMatrix * uModelViewMatrix * aVertexPosition;
}
"#;

const FRAGMENT_SHADER_PROGRAM : &'static str= r#"
void main() {
    gl_FragColor = vec4(1.0, 1.0, 1.0, 1.0);
}
"#;


#[wasm_bindgen]
impl WebClient {
    /// Create a new web client
    #[wasm_bindgen(constructor)]
    pub fn new() -> WebClient {
        console_error_panic_hook::set_once();

        let gl = Rc::new(create_webgl_context().unwrap());

        let vertex_shader = compile_shader(&gl, WebGlRenderingContext::VERTEX_SHADER, VERTEX_SHADER_PROGRAM).unwrap();
        let fragment_shader = compile_shader(&gl, WebGlRenderingContext::FRAGMENT_SHADER, FRAGMENT_SHADER_PROGRAM).unwrap();
        let program = link_program(&gl, &vertex_shader, &fragment_shader).unwrap();

        let renderer = WebRenderer::new(&gl, program);

        WebClient { gl, renderer }
    }

    pub fn start(&self) -> Result<(), JsValue> {
        Ok(())
    }

    pub fn update(&self, _dt: f32) {
    }

    pub fn render(&mut self) {
        self.renderer.render(&self.gl);
    }
}

fn compile_shader(
    gl: &WebGlRenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, String> {
    let shader = gl
        .create_shader(shader_type)
        .ok_or_else(|| "Could not create shader".to_string())?;
    gl.shader_source(&shader, source);
    gl.compile_shader(&shader);

    if gl
        .get_shader_parameter(&shader, WebGlRenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(gl
            .get_shader_info_log(&shader)
            .unwrap_or_else(|| "Unknown error creating shader".to_string()))
    }
}

fn link_program(
    gl: &WebGlRenderingContext,
    vert_shader: &WebGlShader,
    frag_shader: &WebGlShader,
) -> Result<WebGlProgram, String> {
    let program = gl
        .create_program()
        .ok_or_else(|| "Unable to create shader program".to_string())?;

    gl.attach_shader(&program, &vert_shader);
    gl.attach_shader(&program, &frag_shader);

    gl.link_program(&program);

    if gl
        .get_program_parameter(&program, WebGlRenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        Err(gl
            .get_program_info_log(&program)
            .unwrap_or_else(|| "Unknown error creating program".to_string()))
    }
}
