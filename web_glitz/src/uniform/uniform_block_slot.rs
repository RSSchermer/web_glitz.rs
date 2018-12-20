use crate::program::UniformInfo;
use crate::rendering_context::Connection;

pub struct UniformBlockSlot<'a> {
    uniform: &'a mut UniformInfo,
    connection: *mut Connection,
}
