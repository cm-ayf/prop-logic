mod args;
mod logic;
mod parser;
mod solver;
mod exec;

pub use args::Args;
pub use logic::Logic;

trait TeX {
  fn tex(&self) -> String;
}
