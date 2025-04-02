use wasm_bindgen::prelude::wasm_bindgen;

#[macro_export]
macro_rules! log {
    ($($t:tt)*) => ($crate::log::log(&format_args!($($t)*).to_string()))
}

#[macro_export]
macro_rules! info {
    ($($t:tt)*) => ($crate::log::cn_info(&format_args!($($t)*).to_string()))
}

#[macro_export]
macro_rules! warn {
    ($($t:tt)*) => ($crate::log::cn_warn(&format_args!($($t)*).to_string()))
}

#[macro_export]
macro_rules! error {
    ($($t:tt)*) => ($crate::log::cn_error(&format_args!($($t)*).to_string()))
}

#[macro_export]
macro_rules! trace {
    ($($t:tt)*) => ($crate::log::trace(&format_args!($($t)*).to_string()))
}

#[macro_export]
macro_rules! debug {
    ($($t:tt)*) => ($crate::log::debug(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(a: &str);

    #[wasm_bindgen(js_namespace = console)]
    pub fn cn_error(a: &str);

    #[wasm_bindgen(js_namespace = console)]
    pub fn cn_warn(a: &str);

    #[wasm_bindgen(js_namespace = console)]
    pub fn cn_info(a: &str);

    #[wasm_bindgen(js_namespace = console)]
    pub fn debug(a: &str);

    #[wasm_bindgen(js_namespace = console)]
    pub fn trace(a: &str);
}
