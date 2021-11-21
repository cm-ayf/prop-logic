use std::cmp;
use std::collections::{HashSet, HashMap};
use std::fmt::Display;
use std::hash::Hash;
use std::str::FromStr;

use super::{parser, solver};

#[derive(Debug, PartialEq, Hash, Clone)]
pub enum Expr {
  Base(char),
  Cont,
  Not(Box<Self>),
  And(Box<Self>, Box<Self>),
  Or(Box<Self>, Box<Self>),
  To(Box<Self>, Box<Self>)
}

impl FromStr for Expr {
  type Err = String;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match parser::expr(s) {
      Ok((_, expr)) => Ok(expr),
      Err(error) => Err(error.to_string())
    }
  }
}

impl Expr {
  pub fn new(s: &str) -> Result<Self, String> {
    Self::from_str(s)
  }

  pub fn solve(&self) -> Result<String, ()> {
    let mut i = solver::InferenceNode::new(self);
    i.solve().map(|i| format!("{:?}", i))
  }

  pub fn check_all(&self) -> Result<(), String> {
    let c = self.base_set().into_iter().next().ok_or("no base".to_string())?;
    let mut map = HashMap::new();
    for b in [true, false] {
      map.insert(c, b);
      match self.eval_part(&map) {
        Some(Self::Cont) => return Err(format!("{}: {}", c, b)),
        Some(expr) => expr.check_all().map_err(|s| format!("{}, {}: {}", s, c, b))?,
        None => ()
      };
    }
    Ok(())
  }

  fn base_set(&self) -> HashSet<char> {
    match self {
      Self::Base(c) => [c.to_owned()].iter().cloned().collect(),
      Self::Cont => HashSet::new(),
      Self::Not(expr) => expr.base_set(),
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
      Self::Not(expr) => match expr.eval_part(map) {
        Some(Self::Cont) => None,
        Some(expr) => Some(Self::Not(Box::new(expr))),
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
      Self::Not(expr) => expr.depth() + 1,
      Self::And(left, right) => cmp::max(left.depth(), right.depth()) + 1,
      Self::Or(left, right) => cmp::max(left.depth(), right.depth()) + 1,
      Self::To(left, right) => cmp::max(left.depth(), right.depth()) + 1
    }
  }

  pub fn has(&self, refer: &Self) -> Option<&Self> {
    if self == refer {
      return Some(self);
    }

    match self {
      Self::Base(_) => None,
      Self::Cont => Some(&Self::Cont),
      Self::Not(expr) => {
        if refer.eq(expr) {
          Some(self)
        } else {
          expr.has(refer)
        }
      },
      Self::And(left, right) => {
        if refer.eq(left) || refer.eq(right) {
          Some(self)
        } else {
          left.has(refer).map_or(right.has(refer), |e| Some(e))
        }
      }
      Self::Or(left, right) => {
        if let (Some(_), Some(_)) = (left.has(refer), right.has(refer)) {
          Some(self)
        } else {
          None
        }
      }
      Self::To(_, right) => {
        if refer.eq(right) {
          Some(self)
        } else {
          right.has(refer)
        }
      }
    }
  }

  fn is_low(&self) -> bool {
    matches!(self, Self::Base(_) | Self::Cont | Self::Not(_))
  }
}

impl Eq for Expr {}

impl PartialOrd for Expr {
  fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
    self.depth().partial_cmp(&other.depth())
  }
}

impl Ord for Expr {
  fn cmp(&self, other: &Self) -> cmp::Ordering {
    self.depth().cmp(&other.depth())
  }
}

impl Display for Expr {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Base(c) => write!(f, "{}", c),
      Self::Cont => write!(f, "\\perp"),
      Self::Not(expr) =>
        if expr.is_low() {
          write!(f, "\\lnot {}", expr)
        } else {
          write!(f, "\\lnot ({})", expr)
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
      Self::To(left, right) =>
        write!(f, "{} \\to {}", left, right),
    }
    
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn test_base_set() {
    let expr = Expr::new("(A \\lor B) \\land C \\to (A \\land C) \\lor B \\land C").unwrap();
    let expect: HashSet<_> = ['A', 'B', 'C'].iter().cloned().collect();
    assert_eq!(
      expr.base_set(),
      expect
    );
  }

  #[test]
  fn test_has() {
    let expr = Expr::new("(A \\lor B) \\land C").unwrap();

    let refer = Expr::new("A \\lor B").unwrap();
    assert_eq!(expr.has(&refer), Some(&expr));

    let refer = Expr::new("A").unwrap();
    assert_eq!(expr.has(&refer), Some(&Expr::new("(A \\lor B)").unwrap()));

    let refer = Expr::new("C").unwrap();
    assert_eq!(expr.has(&refer), Some(&expr));

    let refer = Expr::new("A \\land B").unwrap();
    assert_eq!(expr.has(&refer), None);
  }
}
