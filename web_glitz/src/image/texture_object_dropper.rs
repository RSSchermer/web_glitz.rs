use wasm_bindgen::JsCast;

use crate::runtime::{Connection, RenderingContext};
use crate::task::{ContextId, GpuTask, Progress};
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

unsafe impl GpuTask<Connection> for TextureDropCommand {
    type Output = ();

    fn context_id(&self) -> ContextId {
        ContextId::Any
    }

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output> {
        let (gl, state) = unsafe { connection.unpack_mut() };

        state
            .framebuffer_cache_mut()
            .remove_attachment_dependents(self.id, gl);

        let value = unsafe { JsId::into_value(self.id).unchecked_into() };

        state.unref_texture(&value);
        gl.delete_texture(Some(&value));

        Progress::Finished(())
    }
}
