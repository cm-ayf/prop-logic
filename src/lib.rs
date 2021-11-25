mod args;
mod exec;
mod logic;
mod parser;
mod solver;

pub use args::Args;

trait TeX {
  fn tex(&self) -> String;
}
