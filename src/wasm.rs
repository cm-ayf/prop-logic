use wasm_bindgen::prelude::*;
use crate::exec;

#[wasm_bindgen]
#[allow(unused)]
pub fn main(input: &str, tex: bool) -> String {
  match exec::exec(&input.to_string(), tex) {
    Ok(res) => res,
    Err(e) => e.to_string(),
    _ => "unexpected error".to_string()
  }
}

