use wasm_bindgen::prelude::wasm_bindgen;

// TODO: remove manual mapping of js_name
//   see https://github.com/rustwasm/wasm-bindgen/issues/1818

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log(s: &str);
}

#[wasm_bindgen(start)]
fn start() {
    log("Hello, world!");
}

#[wasm_bindgen(js_name = "printSomething")]
pub fn print_something() {
    log("something…");
}
