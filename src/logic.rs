//! 論理式を示す[Logic]列挙子を定義し，関連する関数を実装するするモジュールです．

use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt::Display;
use std::hash::Hash;
use std::str::FromStr;

use super::{parser, solver::*, TeX};

/// 論理式を示す列挙子です．木構造のノードです．
#[derive(Debug, PartialEq, Hash, Clone)]
pub enum Logic {
  Base(char),
  Cont,
  Not(Box<Self>),
  And(Box<Self>, Box<Self>),
  Or(Box<Self>, Box<Self>),
  To(Box<Self>, Box<Self>),
}

impl FromStr for Logic {
  type Err = parser::ParseLogicError;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    parser::parse(s)
      .map(|(_, logic)| logic)
      .map_err(|err| err.map_input(|str| str.to_string()))
  }
}

impl Logic {
  /// 文字列リテラルから新たな論理式の木を生成します．`s.parse() as Logic`と同じです．
  pub fn new<'a>(s: &'a str) -> Result<Self, parser::ParseLogicError> {
    Self::from_str(s)
  }

  /// 論理のメソッドで，その論理式を解くメソッドを呼び出します．
  pub fn solve(&self) -> Result<Inference, SolveError> {
    let mut i = Inference::new(self);
    i.solve()?;
    Ok(i)
  }

  /// 古典論理上証明可能かを確かめます．
  /// 論理式にあるすべての文字に真([None])または偽([Some(Logic::Cont)])を代入することで検証しています．
  pub fn check_all(&self) -> Result<(), CheckError> {
    let mut map = HashMap::new();
    let c = self
      .base_set()
      .into_iter()
      .next()
      .ok_or(CheckError::NoBase)?;
    for b in [true, false] {
      map.insert(c, b);
      match self.eval_part(&map) {
        Some(Self::Cont) => return Err(CheckError::TurnsOutFalse(self.clone(), map)),
        Some(logic) => logic.check_all().map_err(|s| match s {
          CheckError::NoBase => CheckError::NoBase,
          CheckError::TurnsOutFalse(_, mut map) => {
            map.insert(c, b);
            CheckError::TurnsOutFalse(self.clone(), map)
          }
        })?,
        None => (),
      };
    }
    Ok(())
  }

  /// 論理式にあるすべての文字を列挙します．
  fn base_set(&self) -> HashSet<char> {
    match self {
      Self::Base(c) => [c.to_owned()].iter().cloned().collect(),
      Self::Cont => HashSet::new(),
      Self::Not(logic) => logic.base_set(),
      Self::And(left, right) => left.base_set().union(&right.base_set()).cloned().collect(),
      Self::Or(left, right) => left.base_set().union(&right.base_set()).cloned().collect(),
      Self::To(left, right) => left.base_set().union(&right.base_set()).cloned().collect(),
    }
  }

  /// 論理式と任意の文字の真偽値をとり，その真偽値で評価できる限り真偽値を評価します．
  fn eval_part(&self, map: &HashMap<char, bool>) -> Option<Self> {
    match self {
      Self::Base(c) => match map.get(c) {
        Some(&b) => {
          if b {
            None
          } else {
            Some(Self::Cont)
          }
        }
        None => Some(Self::Base(*c)),
      },
      Self::Cont => Some(Self::Cont),
      Self::Not(logic) => match logic.eval_part(map) {
        Some(Self::Cont) => None,
        Some(logic) => Some(Self::Not(Box::new(logic))),
        None => Some(Self::Cont),
      },
      Self::And(left, right) => match (left.eval_part(map), right.eval_part(map)) {
        (Some(Self::Cont), _) => Some(Self::Cont),
        (_, Some(Self::Cont)) => Some(Self::Cont),
        (None, right) => right,
        (left, None) => left,
        (Some(left), Some(right)) => Some(Self::And(Box::new(left), Box::new(right))),
      },
      Self::Or(left, right) => match (left.eval_part(map), right.eval_part(map)) {
        (None, _) => None,
        (_, None) => None,
        (Some(Self::Cont), right) => right,
        (left, Some(Self::Cont)) => left,
        (Some(left), Some(right)) => Some(Self::Or(Box::new(left), Box::new(right))),
      },
      Self::To(left, right) => match (left.eval_part(map), right.eval_part(map)) {
        (Some(Self::Cont), _) => None,
        (_, None) => None,
        (None, right) => right,
        (Some(left), Some(Self::Cont)) => Some(Self::Not(Box::new(left))),
        (Some(left), Some(right)) => Some(Self::To(Box::new(left), Box::new(right))),
      },
    }
  }

  /// 論理式の結合順位を計算するための補助関数です．
  fn is_low(&self) -> bool {
    matches!(self, Self::Base(_) | Self::Cont | Self::Not(_))
  }
}

impl Eq for Logic {}

impl TeX for Logic {
  fn tex(&self) -> String {
    match self {
      Self::Base(c) => format!("{}", c),
      Self::Cont => format!("\\perp"),
      Self::Not(logic) => {
        if logic.is_low() {
          format!("\\lnot {}", logic.tex())
        } else {
          format!("\\lnot ({})", logic.tex())
        }
      }
      Self::And(left, right) => {
        let left = if left.is_low() {
          format!("{}", left.tex())
        } else {
          format!("({})", left.tex())
        };
        let right = if right.is_low() {
          format!("{}", right.tex())
        } else {
          format!("({})", right.tex())
        };
        format!("{} \\land {}", left, right)
      }
      Self::Or(left, right) => {
        let left = if left.is_low() {
          format!("{}", left.tex())
        } else {
          format!("({})", left.tex())
        };
        let right = if right.is_low() {
          format!("{}", right.tex())
        } else {
          format!("({})", right.tex())
        };
        format!("{} \\lor {}", left, right)
      }
      Self::To(left, right) => {
        let left = if let Self::To(_, _) = **left {
          format!("({})", left.tex())
        } else {
          format!("{}", left.tex())
        };
        let right = if let Self::To(_, _) = **right {
          format!("({})", right.tex())
        } else {
          format!("{}", right.tex())
        };
        format!("{} \\to {}", left, right)
      }
    }
  }
}

impl Display for Logic {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let string = self
      .tex()
      .replace("\\perp", "⊥")
      .replace("\\lnot", "¬")
      .replace("\\land", "∧")
      .replace("\\lor", "∨")
      .replace("\\to", "→");
    write!(f, "{}", string)
  }
}

/// 入力された論理式が古典論理上証明不可能である場合のエラーです．
#[derive(Debug)]
pub enum CheckError {
  TurnsOutFalse(Logic, HashMap<char, bool>),
  NoBase,
}

impl Display for CheckError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::TurnsOutFalse(logic, map) => write!(f, "{} turns out false when: {:?}", logic, map),
      Self::NoBase => write!(f, "no base"),
    }
  }
}

impl Error for CheckError {}

#[cfg(test)]
mod test {
  ///! テストを行うサブモジュールです．あまり充実していません…

  use super::*;

  #[test]
  fn test_base_set() {
    let logic = Logic::new("(A \\lor B) \\land C \\to (A \\land C) \\lor B \\land C").unwrap();
    let expect: HashSet<_> = ['A', 'B', 'C'].iter().cloned().collect();
    assert_eq!(logic.base_set(), expect);
  }
}
