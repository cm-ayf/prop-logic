use std::cmp;
use std::collections::HashSet;
use std::fmt::Display;
use std::hash::Hash;

#[derive(Debug, PartialEq, Hash, Clone)]
pub enum Expr {
  Base(char),
  Cont,
  Not(Box<Self>),
  And(Box<Self>, Box<Self>),
  Or(Box<Self>, Box<Self>),
  To(Box<Self>, Box<Self>)
}

impl Expr {
  pub fn base_set(&self) -> HashSet<char> {
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

  pub fn eval(&self, trues: &HashSet<char>) -> bool {
    match self {
      Self::Base(c) => trues.contains(c),
      Self::Cont => false,
      Self::Not(expr) => !expr.eval(trues),
      Self::And(left, right) => left.eval(trues) && right.eval(trues),
      Self::Or(left, right) => left.eval(trues) || right.eval(trues),
      Self::To(left, right) => !left.eval(trues) || right.eval(trues)
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
      Expr::Base(_) => None,
      Expr::Cont => Some(&Expr::Cont),
      Expr::Not(expr) => {
        if refer.eq(expr) {
          Some(self)
        } else {
          expr.has(refer)
        }
      },
      Expr::And(left, right) => {
        if refer.eq(left) || refer.eq(right) {
          Some(self)
        } else {
          left.has(refer).map_or(right.has(refer), |e| Some(e))
        }
      }
      Expr::Or(left, right) => {
        if let (Some(_), Some(_)) = (left.has(refer), right.has(refer)) {
          Some(self)
        } else {
          None
        }
      }
      Expr::To(_, right) => {
        if refer.eq(right) {
          Some(self)
        } else {
          right.has(refer)
        }
      }
    }
  }

  fn is_low(&self) -> bool {
    matches!(self, Expr::Base(_) | Expr::Cont | Expr::Not(_))
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
  use std::collections::HashSet;

  #[test]
  fn test_base_set() {
    let expr = crate::parser("(A \\lor B) \\land C \\to (A \\land C) \\lor B \\land C").unwrap();
    let expect: HashSet<_> = ['A', 'B', 'C'].iter().cloned().collect();
    assert_eq!(
      expr.base_set(),
      expect
    );
  }

  #[test]
  fn test_eval() {
    let expr = crate::parser("(A \\lor B) \\land C").unwrap();
    assert!(!expr.eval(&vec!['A', 'B'].iter().cloned().collect()));
    assert!( expr.eval(&vec!['A', 'C'].iter().cloned().collect()));
    assert!(!expr.eval(&vec!['C'].iter().cloned().collect()));
    assert!( expr.eval(&vec!['A', 'B', 'C'].iter().cloned().collect()));
  }

  #[test]
  fn test_has() {
    let expr = crate::parser("(A \\lor B) \\land C").unwrap();

    let refer = crate::parser("A \\lor B").unwrap();
    assert_eq!(expr.has(&refer), Some(&expr));

    let refer = crate::parser("A").unwrap();
    assert_eq!(expr.has(&refer), Some(&crate::parser("(A \\lor B)").unwrap()));

    let refer = crate::parser("C").unwrap();
    assert_eq!(expr.has(&refer), Some(&expr));

    let refer = crate::parser("A \\land B").unwrap();
    assert_eq!(expr.has(&refer), None);
  }
}
