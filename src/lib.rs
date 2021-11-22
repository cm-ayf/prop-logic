mod logic;
mod parser;
mod solver;
mod args;

pub use logic::Logic;
pub use args::Args;

trait TeX {
  fn tex(&self) -> String;
}