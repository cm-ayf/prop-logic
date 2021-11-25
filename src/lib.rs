mod exec;
mod logic;
mod parser;
mod solver;

pub use logic::*;
pub use solver::*;
pub use exec::*;

pub trait TeX {
  fn tex(&self) -> String;
}
