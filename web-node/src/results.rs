use serde::Serialize;
use std::fmt::Display;
use wasm_bindgen::JsValue;

pub type WebResult = Result<JsValue, JsValue>;

pub(crate) trait ToJsValue: Serialize + Sized {
    fn to_js(&self) -> WebResult {
        Ok(serde_wasm_bindgen::to_value(self)?)
    }
}

impl<T: Serialize> ToJsValue for T {}

pub(crate) trait ConvertWasmResultError<O> {
    fn error_to_js(self) -> Result<O, JsValue>;
}

impl<O, E> ConvertWasmResultError<O> for Result<O, E>
where
    E: Display,
{
    fn error_to_js(self) -> Result<O, JsValue> {
        match self {
            Ok(o) => Ok(o),
            Err(e) => Err(e.to_string().trim_start_matches("Error:").trim().to_js()?),
        }
    }
}

pub(crate) trait ConvertWasmResult {
    fn to_js(self) -> WebResult;
}

impl<O, E> ConvertWasmResult for Result<O, E>
where
    O: ToJsValue,
    E: Display,
{
    fn to_js(self) -> WebResult {
        self.error_to_js()?.to_js()
    }
}
