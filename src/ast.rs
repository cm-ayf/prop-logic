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

  fn eval(&self, trues: &Vec<char>) -> bool {
    match self {
      Self::Base(c) => trues.binary_search(c).is_ok(),
      Self::Cont => false,
      Self::Not(expr) => !expr.eval(trues),
      Self::And(left, right) => left.eval(trues) && right.eval(trues),
      Self::Or(left, right) => left.eval(trues) || right.eval(trues),
      Self::To(left, right) => !left.eval(trues) || right.eval(trues)
    }
  }
}

impl Eq for Expr {}

#[cfg(test)]
mod test {
  use std::collections::HashSet;

  #[test]
  fn test_base_list() {
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
}
