use std::error::Error;
use std::fmt::Display;
use std::path::PathBuf;

use crate::{Logic, TeX};
use crate::logic::CheckError;
use crate::solver::SolveError;

pub fn exec(input: &String, tex: bool, out: &Option<PathBuf>) -> Result<Option<String>, ExecError> {
  let logic: Logic = input.parse()?;

  logic.check_all()?;

  let inference = logic.solve()?;

  let res = if tex {
    inference.tex()
  } else {
    inference.to_string()
  };

  match out {
    Some(ref path) => {
      std::fs::write(path, res)?;
      Ok(None)
    },
    None => Ok(Some(res)),
  }
}

#[derive(Debug)]
pub enum ExecError {
  ParseError(nom::Err<nom::error::Error<String>>),
  CheckError(CheckError),
  SolveError(SolveError),
  FileError(std::io::Error),
}

impl From<nom::Err<nom::error::Error<String>>> for ExecError {
  fn from(e: nom::Err<nom::error::Error<String>>) -> Self {
    Self::ParseError(e)
  }
}

impl From<CheckError> for ExecError {
  fn from(e: CheckError) -> Self {
    Self::CheckError(e)
  }
}

impl From<SolveError> for ExecError {
  fn from(e: SolveError) -> Self {
    Self::SolveError(e)
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
      Self::SolveError(e) => write!(f, "error when solving:\n{}", e),
      Self::FileError(e) => write!(f, "error when writing file:\n{}", e),
    }
  }
}

impl Error for ExecError {
  fn source(&self) -> Option<&(dyn Error + 'static)> {
    match self {
      Self::ParseError(e) => Some(e),
      Self::CheckError(e) => Some(e),
      Self::SolveError(e) => Some(e),
      Self::FileError(e) => Some(e),
    }
  }
}
