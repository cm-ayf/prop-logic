use std::cmp;
use std::collections::{HashSet, HashMap};
use std::fmt::Display;
use std::hash::Hash;
use std::str::FromStr;
use nom::{Err, error::Error};

use super::{parser, solver};

#[derive(Debug, PartialEq, Hash, Clone)]
pub enum Logic {
  Base(char),
  Cont,
  Not(Box<Self>),
  And(Box<Self>, Box<Self>),
  Or(Box<Self>, Box<Self>),
  To(Box<Self>, Box<Self>)
}

impl FromStr for Logic {
  type Err = Err<Error<String>>;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    parser::expr(s)
      .map(|(_, logic)| logic)
      .map_err(|err| {
        err.map_input(|str| str.to_string())
      })
  }
}

impl Logic {
  pub fn new<'a>(s: &'a str) -> Result<Self, Err<Error<String>>> {
    Self::from_str(s)
  }

  pub fn solve(&self) -> Result<String, ()> {
    let mut i = solver::Inference::new(self);
    i.solve().map(|i| format!("{:?}", i))
  }

  pub fn check_all(&self) -> Result<(), String> {
    let c = self.base_set().into_iter().next().ok_or("no base".to_string())?;
    let mut map = HashMap::new();
    for b in [true, false] {
      map.insert(c, b);
      match self.eval_part(&map) {
        Some(Self::Cont) => return Err(format!("{}: {}", c, b)),
        Some(logic) => logic.check_all().map_err(|s| format!("{}, {}: {}", s, c, b))?,
        None => ()
      };
    }
    Ok(())
  }

  fn base_set(&self) -> HashSet<char> {
    match self {
      Self::Base(c) => [c.to_owned()].iter().cloned().collect(),
      Self::Cont => HashSet::new(),
      Self::Not(logic) => logic.base_set(),
      Self::And(left, right) =>
        left.base_set().union(&right.base_set()).cloned().collect(),
      Self::Or(left, right) =>
        left.base_set().union(&right.base_set()).cloned().collect(),
      Self::To(left, right) =>
        left.base_set().union(&right.base_set()).cloned().collect()
    }
  }

  fn eval_part(&self, map: &HashMap<char, bool>) -> Option<Self> {
    match self {
      Self::Base(c) => match map.get(c) {
        Some(&b) => if b {
          None
        } else {
          Some(Self::Cont)
        },
        None => Some(Self::Base(*c))
      },
      Self::Cont => Some(Self::Cont),
      Self::Not(logic) => match logic.eval_part(map) {
        Some(Self::Cont) => None,
        Some(logic) => Some(Self::Not(Box::new(logic))),
        None => Some(Self::Cont)
      },
      Self::And(left, right) =>
        match (left.eval_part(map), right.eval_part(map)) {
          (Some(Self::Cont), _) => Some(Self::Cont),
          (_, Some(Self::Cont)) => Some(Self::Cont),
          (None, right) => right,
          (left, None) => left,
          (Some(left), Some(right)) => Some(Self::And(Box::new(left), Box::new(right)))
        },
      Self::Or(left, right) =>
        match (left.eval_part(map), right.eval_part(map)) {
          (None, _) => None,
          (_, None) => None,
          (Some(Self::Cont), right) => right,
          (left, Some(Self::Cont)) => left,
          (Some(left), Some(right)) => Some(Self::Or(Box::new(left), Box::new(right)))
        },
      Self::To(left, right) =>
        match (left.eval_part(map), right.eval_part(map)) {
          (Some(Self::Cont), _) => None,
          (_, None) => None,
          (None, right) => right,
          (Some(left), Some(Self::Cont)) => Some(Self::Not(Box::new(left))),
          (Some(left), Some(right)) => Some(Self::To(Box::new(left), Box::new(right)))
        },
    }
  }

  fn depth(&self) -> usize {
    match self {
      Self::Base(_) => 0,
      Self::Cont => 0,
      Self::Not(logic) => logic.depth() + 1,
      Self::And(left, right) => cmp::max(left.depth(), right.depth()) + 1,
      Self::Or(left, right) => cmp::max(left.depth(), right.depth()) + 1,
      Self::To(left, right) => cmp::max(left.depth(), right.depth()) + 1
    }
  }

  pub fn has(&self, refer: &Self) -> Vec<&Self> {
    let mut vec = Vec::new();
    if self == refer {
      vec.push(self);
    }

    match self {
      Self::Base(_) => (),
      Self::Cont => vec.push(&Self::Cont),
      Self::Not(logic) => {
        if refer.eq(logic) {
          vec.push(self);
        } else {
          vec.append(&mut logic.has(refer));
        }
      },
      Self::And(left, right) => {
        if refer.eq(left) || refer.eq(right) {
          vec.push(self);
        } else {
          vec.append(&mut left.has(refer));
          vec.append(&mut right.has(refer));
        }
      }
      Self::Or(left, right) => {
        if left.has(refer).len() > 0 && right.has(refer).len() > 0 {
          vec.push(self);
        }
      }
      Self::To(_, right) => {
        if refer.eq(right) {
          vec.push(self);
        } else {
          vec.append(&mut right.has(refer));
        }
      }
    }
    vec
  }

  pub fn children(&self) -> HashSet<&Self> {
    let mut set = match self {
      Self::Not(logic) => logic.children(),
      Self::And(left, right) =>
        left.children().union(&right.children()).cloned().collect(),
      Self::Or(left, right) =>
        left.children().union(&right.children()).cloned().collect(),
      Self::To(left, right) =>
        left.children().union(&right.children()).cloned().collect(),
      _ => HashSet::new()
    };
    set.insert(self);
    set
  }

  fn is_low(&self) -> bool {
    matches!(self, Self::Base(_) | Self::Cont | Self::Not(_))
  }
}

impl Eq for Logic {}

impl PartialOrd for Logic {
  fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
    self.depth().partial_cmp(&other.depth())
  }
}

impl Ord for Logic {
  fn cmp(&self, other: &Self) -> cmp::Ordering {
    self.depth().cmp(&other.depth())
  }
}

impl Display for Logic {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Base(c) => write!(f, "{}", c),
      Self::Cont => write!(f, "\\perp"),
      Self::Not(logic) =>
        if logic.is_low() {
          write!(f, "\\lnot {}", logic)
        } else {
          write!(f, "\\lnot ({})", logic)
        },
      Self::And(left, right) => {
        let left = if left.is_low() {
          format!("{}", left)
        } else {
          format!("({})", left)
        };
        let right = if right.is_low() {
          format!("{}", right)
        } else {
          format!("({})", right)
        };
        write!(f, "{} \\land {}", left, right)
      },
      Self::Or(left, right) => {
        let left = if left.is_low() {
          format!("{}", left)
        } else {
          format!("({})", left)
        };
        let right = if right.is_low() {
          format!("{}", right)
        } else {
          format!("({})", right)
        };
        write!(f, "{} \\lor {}", left, right)
      },
      Self::To(left, right) =>{
        let left = if let Self::To(_, _) = **left {
          format!("({})", left)
        } else {
          format!("{}", left)
        };
        let right = if let Self::To(_, _) = **right {
          format!("({})", right)
        } else {
          format!("{}", right)
        };
        write!(f, "{} \\to {}", left, right)
      },
    }
    
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn test_base_set() {
    let logic = Logic::new("(A \\lor B) \\land C \\to (A \\land C) \\lor B \\land C").unwrap();
    let expect: HashSet<_> = ['A', 'B', 'C'].iter().cloned().collect();
    assert_eq!(
      logic.base_set(),
      expect
    );
  }

  #[test]
  fn test_has() {
    let logic = Logic::new("(A \\lor B) \\land C \\to (A \\land C) \\lor (B \\land C)").unwrap();

    let refer = Logic::new("A \\lor B").unwrap();
    assert_eq!(logic.has(&refer), vec![] as Vec<&Logic>);

    let refer = Logic::new("A").unwrap();
    assert_eq!(logic.has(&refer), vec![] as Vec<&Logic>);

    let refer = Logic::new("C").unwrap();
    assert_eq!(logic.has(&refer), vec![
      &Logic::new("(A \\land C) \\lor (B \\land C)").unwrap()
    ]);

    let refer = Logic::new("A \\land B").unwrap();
    assert_eq!(logic.has(&refer), vec![] as Vec<&Logic>);
  }
}
