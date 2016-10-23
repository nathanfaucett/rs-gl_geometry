extern crate gl;
extern crate glutin;
extern crate gl_context;

extern crate geometry;
extern crate gl_geometry;


use gl_context::{Context, Depth};

use geometry::{Geometry, Attribute};
use gl_geometry::GLGeometry;


static VERTEX_DATA: [f32; 12] = [
    0.5f32, 0.5f32, 0f32,
    -0.5f32, 0.5f32, 0f32,
    0.5f32, -0.5f32, 0f32,
    -0.5f32, -0.5f32, 0f32
];

static UV_DATA: [f32; 8] = [
    1f32, 1f32,
    0f32, 1f32,
    1f32, 0f32,
    0f32, 0f32
];

static VS_SRC: &'static str = "
    #version 140

    in vec3 position;
    in vec2 uv;

    varying vec2 v_uv;

    void main() {
        v_uv = uv;
        gl_Position = vec4(position, 1.0);
    }
";
static FS_SRC: &'static str = "
    #version 140

    out vec4 out_color;

    varying vec2 v_uv;

    void main() {
        out_color = vec4(v_uv, 1.0, 1.0);
    }
";

fn main() {
    let window = glutin::Window::new().unwrap();
    let mut context = Context::new();

    unsafe {
        match window.make_current() {
            Ok(_) => {
                gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);
            },
            Err(e) => panic!("{:?}", e),
        }
    }

    context.init();
    context.set_depth_func(Depth::Always);

    println!(
        "OpenGL version: {:?}.{:?}, GLSL version {:?}.{:?}0",
        context.get_major(), context.get_minor(), context.get_glsl_major(), context.get_glsl_minor()
    );

    let mut geometry = Geometry::new();
    geometry.add_attribute(Attribute::new_f32("position", Box::new(VERTEX_DATA), 3, false));
    geometry.add_attribute(Attribute::new_f32("uv", Box::new(UV_DATA), 2, false));

    let mut gl_geometry = GLGeometry::new(&context, geometry);

    let mut program = context.new_program();
    program.set(VS_SRC, FS_SRC);
    context.set_program(&program, false);

    let vertex_buffer = gl_geometry.get_vertex_buffer(&mut context, false);
    program.set_attribute("position", &mut context, vertex_buffer, 0, false);
    program.set_attribute("uv", &mut context, vertex_buffer, 3, false);

    let mut playing = true;
    while playing {
        for event in window.poll_events() {
            match event {
                glutin::Event::Closed => {
                    playing = false;
                },
                glutin::Event::Resized(w, h) => {
                    context.set_viewport(0, 0, w as usize, h as usize);
                },
                _ => (),
            }
        }

        context.clear(true, true, true);
        context.set_clear_color(&[0.3, 0.3, 0.3, 1.0]);

        unsafe { gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4); }

        match window.swap_buffers() {
            Ok(_) => (),
            Err(e) => panic!("{:?}", e),
        }
    }
}
