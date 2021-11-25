mod exec;
mod logic;
mod parser;
mod solver;
mod wasm;

pub use exec::*;
pub use logic::*;
pub use solver::*;

pub trait TeX {
  fn tex(&self) -> String;
}
