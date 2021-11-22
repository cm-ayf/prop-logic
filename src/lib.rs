mod args;
mod logic;
mod parser;
mod solver;
mod wasm;
mod exec;

pub use args::Args;
pub use logic::Logic;

trait TeX {
  fn tex(&self) -> String;
}
