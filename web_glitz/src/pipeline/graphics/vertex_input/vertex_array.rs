use std::sync::Arc;
use std::{marker, mem};

use wasm_bindgen::JsCast;

use web_sys::WebGl2RenderingContext as Gl;

use crate::buffer::{Buffer, BufferData, BufferView};
use crate::pipeline::graphics::vertex_input::{
    InputRate, Vertex, VertexBufferDescription, VertexBufferDescriptor,
    VertexInputAttributeDescriptor,
};
use crate::runtime::state::ContextUpdate;
use crate::runtime::{Connection, RenderingContext};
use crate::task::{ContextId, GpuTask, Progress};
use crate::util::{arc_get_mut_unchecked, JsId};



