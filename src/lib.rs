mod ast;
mod parser;
mod solver;

pub use ast::Logic;

pub trait TeX {
  fn tex(&self) -> String;
}