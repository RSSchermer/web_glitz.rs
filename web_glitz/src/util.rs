use std::mem;

use wasm_bindgen::convert::{FromWasmAbi, IntoWasmAbi};
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;

// This is a hack until wasm_bindgen's API settles around `anyref`, see
// https://github.com/rustwasm/wasm-bindgen/issues/999

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct JsId {
    // TODO: figure out if we could use NonZero here
    id: u32,
}

impl JsId {
    pub(crate) fn from_value(value: JsValue) -> JsId {
        JsId::from_abi(value.into_abi())
    }

    pub(crate) fn from_abi(abi: u32) -> JsId {
        JsId { id: abi }
    }

    /// Only safe to call in the same thread that originally created the `id`.
    pub(crate) unsafe fn into_value(id: JsId) -> JsValue {
        JsValue::from_abi(id.id)
    }

    /// Only safe to call in the same thread that originally created the [JsId].
    pub(crate) unsafe fn with_value_unchecked<F, T, R>(&self, f: F) -> R
    where
        F: FnOnce(&T) -> R,
        T: JsCast,
    {
        let value = JsId::into_value(self.clone()).unchecked_into();

        let result = f(&value);

        mem::forget(value);

        result
    }
}

pub(crate) fn identical<A, B>(a: Option<&A>, b: Option<&B>) -> bool
where
    A: AsRef<JsValue>,
    B: AsRef<JsValue>,
{
    a.map(|t| t.as_ref()) == b.map(|t| t.as_ref())
}
