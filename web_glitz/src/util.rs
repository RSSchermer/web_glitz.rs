use std::mem;

use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;

// This is a hack untill wasm_bindgen's API settles around `anyref`, see
// https://github.com/rustwasm/wasm-bindgen/issues/999

#[derive(Clone, Copy, Debug)]
pub(crate) struct JsId {
    id: u32,
}

impl JsId {
    pub(crate) fn from_value(value: JsValue) -> JsId {
        unsafe {
            JsId {
                id: mem::transmute(value),
            }
        }
    }

    /// Only safe to call in the same thread that originally created the `id`.
    pub(crate) unsafe fn into_value(id: JsId) -> JsValue {
        mem::transmute(id.id)
    }

    pub(crate) fn with_value_unchecked<F, T>(&self, f: F)
    where
        F: FnOnce(&T),
        T: JsCast,
    {
        let value = unsafe { JsId::into_value(self.clone()).unchecked_into() };

        f(&value);

        mem::forget(value);
    }
}
