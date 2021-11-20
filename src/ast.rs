use std::collections::HashSet;

#[derive(Debug, PartialEq)]
pub enum Expr {
  Base(char),
  UnaryOp {
    op: UnaryOpKind,
    expr: Box<Self>
  },
  BinaryOp {
    op: BinaryOpKind,
    left: Box<Self>,
    right: Box<Self>
  }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum UnaryOpKind {
  Not
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum BinaryOpKind {
  To,
  And,
  Or
}

impl Expr {
  fn base_set(&self) -> HashSet<char> {
    match self {
      Self::Base(c) => [c.to_owned()].iter().cloned().collect(),
      Self::UnaryOp { expr, .. } => expr.base_set(),
      Self::BinaryOp { left, right, .. } =>
        left.base_set().union(&right.base_set()).cloned().collect()
    }
  }

  fn eval(&self, trues: &Vec<char>) -> bool {
    match self {
      Self::Base(c) => trues.binary_search(c).is_ok(),
      Self::UnaryOp { op, expr } => match op {
        UnaryOpKind::Not => !expr.eval(trues)
      },
      Self::BinaryOp { op, left, right } => match op {
        BinaryOpKind::And => left.eval(trues) && right.eval(trues),
        BinaryOpKind::Or => left.eval(trues) || right.eval(trues),
        BinaryOpKind::To => !left.eval(trues) || right.eval(trues)
      }
    }
  }
}

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
