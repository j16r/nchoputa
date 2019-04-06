use crate::app::State;
use js_sys::WebAssembly;
use nalgebra::{Matrix4, Perspective3};
use wasm_bindgen::JsCast;
use web_sys::*;
use web_sys::WebGlRenderingContext as GL;
//use web_sys::console;

mod render_trait;

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

        console::log_1(&data_array);

        gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &data_array, GL::STATIC_DRAW);
        gl.vertex_attrib_pointer_with_i32(0, 4, GL::FLOAT, false, 0, 0);
        gl.enable_vertex_attrib_array(0);

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

    pub fn render(&mut self, gl: &WebGlRenderingContext, _state: &State) {
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
        let mut projection_matrix = projection.as_matrix().to_owned();

        // TODO: How to do the translate with the vector below to produce the view_matrix below
        //let mut view_matrix = Matrix4::repeat(0.);
        //view_matrix += Translation3::new(-0., 0., -6.);

        let mut view_matrix = Matrix4::new(
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

        let mut projection_matrix_data = [0.; 16];
        projection_matrix_data.copy_from_slice(projection_matrix.as_slice());

        gl.uniform_matrix4fv_with_f32_array(
            self.uniform_project_matrix.as_ref(),
            false,
            &mut projection_matrix_data);

        let mut view_matrix_data = [0.; 16];
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
