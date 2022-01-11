//! 実行する流れをまとめた関数と，その際に出るエラーをまとめた構造を実装するモジュールです．
//! CLI Appからマイナーな変更でマージできるよう，[wasm](super::wasm)モジュールから分離されています．

use std::error::Error;
use std::fmt::Display;

use super::logic::*;
use super::parser::ParseLogicError;
use super::solver::SolveError;
use super::TeX;

/// 入力された文字列から論理式をパースし，ソルバを呼び出し，設定に則って出力します．
  pub fn exec(input: &str, tex: bool) -> Result<String, ExecError> {
  // Logic::from(&str) as FromStr を呼び出しています．
  let logic: Logic = input.parse()?;

  logic.check_all()?;

  let inference = logic.solve()?;

  Ok(if tex {
    inference.tex()
  } else {
    inference.to_string()
  })
}

/// 実行時のエラーをまとめた列挙子です．
#[derive(Debug)]
pub enum ExecError {
  /// 入力文字列をパースした場合のエラーです．
  ParseError(ParseLogicError),

  /// 入力された論理式が古典論理上証明不可能である場合のエラーです．
  CheckError(CheckError),

  /// 入力された論理式を証明できなかった場合のエラーです．必ずしも直観主義論理上証明不可能な命題であることを意味しません．
  SolveError(SolveError),

  /// 出力形式をファイルにした際に出力できなかった場合のエラーです．
  FileError(std::io::Error),
}

impl From<ParseLogicError> for ExecError {
  fn from(e: ParseLogicError) -> Self {
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
