use wasm_bindgen::prelude::*;

use super::exec::*;

#[wasm_bindgen]
#[allow(unused)]
pub fn main(input: &str, tex: bool) -> Result<String, JsValue> {
  exec(input, tex).map_err(|e| JsValue::from_str(&e.to_string()))
}