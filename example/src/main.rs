extern crate gl;
extern crate glutin;
extern crate gl_context;

#[macro_use]
extern crate vector;

extern crate geometry;
extern crate gl_geometry;


use std::mem;

use gl_context::Context;

use geometry::{Geometry, Attribute};
use gl_geometry::GLGeometry;


static VS_SRC: &'static str = "
    #version 140

    in vec3 position;
    in vec3 normal;
    in vec2 uv;

    varying vec3 v_normal;
    varying vec2 v_uv;

    void main() {
        v_normal = normal;
        v_uv = uv;
        gl_Position = vec4(position, 1.0);
    }
";
static FS_SRC: &'static str = "
    #version 140

    out vec4 out_color;

    varying vec3 v_normal;
    varying vec2 v_uv;

    void main() {
        out_color = vec4(v_uv, v_normal.z, 1.0);
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

    println!(
        "OpenGL version: {:?}.{:?}, GLSL version {:?}.{:?}0",
        context.get_major(), context.get_minor(), context.get_glsl_major(), context.get_glsl_minor()
    );

    let mut geometry = Geometry::new();
    geometry.add_attribute(Attribute::new_f32("position", vector![
        -0.5, -0.5, 0.0,
        -0.5, 0.5, 0.0,
        0.5, 0.5, 0.0,
        0.5, -0.5, 0.0
    ], 3, false));
    geometry.add_attribute(Attribute::new_f32("normal", vector![
        0.0, 0.0, 1.0,
        0.0, 0.0, 1.0,
        0.0, 0.0, 1.0,
        0.0, 0.0, 1.0
    ], 3, false));
    geometry.add_attribute(Attribute::new_f32("uv", vector![
        0.0, 0.0,
        0.0, 1.0,
        1.0, 1.0,
        1.0, 0.0
    ], 2, false));
    geometry.set_index(Attribute::new_u32("index", vector![
        0, 2, 1, 0, 3, 2
    ], 1, false));

    let mut gl_geometry = GLGeometry::new(&context, geometry);

    let mut program = context.new_program();
    program.set(VS_SRC, FS_SRC);
    context.set_program(&program, false);

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

        {
            let mut tmp = gl_geometry.clone();
            let vertex_buffer = tmp.get_vertex_buffer(&mut context, false);

            for (name, attribute) in program.get_attributes_mut() {
                attribute.set(&mut context, vertex_buffer, gl_geometry.get_offset(name), false);
            }
        }

        let index_buffer = gl_geometry.get_index_buffer(&mut context, false);
        context.set_buffer(&index_buffer, false);

        unsafe {
            gl::DrawElements(
                gl::TRIANGLES,
                index_buffer.get_length() as i32,
                gl::UNSIGNED_INT,
                mem::transmute(0usize)
            );
        }

        match window.swap_buffers() {
            Ok(_) => (),
            Err(e) => panic!("{:?}", e),
        }
    }
}
