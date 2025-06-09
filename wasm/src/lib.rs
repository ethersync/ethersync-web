use wasm_bindgen::prelude::*;

// TODO: remove manual mapping of js_name
//   see https://github.com/rustwasm/wasm-bindgen/issues/1818

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log(s: &str);
}

#[wasm_bindgen(start)]
fn start() {
    log("WASM loaded");
}

#[wasm_bindgen]
pub struct Counter {
    value: i32,
}

#[wasm_bindgen]
impl Counter {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self { value: 0 }
    }

    pub fn increase(&mut self) {
        self.value += 1;
    }

    #[wasm_bindgen(js_name = getText)]
    pub fn get_text(&self) -> String {
        format!("count is {value}", value = self.value)
    }
}
