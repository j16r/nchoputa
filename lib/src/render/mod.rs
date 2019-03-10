use self::framebuffer::*;
use crate::app::State;
use crate::canvas::{CANVAS_HEIGHT, CANVAS_WIDTH};
use web_sys::WebGlRenderingContext as GL;
use web_sys::*;

pub static WATER_TILE_Y_POS: f32 = 0.0;

mod framebuffer;
mod render_trait;

struct Vao(js_sys::Object);

pub struct WebRenderer {
    #[allow(unused)]
    depth_texture_ext: Option<js_sys::Object>,
    refraction_framebuffer: Framebuffer,
    reflection_framebuffer: Framebuffer,
}

impl WebRenderer {
    pub fn new(gl: &WebGlRenderingContext) -> WebRenderer {
        let depth_texture_ext = gl
            .get_extension("WEBGL_depth_texture")
            .expect("Depth texture extension");

        let oes_vao_ext = gl
            .get_extension("OES_vertex_array_object")
            .expect("Get OES vao ext")
            .expect("OES vao ext");

        let refraction_framebuffer = WebRenderer::create_refraction_framebuffer(&gl).unwrap();
        let reflection_framebuffer = WebRenderer::create_reflection_framebuffer(&gl).unwrap();

        WebRenderer {
            depth_texture_ext,
            refraction_framebuffer,
            reflection_framebuffer,
        }
    }

    pub fn render(&mut self, gl: &WebGlRenderingContext, state: &State) {
        gl.clear_color(0.53, 0.8, 0.98, 1.);
        gl.clear(GL::COLOR_BUFFER_BIT | GL::DEPTH_BUFFER_BIT);

        let above = 1000000.0;
        // Position is positive instead of negative for.. mathematical reasons..
        let clip_plane = [0., 1., 0., above];

        //self.render_refraction_fbo(gl, state, assets);
        //self.render_reflection_fbo(gl, state, assets);

        gl.viewport(0, 0, CANVAS_WIDTH, CANVAS_HEIGHT);

        //self.render_water(gl, state);
        //self.render_meshes(gl, state, assets, clip_plane, false);

        //self.render_refraction_visual(gl, state);
        //self.render_reflection_visual(gl, state);
    }
}
