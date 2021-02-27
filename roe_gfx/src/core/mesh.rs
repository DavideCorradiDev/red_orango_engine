use ::core::ops::Range;
use std::marker::PhantomData;

use super::{Buffer, BufferInitDescriptor, BufferUsage, Instance};

pub type MeshVertexRange = Range<u32>;
pub type MeshIndexRange = Range<u32>;
pub type MeshIndex = u16;

#[derive(Debug)]
struct TypedBuffer<T: bytemuck::Pod> {
    buffer: Buffer,
    element_count: u32,
    _p: PhantomData<T>,
}

impl<T: bytemuck::Pod> TypedBuffer<T> {
    pub fn new(instance: &Instance, vertex_list: &[T], usage: BufferUsage) -> Self {
        let buffer = Buffer::init(
            &instance,
            &BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(vertex_list),
                usage,
            },
        );
        let element_count = vertex_list.len() as u32;
        Self {
            buffer,
            element_count,
            _p: PhantomData,
        }
    }
}

#[derive(Debug)]
pub struct Mesh<V: bytemuck::Pod> {
    vertex_buffer: TypedBuffer<V>,
}

impl<V: bytemuck::Pod> Mesh<V> {
    pub fn new(instance: &Instance, vertex_list: &[V]) -> Self {
        let vertex_buffer = TypedBuffer::new(instance, vertex_list, BufferUsage::VERTEX);
        Self { vertex_buffer }
    }

    pub fn vertex_buffer(&self) -> &Buffer {
        &self.vertex_buffer.buffer
    }

    pub fn vertex_count(&self) -> u32 {
        self.vertex_buffer.element_count
    }
}

#[derive(Debug)]
pub struct IndexedMesh<V: bytemuck::Pod> {
    vertex_buffer: TypedBuffer<V>,
    index_buffer: TypedBuffer<MeshIndex>,
}

impl<V: bytemuck::Pod> IndexedMesh<V> {
    pub fn new(instance: &Instance, vertex_list: &[V], index_list: &[MeshIndex]) -> Self {
        let vertex_buffer = TypedBuffer::new(instance, vertex_list, BufferUsage::VERTEX);
        let index_buffer = TypedBuffer::new(instance, index_list, BufferUsage::INDEX);
        Self {
            vertex_buffer,
            index_buffer,
        }
    }

    pub fn vertex_buffer(&self) -> &Buffer {
        &self.vertex_buffer.buffer
    }

    pub fn vertex_count(&self) -> u32 {
        self.vertex_buffer.element_count
    }

    pub fn index_buffer(&self) -> &Buffer {
        &self.index_buffer.buffer
    }

    pub fn index_count(&self) -> u32 {
        self.index_buffer.element_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use galvanic_assert::{matchers::*, *};

    use crate::core::InstanceDescriptor;

    #[derive(Debug, PartialEq, Clone, Copy)]
    struct Vertex {
        pos: [f32; 2],
    }

    unsafe impl bytemuck::Zeroable for Vertex {
        fn zeroed() -> Self {
            Self { pos: [0., 0.] }
        }
    }

    unsafe impl bytemuck::Pod for Vertex {}

    #[test]
    #[serial_test::serial]
    fn mesh_creation() {
        let instance = Instance::new(&InstanceDescriptor::default()).unwrap();
        let mesh = Mesh::<Vertex>::new(
            &instance,
            &[
                Vertex { pos: [1., 2.] },
                Vertex { pos: [3., 4.] },
                Vertex { pos: [3., 4.] },
                Vertex { pos: [5., 6.] },
            ],
        );
        expect_that!(&mesh.vertex_count(), eq(4));
    }

    #[test]
    #[serial_test::serial]
    fn indexed_mesh_creation() {
        let instance = Instance::new(&InstanceDescriptor::default()).unwrap();
        let mesh = IndexedMesh::<Vertex>::new(
            &instance,
            &[
                Vertex { pos: [1., 2.] },
                Vertex { pos: [3., 4.] },
                Vertex { pos: [5., 6.] },
            ],
            &[0, 1, 1, 2],
        );
        expect_that!(&mesh.vertex_count(), eq(3));
        expect_that!(&mesh.index_count(), eq(4));
    }
}
