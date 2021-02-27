use js_sys::WebAssembly;
use tracing::trace;
use wasm_bindgen::JsCast;
use web_sys::{self, WebGlRenderingContext};

pub struct LineGraph {
    vertices: js_sys::Float32Array,
    count: usize,
}

impl LineGraph {
    pub fn new() -> LineGraph {
        let vertices: [f32; 9] = [-1.0, -1.0, 0.0, 1.0, -1.0, 0.0, 0.0, 1.0, 0.0];
        let memory_buffer = wasm_bindgen::memory()
            .dyn_into::<WebAssembly::Memory>()
            .unwrap()
            .buffer();
        let vertices_location = vertices.as_ptr() as u32 / 4;
        let vert_array = js_sys::Float32Array::new(&memory_buffer)
            .subarray(vertices_location, vertices_location + vertices.len() as u32);

        LineGraph {
            vertices: vert_array,
            count: vertices.len(),
        }
    }

    pub fn render(&self, gl: &WebGlRenderingContext) {
        let buffer = gl.create_buffer().expect("failed to create buffer");
        gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&buffer));
        gl.buffer_data_with_array_buffer_view(
            WebGlRenderingContext::ARRAY_BUFFER,
            &self.vertices,
            WebGlRenderingContext::STATIC_DRAW,
        );
        gl.vertex_attrib_pointer_with_i32(0, 3, WebGlRenderingContext::FLOAT, false, 0, 0);

        gl.draw_arrays(WebGlRenderingContext::TRIANGLES, 0, (self.count / 3) as i32);
        gl.enable_vertex_attrib_array(0);
    }
}
