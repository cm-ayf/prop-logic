mod exec;
mod logic;
mod parser;
mod solver;
mod wasm;

pub use logic::Logic;

trait TeX {
  fn tex(&self) -> String;
}
