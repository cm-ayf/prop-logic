use std::error::Error;
use std::fmt::Display;

use super::logic::*;
use super::parser::ParseLogicError;
use super::solver::SolveError;
use super::TeX;

pub fn exec(input: &str, tex: bool) -> Result<String, ExecError> {
  let logic: Logic = input.parse()?;

  let inference = logic.solve()?;

  Ok(if tex {
    inference.tex()
  } else {
    inference.to_string()
  })
}

#[derive(Debug)]
pub enum ExecError {
  ParseError(ParseLogicError),
  CheckError(CheckError),
  InferError(Logic),
  FileError(std::io::Error),
}

impl From<ParseLogicError> for ExecError {
  fn from(e: ParseLogicError) -> Self {
    Self::ParseError(e)
  }
}

impl From<SolveError> for ExecError {
  fn from(e: SolveError) -> Self {
    match e {
      SolveError::CheckError(e) => Self::CheckError(e),
      SolveError::InferError(e) => Self::InferError(e),
    }
  }
}

impl From<std::io::Error> for ExecError {
  fn from(e: std::io::Error) -> Self {
    Self::FileError(e)
  }
}

impl Display for ExecError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::ParseError(e) => write!(f, "error when parsing:\n{}", e),
      Self::CheckError(e) => write!(f, "error when checking:\n{}", e),
      Self::InferError(e) => write!(f, "could not infer:\n{}", e),
      Self::FileError(e) => write!(f, "error when writing file:\n{}", e),
    }
  }
}

impl Error for ExecError {
  fn source(&self) -> Option<&(dyn Error + 'static)> {
    match self {
      Self::ParseError(e) => Some(e),
      Self::CheckError(e) => Some(e),
      Self::InferError(_) => None,
      Self::FileError(e) => Some(e),
    }
  }
}
