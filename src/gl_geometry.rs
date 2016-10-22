use collections::string::String;

use vector::Vector;
use hash_map::HashMap;
use insert::Insert;

use gl;
use gl_context::{Context, Buffer, VertexArray};
use geometry::{Geometry, AttributeValue};


pub struct BufferData {
    pub name: String,
    pub offset: usize,
}

impl BufferData {
    fn new(name: &str, offset: usize) -> Self {
        BufferData {
            name: String::from(name),
            offset: offset,
        }
    }
}


pub struct GLGeometry<'a> {
    pub geometry: Geometry<'a>,

    pub buffer_data: HashMap<String, BufferData>,

    pub gl_vertex_array: VertexArray,
    pub gl_vertex_buffer: Buffer,
    pub gl_index_buffer: Buffer,
    pub gl_index_line_buffer: Buffer,

    pub vertex_needs_compile: bool,
    pub index_needs_compile: bool,
    pub index_line_needs_compile: bool,
}

impl<'a> GLGeometry<'a> {
    pub fn new(context: &Context, geometry: Geometry<'a>) -> Self {
        GLGeometry {
            geometry: geometry,

            buffer_data: HashMap::new(),

            gl_vertex_array: context.new_vertex_array(),
            gl_vertex_buffer: context.new_buffer(),
            gl_index_buffer: context.new_buffer(),
            gl_index_line_buffer: context.new_buffer(),

            vertex_needs_compile: true,
            index_needs_compile: true,
            index_line_needs_compile: true,
        }
    }

    pub fn get_vertex_buffer(&mut self, context: &mut Context, force: bool) -> &Buffer {

        context.set_vertex_array(&self.gl_vertex_array, force);

        if force || self.vertex_needs_compile {
            self.compile_vertex_buffer(context)
        } else {
            &self.gl_vertex_buffer
        }
    }

    fn compile_vertex_buffer(&mut self, _context: &mut Context) -> &Buffer {
        let attributes = self.geometry.get_attributes();

        let ref mut buffer_data = self.buffer_data;
        buffer_data.clear();

        let mut vertex_length = 0;
        let mut stride = 0;

        for (_, attribute) in attributes {
            vertex_length += attribute.len();
            stride += attribute.item_size;
        }

        let mut vertex_array = Vector::with_capacity(vertex_length);
        unsafe {
            vertex_array.set_len(vertex_length);
        }

        let mut last = 0;
        let mut offset = 0;

        for (_, attribute) in attributes {
            let attribute_array = Self::cast_to_f32_array(&attribute.value);

            let item_size = attribute.item_size;
            let mut index = 0;

            offset += last;
            last = item_size;

            let mut j = 0;
            while j < vertex_length {

                for k in 0..item_size {
                    vertex_array[offset + j + k] = attribute_array[index + k];
                }

                j += stride;
                index += item_size;
            }

            buffer_data.insert(attribute.name.clone(), BufferData::new(&attribute.name, offset));
        }

        self.gl_vertex_buffer.set(gl::ARRAY_BUFFER, &vertex_array, stride, gl::STATIC_DRAW);
        self.vertex_needs_compile = false;

        return &self.gl_vertex_buffer;
    }

    fn cast_to_f32_array(value: &AttributeValue<'a>) -> &'a [f32] {
        match value {
            &AttributeValue::F32(v) => v,
            _ => panic!("invalid attribute value"),
        }
    }
}
