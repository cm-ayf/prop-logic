mod args;
mod exec;
mod logic;
mod parser;
mod solver;

pub use args::Args;
pub use logic::Logic;

trait TeX {
  fn tex(&self) -> String;
}
