mod logic;
mod parser;
mod solver;

pub use logic::Logic;

pub trait TeX {
  fn tex(&self) -> String;
}