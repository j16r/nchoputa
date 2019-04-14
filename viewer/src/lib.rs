extern crate wasm_bindgen;
use console_error_panic_hook;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::*;
use web_sys::*;
use js_sys::WebAssembly;
use nalgebra::{Matrix4, Perspective3};
use web_sys::WebGlRenderingContext as GL;

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


#[wasm_bindgen]
impl WebClient {
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

pub struct WebRenderer {
    shader: WebGlProgram,
    buffer: WebGlBuffer,
    vertex_position: i32,
    uniform_project_matrix: Option<WebGlUniformLocation>,
    uniform_model_view_matrix: Option<WebGlUniformLocation>
}

impl WebRenderer {
    pub fn new(gl: &WebGlRenderingContext, program: WebGlProgram) -> WebRenderer {

        // Create the shape (a square)
        let buffer = gl.create_buffer().unwrap();
        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&buffer));
        let positions : [f32; 8] = [
            -1.0,  1.0,
            1.0,  1.0,
            -1.0, -1.0,
            1.0, -1.0,
        ];
        
        let memory_buffer = wasm_bindgen::memory()
            .dyn_into::<WebAssembly::Memory>()
            .unwrap()
            .buffer();

        let data_location = positions.as_ptr() as u32 / 4;

        let data_array = js_sys::Float32Array::new(&memory_buffer)
            .subarray(data_location, data_location + positions.len() as u32);

        gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &data_array, GL::STATIC_DRAW);

        console::log_1(&data_array);

        let vertex_position = gl.get_attrib_location(&program, "aVertexPosition");

        let uniform_project_matrix = gl.get_uniform_location(&program, "uProjectionMatrix");
        let uniform_model_view_matrix = gl.get_uniform_location(&program, "uModelViewMatrix");

        WebRenderer {
            shader: program,
            buffer,
            vertex_position,
            uniform_project_matrix,
            uniform_model_view_matrix,
        }
    }

    pub fn render(&mut self, gl: &WebGlRenderingContext) {
        gl.clear_color(0.0, 0.0, 0.0, 1.);
        gl.clear_depth(1.);
        gl.enable(GL::DEPTH_TEST);
        gl.depth_func(GL::LEQUAL);

        // Clear the canvas

        gl.clear(GL::COLOR_BUFFER_BIT | GL::DEPTH_BUFFER_BIT);

        let field_of_view = 45. * std::f32::consts::PI / 180.;   // in radians
        let aspect = 1.; // 1600. / 1200.; // gl.canvas.clientWidth / gl.canvas.clientHeight;
        let z_near = 0.1;
        let z_far = 100.0;

        let projection = Perspective3::new(aspect, field_of_view, z_near, z_far);
        let projection_matrix = projection.as_matrix().to_owned();

        //console::log_1(&format!("projection_matrix {:#?}", projection_matrix).into());

        // TODO: How to do the translate with the vector below to produce the view_matrix below
        //let mut view_matrix = Matrix4::repeat(0.);
        //view_matrix += Translation3::new(-0., 0., -6.);

        let view_matrix = Matrix4::new(
            1., 0., 0., 0.,
            0., 1., 0., 0.,
            0., 0., 1., 0.,
            0., 0., -6., 1.);

        // Tell WebGL how to pull out the positions from the position
        // buffer into the vertexPosition attribute.

        {
            let num_components = 2;  // pull out 2 values per iteration
            let normalize = false;  // don't normalize
            let stride = 0;         // how many bytes to get from one set of values to the next
            // 0 = use type and numComponents above
            let offset = 0;         // how many bytes inside the buffer to start from

            gl.bind_buffer(GL::ARRAY_BUFFER, Some(&self.buffer));
            gl.vertex_attrib_pointer_with_i32(
                self.vertex_position as u32,
                num_components,
                GL::FLOAT,
                normalize,
                stride,
                offset);
            gl.enable_vertex_attrib_array(
                self.vertex_position as u32);
        }

        // Tell WebGL to use our program when drawing

        gl.use_program(Some(&self.shader));

        // Set the shader uniforms

        let mut projection_matrix_data = [0.0_f32; 16];
        projection_matrix_data.copy_from_slice(projection_matrix.as_slice());

        console::log_1(&format!("projection_matrix_data {:#?}", projection_matrix_data).into());

        gl.uniform_matrix4fv_with_f32_array(
            self.uniform_project_matrix.as_ref(),
            false,
            &mut projection_matrix_data);

        let mut view_matrix_data = [0.0_f32; 16];
        view_matrix_data.copy_from_slice(view_matrix.as_slice());

        gl.uniform_matrix4fv_with_f32_array(
            self.uniform_model_view_matrix.as_ref(),
            false,
            &mut view_matrix_data);

        {
            let offset = 0;
            let vertex_count = 4;
            gl.draw_arrays(GL::TRIANGLE_STRIP, offset, vertex_count);
        }

        panic!();
    }
}
