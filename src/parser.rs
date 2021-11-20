use nom::{
  IResult,
  branch::*,
  combinator::*,
  sequence::*,
  character::complete::*
};

use super::ast::*;

/*
  <expr>    := <term> [ [ ws ] '\to' ws <term>]
  <term>    := <and> | <or> | <factor>
  <and>     := <factor> [ ws ] '\land' ws ( <and> | <factor> )
  <or>      := <factor> [ ws ] '\lor' ws ( <or> | <factor> )
  <factor>  := [ '\lnot' ws ] ( <base> | <paren> )
  <paren>   := '(' [ ws ] <expr> [ ws ] ')'
  <base>    := A-Z
*/

fn base(s: &str) -> IResult<&str, Expr> {
  map(
    one_of("ABCDEFGHIJKLMNOPQRSTUVWXYZ"),
    |c| Expr::Base(c)
  )(s)
}

fn paren(s: &str) -> IResult<&str, Expr> {
  delimited(
    char('('),
    delimited(multispace0, expr, multispace0),
    char(')')
  )(s)
}

fn factor(s: &str) -> IResult<&str, Expr> {
  map(
    tuple((
      opt(tuple((
        tuple((char('\\'), char('l'), char('n'), char('o'), char('t'))),
        multispace1
      ))),
      alt((base, paren))
    )),
    |(opt, e)| {
      match opt {
        Some(_) => Expr::UnaryOp {
          op: UnaryOpKind::Not,
          expr: Box::new(e)
        },
        None => e
      }
    }
  )(s)
}

fn and(s: &str) -> IResult<&str, Expr> {
  map(
    tuple((
      factor,
      multispace0,
      tuple((char('\\'), char('l'), char('a'), char('n'), char('d'))),
      multispace1,
      alt((and, factor))
    )),
    |t| Expr::BinaryOp {
      op: BinaryOpKind::And,
      left: Box::new(t.0),
      right: Box::new(t.4)
    }
  )(s)
}

fn or(s: &str) -> IResult<&str, Expr> {
  map(
    tuple((
      factor,
      multispace0,
      tuple((char('\\'), char('l'), char('o'), char('r'))),
      multispace1,
      alt((or, factor))
    )),
    |t| Expr::BinaryOp {
      op: BinaryOpKind::Or,
      left: Box::new(t.0),
      right: Box::new(t.4)
    }
  )(s)
}

fn term(s: &str) -> IResult<&str, Expr> {
  alt((and, or, factor))(s)
}

fn expr(s: &str) -> IResult<&str, Expr> {
  map(
    tuple((
      opt(tuple((
        term,
        multispace0,
        tuple((char('\\'), char('t'), char('o'))),
        multispace1,
      ))),
      term
    )),
    |(opt, e)| {
      match opt {
        Some(t) => Expr::BinaryOp {
          op: BinaryOpKind::To,
          left: Box::new(t.0),
          right: Box::new(e)
        },
        None => e
      }
    }
  )(s)
}

pub fn parser<'a>(s: &str) -> Result<Expr, String> {
  match expr(s) {
    Ok((_, expr)) => Ok(expr),
    Err(error) => Err(error.to_string())
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use Expr::*;
  use UnaryOpKind::*;
  use BinaryOpKind::*;

  #[test]
  fn test_base() {
    assert_eq!(
      base("A").unwrap(),
      ("", Base('A'))
    );
  }

  #[test]
  fn test_paren() {
    assert_eq!(
      paren("(A)").unwrap(),
      ("", Base('A'))
    );
  }

  #[test]
  fn test_factor() {
    assert_eq!(
      factor("A").unwrap(),
      ("", Base('A'))
    );
    assert_eq!(
      factor("\\lnot (A)").unwrap(),
      ("", UnaryOp {
        op: Not,
        expr: Box::new(Base('A'))
      })
    );
  }

  #[test]
  fn test_and() {
    assert_eq!(
      and("A \\land B \\land C").unwrap(),
      ("", BinaryOp {
        op: And,
        left: Box::new(Base('A')),
        right: Box::new(BinaryOp {
          op: And,
          left: Box::new(Base('B')),
          right: Box::new(Base('C'))
        })
      })
    );
  }

  #[test]
  fn test_or() {
    assert_eq!(
      or("A \\lor B \\lor C").unwrap(),
      ("", BinaryOp {
        op: Or,
        left: Box::new(Base('A')),
        right: Box::new(BinaryOp {
          op: Or,
          left: Box::new(Base('B')),
          right: Box::new(Base('C'))
        })
      })
    );
  }

  #[test]
  fn test_term() {
    assert_eq!(
      term("\\lnot A").unwrap(),
      ("", UnaryOp {
        op: Not,
        expr: Box::new(Base('A'))
      })
    );
    assert_eq!(
      term("A \\land B").unwrap(),
      ("", BinaryOp {
        op: And,
        left: Box::new(Base('A')),
        right: Box::new(Base('B'))
      })
    );
    assert_eq!(
      term("A \\lor B").unwrap(),
      ("", BinaryOp {
        op: Or,
        left: Box::new(Base('A')),
        right: Box::new(Base('B'))
      })
    );
  }

  #[test]
  fn test_expr() {
    assert_eq!(
      expr("\\lnot A").unwrap(),
      ("", UnaryOp {
        op: Not,
        expr: Box::new(Base('A'))
      })
    );
    assert_eq!(
      expr("(A \\lor B \\to C) \\to ((A \\to C) \\land (B \\to C))").unwrap(),
      ("", BinaryOp {
        op: To,
        left: Box::new(BinaryOp {
          op: To,
          left: Box::new(BinaryOp {
            op: Or,
            left: Box::new(Base('A')),
            right: Box::new(Base('B'))
          }),
          right: Box::new(Base('C')),
        }),
        right: Box::new(BinaryOp {
          op: And,
          left: Box::new(BinaryOp {
            op: To,
            left: Box::new(Base('A')),
            right: Box::new(Base('C'))
          }),
          right: Box::new(BinaryOp {
            op: To,
            left: Box::new(Base('B')),
            right: Box::new(Base('C'))
          })
        })
      })
    )
  }
}