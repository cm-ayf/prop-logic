use wasm_bindgen::prelude::*;
use crate::exec;

#[wasm_bindgen]
pub fn main(input: &str, tex: bool) -> String {
  match exec::exec(&input.to_string(), tex, &None) {
    Ok(Some(res)) => res,
    Err(e) => e.to_string(),
    _ => "unexpected error".to_string()
  }
}

