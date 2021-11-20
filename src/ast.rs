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

#[derive(Clone, Copy)]
pub enum UnaryOpKind {
  Not
}

#[derive(Clone, Copy)]
pub enum BinaryOpKind {
  To,
  And,
  Or
}

impl Expr {
  pub fn new_base(c: char) -> Self {
    Self::Base(c)
  }

  pub fn new_unary(op: UnaryOpKind, expr: Self) -> Self {
      Self::UnaryOp{
        op,
        expr: Box::new(expr)
      }
  }

  pub fn new_binary(op: BinaryOpKind, left: Self, right: Self) -> Self {
    Self::BinaryOp{
      op,
      left: Box::new(left),
      right: Box::new(right),
    }
  }
}

