use collections::string::String;

use vector::Vector;
use stack::Stack;
use collection::Collection;

use hash_map::HashMap;
use insert::Insert;

use shared::Shared;
use num::Num;

use gl;
use gl_context::{Context, Buffer, VertexArray};
use geometry::{Geometry, AttributeValue};


#[derive(Clone, Debug)]
pub struct BufferData {
    pub name: String,
    pub offset: usize,
}

impl BufferData {
    pub fn new(name: &str, offset: usize) -> Self {
        BufferData {
            name: String::from(name),
            offset: offset,
        }
    }
}


pub struct GLGeometryData {
    geometry: Geometry,

    buffer_data: HashMap<String, BufferData>,

    gl_vertex_array: VertexArray,
    gl_vertex_buffer: Buffer,
    gl_index_buffer: Buffer,
    gl_index_line_buffer: Buffer,

    vertex_needs_compile: bool,
    index_needs_compile: bool,
    index_line_needs_compile: bool,
}

#[derive(Clone)]
pub struct GLGeometry {
    data: Shared<GLGeometryData>,
}

impl GLGeometry {
    pub fn new(context: &Context, geometry: Geometry) -> Self {
        GLGeometry {
            data: Shared::new(GLGeometryData {
                geometry: geometry,

                buffer_data: HashMap::new(),

                gl_vertex_array: context.new_vertex_array(),
                gl_vertex_buffer: context.new_buffer(),
                gl_index_buffer: context.new_buffer(),
                gl_index_line_buffer: context.new_buffer(),

                vertex_needs_compile: true,
                index_needs_compile: true,
                index_line_needs_compile: true,
            })
        }
    }

    pub fn get_offset(&self, name: &str) -> usize {
        self.data.buffer_data.get(name).unwrap().offset
    }

    pub fn get_index_buffer(&mut self, context: &mut Context, force: bool) -> &Buffer {

        context.set_vertex_array(&self.data.gl_vertex_array, force);

        if force || self.data.index_needs_compile {
            self.compile_index_buffer(context)
        } else {
            &self.data.gl_index_buffer
        }
    }
    fn compile_index_buffer(&mut self, _context: &mut Context) -> &Buffer {
        let index_array = if let Some(index) = self.data.geometry.get_index() {
            Self::cast_to_u32_array(&index.value)
        } else {
            Vector::new()
        };

        self.data.index_needs_compile = false;
        self.data.gl_index_buffer.set(gl::ELEMENT_ARRAY_BUFFER, &index_array, 0, gl::STATIC_DRAW);

        &self.data.gl_index_buffer
    }

    pub fn get_line_index_buffer(&mut self, context: &mut Context, force: bool) -> &Buffer {

        context.set_vertex_array(&self.data.gl_vertex_array, force);

        if force || self.data.index_line_needs_compile {
            self.compile_line_index_buffer(context)
        } else {
            &self.data.gl_index_line_buffer
        }
    }
    fn compile_line_index_buffer(&mut self, _context: &mut Context) -> &Buffer {
        let line_index = if let Some(index) = self.data.geometry.get_index() {
            Self::cast_to_u32_line_array(&index.value)
        } else {
            Vector::new()
        };

        self.data.index_line_needs_compile = false;
        self.data.gl_index_line_buffer.set(gl::ELEMENT_ARRAY_BUFFER, &line_index, 0, gl::STATIC_DRAW);

        &self.data.gl_index_line_buffer
    }
    fn cast_to_u32_line_array<'a>(value: &'a AttributeValue) -> Vector<u32> {
        match value {
            &AttributeValue::F32(ref v) => Self::to_u32_line_array::<f32>(v),
            &AttributeValue::F64(ref v) => Self::to_u32_line_array::<f64>(v),

            &AttributeValue::U8(ref v) => Self::to_u32_line_array::<u8>(v),
            &AttributeValue::U16(ref v) => Self::to_u32_line_array::<u16>(v),
            &AttributeValue::U32(ref v) => Self::to_u32_line_array::<u32>(v),
            &AttributeValue::U64(ref v) => Self::to_u32_line_array::<u64>(v),

            &AttributeValue::I8(ref v) => Self::to_u32_line_array::<i8>(v),
            &AttributeValue::I16(ref v) => Self::to_u32_line_array::<i16>(v),
            &AttributeValue::I32(ref v) => Self::to_u32_line_array::<i32>(v),
            &AttributeValue::I64(ref v) => Self::to_u32_line_array::<i64>(v),
        }
    }
    fn to_u32_line_array<'a, T: Num>(index_array: &'a Vector<T>) -> Vector<u32> {
        let length = index_array.len();

        let mut line_buffer = Vector::with_capacity(length * 2);
        unsafe {line_buffer.set_len(length * 2)}

        let mut i = 0;
        let mut index = 0;
        let mut triangle_index = 0;

        while i < length {
            line_buffer[index] = index_array[triangle_index].to_u32();
            line_buffer[index + 1] = index_array[triangle_index + 1].to_u32();

            line_buffer[index + 2] = index_array[triangle_index + 1].to_u32();
            line_buffer[index + 3] = index_array[triangle_index + 2].to_u32();

            line_buffer[index + 4] = index_array[triangle_index + 2].to_u32();
            line_buffer[index + 5] = index_array[triangle_index].to_u32();

            triangle_index += 3;
            index += 6;
            i += 3;
        }

        line_buffer
    }

    pub fn get_vertex_buffer(&mut self, context: &mut Context, force: bool) -> &Buffer {

        context.set_vertex_array(&self.data.gl_vertex_array, force);

        if force || self.data.vertex_needs_compile {
            self.compile_vertex_buffer(context)
        } else {
            &self.data.gl_vertex_buffer
        }
    }
    fn compile_vertex_buffer(&mut self, _context: &mut Context) -> &Buffer {
        let mut vertex_length = 0;
        let mut stride = 0;

        for (_, attribute) in self.data.geometry.get_attributes() {
            vertex_length += attribute.len();
            stride += attribute.item_size;
        }

        let mut vertex_array = Vector::with_capacity(vertex_length);
        unsafe {
            vertex_array.set_len(vertex_length);
        }

        let mut last = 0;
        let mut offset = 0;

        let mut buffer_data = Vector::new();

        for (_, attribute) in self.data.geometry.get_attributes() {
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

            buffer_data.push(BufferData::new(&attribute.name, offset));
        }

        while !buffer_data.is_empty() {
            let data = buffer_data.pop().unwrap();
            self.data.buffer_data.insert(data.name.clone(), data);
        }

        self.data.gl_vertex_buffer.set(gl::ARRAY_BUFFER, &vertex_array, stride, gl::STATIC_DRAW);
        self.data.vertex_needs_compile = false;

        return &self.data.gl_vertex_buffer;
    }

    fn cast_to_u32_array<'a>(value: &'a AttributeValue) -> Vector<u32> {
        match value {
            &AttributeValue::F32(ref v) => Self::to_u32_array::<f32>(v),
            &AttributeValue::F64(ref v) => Self::to_u32_array::<f64>(v),

            &AttributeValue::U8(ref v) => Self::to_u32_array::<u8>(v),
            &AttributeValue::U16(ref v) => Self::to_u32_array::<u16>(v),
            &AttributeValue::U32(ref v) => v.clone(),
            &AttributeValue::U64(ref v) => Self::to_u32_array::<u64>(v),

            &AttributeValue::I8(ref v) => Self::to_u32_array::<i8>(v),
            &AttributeValue::I16(ref v) => Self::to_u32_array::<i16>(v),
            &AttributeValue::I32(ref v) => Self::to_u32_array::<i32>(v),
            &AttributeValue::I64(ref v) => Self::to_u32_array::<i64>(v),
        }
    }
    fn cast_to_f32_array<'a>(value: &'a AttributeValue) -> Vector<f32> {
        match value {
            &AttributeValue::F32(ref v) => v.clone(),
            &AttributeValue::F64(ref v) => Self::to_f32_array::<f64>(v),

            &AttributeValue::U8(ref v) => Self::to_f32_array::<u8>(v),
            &AttributeValue::U16(ref v) => Self::to_f32_array::<u16>(v),
            &AttributeValue::U32(ref v) => Self::to_f32_array::<u32>(v),
            &AttributeValue::U64(ref v) => Self::to_f32_array::<u64>(v),

            &AttributeValue::I8(ref v) => Self::to_f32_array::<i8>(v),
            &AttributeValue::I16(ref v) => Self::to_f32_array::<i16>(v),
            &AttributeValue::I32(ref v) => Self::to_f32_array::<i32>(v),
            &AttributeValue::I64(ref v) => Self::to_f32_array::<i64>(v),
        }
    }

    fn to_u32_array<'a, T: Num>(values: &'a Vector<T>) -> Vector<u32> {
        let mut out: Vector<u32> = Vector::with_capacity(values.len());
        for x in values.iter() {
            out.push(x.to_u32());
        }
        out
    }
    fn to_f32_array<'a, T: Num>(values: &'a Vector<T>) -> Vector<f32> {
        let mut out: Vector<f32> = Vector::with_capacity(values.len());
        for x in values.iter() {
            out.push(x.to_f32());
        }
        out
    }
}
