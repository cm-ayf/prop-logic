use nom::{
  IResult,
  branch::*,
  combinator::*,
  sequence::*,
  character::complete::*
};

use super::logic::*;

/*
  <expr>    := <term> [ [ ws ] '\to' ws <term>]
  <term>    := <and> | <or> | <factor>
  <and>     := <factor> [ ws ] '\land' ws <factor>
  <or>      := <factor> [ ws ] '\lor' ws <factor>
  <factor>  := [ '\lnot' ws ] ( <base> | <paren> )
  <paren>   := '(' [ ws ] <expr> [ ws ] ')'
  <base>    := A-Z
*/

fn base(s: &str) -> IResult<&str, Logic> {
  map(
    one_of("ABCDEFGHIJKLMNOPQRSTUVWXYZ"),
    |c| Logic::Base(c)
  )(s)
}

fn paren(s: &str) -> IResult<&str, Logic> {
  delimited(
    char('('),
    delimited(multispace0, expr, multispace0),
    char(')')
  )(s)
}

fn factor(s: &str) -> IResult<&str, Logic> {
  map(
    tuple((
      opt(tuple((
        tuple((opt(char('\\')), opt(char('l')), char('n'), char('o'), char('t'))),
        multispace1
      ))),
      alt((base, paren))
    )),
    |(opt, e)| {
      match opt {
        Some(_) => Logic::Not(Box::new(e)),
        None => e
      }
    }
  )(s)
}

fn and(s: &str) -> IResult<&str, Logic> {
  map(
    tuple((
      factor,
      multispace0,
      tuple((opt(char('\\')), opt(char('l')), char('a'), char('n'), char('d'))),
      multispace1,
      factor
    )),
    |t| Logic::And(Box::new(t.0), Box::new(t.4))
  )(s)
}

fn or(s: &str) -> IResult<&str, Logic> {
  map(
    tuple((
      factor,
      multispace0,
      tuple((opt(char('\\')), opt(char('l')), char('o'), char('r'))),
      multispace1,
      factor
    )),
    |t| Logic::Or(Box::new(t.0), Box::new(t.4))
  )(s)
}

fn term(s: &str) -> IResult<&str, Logic> {
  alt((and, or, factor))(s)
}

pub fn expr(s: &str) -> IResult<&str, Logic> {
  map(
    tuple((
      opt(tuple((
        term,
        multispace0,
        tuple((opt(char('\\')), char('t'), char('o'))),
        multispace1,
      ))),
      term
    )),
    |(opt, e)| {
      match opt {
        Some(t) => Logic::To(Box::new(t.0), Box::new(e)),
        None => e
      }
    }
  )(s)
}

#[cfg(test)]
mod test {
  use super::*;
  use Logic::*;

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
      factor("\\lnot A").unwrap(),
      ("", Not(
        Box::new(Base('A'))
      ))
    );
  }

  #[test]
  fn test_and() {
    assert_eq!(
      and("A \\land B \\land C").unwrap(),
      ("", And(
        Box::new(Base('A')),
        Box::new(And(
          Box::new(Base('B')),
          Box::new(Base('C'))
        ))
      ))
    );
  }

  #[test]
  fn test_or() {
    assert_eq!(
      or("A \\lor B \\lor C").unwrap(),
      ("", Or(
        Box::new(Base('A')),
        Box::new(And(
          Box::new(Base('B')),
          Box::new(Base('C'))
        ))
      ))
    );
  }

  #[test]
  fn test_term() {
    assert_eq!(
      term("\\lnot A").unwrap(),
      ("", Not(
        Box::new(Base('A'))
      ))
    );
    assert_eq!(
      term("A \\land B").unwrap(),
      ("", And(
        Box::new(Base('A')),
        Box::new(Base('B'))
      ))
    );
    assert_eq!(
      term("A \\lor B").unwrap(),
      ("", Or(
        Box::new(Base('A')),
        Box::new(Base('B'))
      ))
    );
  }

  #[test]
  fn test_expr() {
    assert_eq!(
      expr("\\lnot A").unwrap(),
      ("", Not(
        Box::new(Base('A'))
      ))
    );
    assert_eq!(
      expr("(A \\lor B \\to C) \\to ((A \\to C) \\land (B \\to C))").unwrap(),
      ("", To(
        Box::new(To(
          Box::new(Or(
            Box::new(Base('A')),
            Box::new(Base('B'))
          )),
          Box::new(Base('C'))
        )),
        Box::new(And(
          Box::new(To(
            Box::new(Base('A')),
            Box::new(Base('C'))
          )),
          Box::new(To(
            Box::new(Base('B')),
            Box::new(Base('C'))
          ))
        ))
      ))
    )
  }
}