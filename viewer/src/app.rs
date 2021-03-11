use std::cell::RefCell;
use std::rc::Rc;

use js_sys::WebAssembly;
use wasm_bindgen::JsCast;
use web_sys::{self, WebGl2RenderingContext as GL};
use nalgebra::{Matrix4, Perspective3, Vector3};
use tracing::debug;

use crate::render::LineGraph;
use crate::store::*;
use crate::shader::{ShaderSystem, ShaderKind};

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

        gl.clear_color(0.0, 0.0, 0.0, 1.0);
        gl.clear_depth(1.0);
        gl.clear(GL::COLOR_BUFFER_BIT);
        gl.enable(GL::DEPTH_TEST);
        gl.depth_func(GL::LEQUAL);

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

        debug!("data_array = {:?}", &data_array);

        let shader = self.shaders.get(&ShaderKind::SolidWhite).unwrap();
        self.shaders.use_program(gl, ShaderKind::SolidWhite);

        let vertex_position = gl.get_attrib_location(&shader.program, "aVertexPosition");

        let uniform_project_matrix = gl.get_uniform_location(&shader.program, "uProjectionMatrix");
        let uniform_model_view_matrix = gl.get_uniform_location(&shader.program, "uModelViewMatrix");

        let field_of_view = 45. * std::f32::consts::PI / 180.;
        let aspect = state.canvas_dimensions.width as f32 / state.canvas_dimensions.height as f32;
        let z_near = 0.1;
        let z_far = 100.0;

        let projection = Perspective3::new(aspect, field_of_view, z_near, z_far);
        let projection_matrix = projection.as_matrix().to_owned();

        let view_matrix = Matrix4::new_translation(&Vector3::new(-0., 0., -6.));
        debug!("view_matrix {:#?}", view_matrix);

        {
            let num_components = 2;  // pull out 2 values per iteration
            let normalize = false;  // don't normalize
            let stride = 0;         // how many bytes to get from one set of values to the next
            // 0 = use type and numComponents above
            let offset = 0;         // how many bytes inside the buffer to start from

            gl.bind_buffer(GL::ARRAY_BUFFER, Some(&buffer));
            gl.vertex_attrib_pointer_with_i32(
                vertex_position as u32,
                num_components,
                GL::FLOAT,
                normalize,
                stride,
                offset);
            gl.enable_vertex_attrib_array(
                vertex_position as u32);
        }

        // Set the shader uniforms
        let mut projection_matrix_data = [0.0_f32; 16];
        projection_matrix_data.copy_from_slice(projection_matrix.as_slice());

        debug!("projection_matrix_data {:#?}", projection_matrix_data);

        gl.uniform_matrix4fv_with_f32_array(
            uniform_project_matrix.as_ref(),
            false,
            &mut projection_matrix_data);

        let mut view_matrix_data = [0.0_f32; 16];
        view_matrix_data.copy_from_slice(view_matrix.as_slice());

        gl.uniform_matrix4fv_with_f32_array(
            uniform_model_view_matrix.as_ref(),
            false,
            &mut view_matrix_data);

        {
            let offset = 0;
            let vertex_count = 4;
            gl.draw_arrays(GL::TRIANGLE_STRIP, offset, vertex_count);
        }

        //self.lg.render(gl, &self.shaders);
    }
}
