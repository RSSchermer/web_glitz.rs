use std::rc::Rc;
use std::sync::Arc;

use wasm_bindgen::JsCast;

use crate::runtime::Connection;
use crate::task::{GpuTask, Progress};
use crate::util::JsId;

pub(crate) trait Dropper {
    fn drop_gl_object(&self, object: DropObject);
}

pub(crate) enum RefCountedDropper {
    Rc(Rc<Dropper>),
    Arc(Arc<Dropper>),
}

impl Dropper for RefCountedDropper {
    fn drop_gl_object(&self, object: DropObject) {
        match self {
            RefCountedDropper::Rc(dropper) => dropper.drop_gl_object(object),
            RefCountedDropper::Arc(dropper) => dropper.drop_gl_object(object),
        }
    }
}

pub(crate) enum DropObject {
    Buffer(JsId),
    Framebuffer(JsId),
    Program(JsId),
    Renderbuffer(JsId),
    Texture(JsId),
    Shader(JsId),
    VertexArray(JsId),
}

pub(crate) struct DropTask {
    object: DropObject,
}

impl DropTask {
    pub(crate) fn new(object: DropObject) -> Self {
        DropTask { object }
    }
}

impl GpuTask<Connection> for DropTask {
    type Output = ();

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output> {
        let Connection(gl, _) = connection;

        match self.object {
            DropObject::Buffer(id) => unsafe {
                gl.delete_buffer(Some(&JsId::into_value(id).unchecked_into()));
            },
            DropObject::Framebuffer(id) => unsafe {
                gl.delete_framebuffer(Some(&JsId::into_value(id).unchecked_into()));
            },
            DropObject::Program(id) => unsafe {
                gl.delete_program(Some(&JsId::into_value(id).unchecked_into()));
            },
            DropObject::Renderbuffer(id) => unsafe {
                gl.delete_renderbuffer(Some(&JsId::into_value(id).unchecked_into()));
            },
            DropObject::Texture(id) => unsafe {
                gl.delete_texture(Some(&JsId::into_value(id).unchecked_into()));
            },
            DropObject::Shader(id) => unsafe {
                gl.delete_shader(Some(&JsId::into_value(id).unchecked_into()));
            },
            DropObject::VertexArray(id) => unsafe {
                gl.delete_vertex_array(Some(&JsId::into_value(id).unchecked_into()));
            },
        }

        Progress::Finished(())
    }
}
