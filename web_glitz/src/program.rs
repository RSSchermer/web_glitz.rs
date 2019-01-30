use std::borrow::Borrow;
use std::marker;
use std::mem;
use std::slice;
use std::sync::Arc;

use wasm_bindgen::JsCast;
use web_sys::{WebGl2RenderingContext as Gl, WebGlActiveInfo, WebGlProgram, WebGlUniformLocation};

use crate::runtime::dynamic_state::ContextUpdate;
use crate::runtime::{Connection, RenderingContext};
use crate::task::{GpuTask, Progress};
use crate::uniform::binding::UniformSlot;
use crate::uniform::{BindingError, Uniform, UniformIdentifier};
use crate::util::{arc_get_mut_unchecked, slice_make_mut, JsId};



