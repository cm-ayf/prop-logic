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
