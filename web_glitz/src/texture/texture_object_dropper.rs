use wasm_bindgen::JsCast;

use crate::runtime::{RenderingContext, Connection};
use crate::task::{GpuTask, Progress};
use crate::util::JsId;

pub(super) trait TextureObjectDropper {
    fn drop_texture_object(&self, id: JsId);
}

impl<T> TextureObjectDropper for T
    where
        T: RenderingContext,
{
    fn drop_texture_object(&self, id: JsId) {
        self.submit(TextureDropCommand { id });
    }
}

struct TextureDropCommand {
    id: JsId,
}

impl GpuTask<Connection> for TextureDropCommand {
    type Output = ();

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output> {
        let (gl, _) = unsafe { connection.unpack() };
        let value = unsafe { JsId::into_value(self.id) };

        gl.delete_texture(Some(&value.unchecked_into()));

        Progress::Finished(())
    }
}
