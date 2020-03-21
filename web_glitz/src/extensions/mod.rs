use crate::runtime::Connection;

pub mod color_buffer_float;
pub mod texture_float_linear;

pub trait Extension: Sized {
    fn try_init(connection: &mut Connection, context_id: usize) -> Option<Self>;
}