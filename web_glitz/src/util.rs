use std::mem;
use std::ops::Deref;

use std::sync::Arc;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;

// This is a hack untill wasm_bindgen's API settles around `anyref`, see
// https://github.com/rustwasm/wasm-bindgen/issues/999

#[derive(Clone, Copy, Debug, PartialEq, Hash)]
pub(crate) struct JsId {
    // TODO: figure out if we could use NonZero here
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

    pub(crate) unsafe fn with_value_unchecked<F, T, R>(&self, f: F) -> R
    where
        F: FnOnce(&T) -> R,
        T: JsCast,
    {
        let value = unsafe { JsId::into_value(self.clone()).unchecked_into() };

        let result = f(&value);

        mem::forget(value);

        result
    }
}

pub(crate) unsafe fn arc_get_mut_unchecked<T>(arc: &Arc<T>) -> &mut T {
    unsafe {
        let ptr = arc.deref() as *const _;

        &mut *(ptr as *mut _)
    }
}

pub(crate) fn identical<T>(a: Option<&T>, b: Option<&T>) -> bool
where
    T: AsRef<JsValue>,
{
    a.map(|t| t.as_ref()) == b.map(|t| t.as_ref())
}

pub(crate) unsafe fn slice_make_mut<T>(slice: &[T]) -> &mut [T] {
    &mut *(slice as *const _ as *mut _)
}
