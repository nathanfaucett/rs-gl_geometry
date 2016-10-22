#![feature(alloc)]
#![feature(collections)]
#![no_std]


extern crate alloc;
extern crate collections;

extern crate gl;
extern crate shared;
extern crate gl_context;
extern crate vector;
extern crate hash_map;
extern crate insert;
extern crate geometry;


mod gl_geometry;


pub use gl_geometry::GLGeometry;
