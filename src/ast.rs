use std::collections::HashSet;
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

  pub fn eval(&self, trues: &Vec<char>) -> bool {
    match self {
      Self::Base(c) => trues.binary_search(c).is_ok(),
      Self::Cont => false,
      Self::Not(expr) => !expr.eval(trues),
      Self::And(left, right) => left.eval(trues) && right.eval(trues),
      Self::Or(left, right) => left.eval(trues) || right.eval(trues),
      Self::To(left, right) => !left.eval(trues) || right.eval(trues)
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
      Expr::Or(left, right) =>{
        if left.has(refer) != None && right.has(refer) != None {
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
}

impl Eq for Expr {}

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
    assert!(!expr.eval(&vec!['A', 'B']));
    assert!( expr.eval(&vec!['A', 'C']));
    assert!(!expr.eval(&vec!['C']));
    assert!( expr.eval(&vec!['A', 'B', 'C']));
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
