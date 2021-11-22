mod logic;
mod parser;
mod solver;
mod wasm;
mod exec;

pub use logic::Logic;

trait TeX {
  fn tex(&self) -> String;
}
