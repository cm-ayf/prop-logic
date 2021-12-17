use nom::{
  branch::*, bytes::complete::*, character::complete::*, combinator::*, error::Error, sequence::*,
  Err, IResult,
};

use super::logic::*;

pub type ParseLogicError = Err<Error<String>>;

/*
  <to>      := ( <and> | <or> ) [ ws0 '\to ' ws0 <to> ]
  <and>     := <not> [ ws0 '\land ' ws0 <and> ]
  <or>      := <not> [ ws0 '\lor ' ws0 <or> ]
  <not>     := [ '\lnot ' ws0 ] ( <base> | <cont> | <paren> | <not> )
  <paren>   := '(' ws0 <to> ws0 ')'
  <cont>    := '\perp '
  <base>    := A-Z
*/

fn base(s: &str) -> IResult<&str, Logic> {
  map(one_of("ABCDEFGHIJKLMNOPQRSTUVWXYZ"), |c| Logic::Base(c))(s)
}

fn cont(s: &str) -> IResult<&str, Logic> {
  value(Logic::Cont, alt((tag("\\perp "), tag("cont"), tag("⊥"))))(s)
}

fn paren(s: &str) -> IResult<&str, Logic> {
  delimited(
    char('('),
    delimited(multispace0, parse, multispace0),
    char(')'),
  )(s)
}

fn term(s: &str) -> IResult<&str, Logic> {
  alt((base, cont, paren, not))(s)
}

fn not(s: &str) -> IResult<&str, Logic> {
  map(
    tuple((
      alt((tag("\\lnot "), tag("not"), tag("¬"))),
      multispace0,
      term,
    )),
    |t| Logic::Not(Box::new(t.2)),
  )(s)
}

fn and(s: &str) -> IResult<&str, Logic> {
  map(
    tuple((
      term,
      multispace0,
      alt((tag("\\land "), tag("and"), tag("∧"))),
      multispace0,
      alt((and, term)),
    )),
    |t| Logic::And(Box::new(t.0), Box::new(t.4)),
  )(s)
}

fn or(s: &str) -> IResult<&str, Logic> {
  map(
    tuple((
      term,
      multispace0,
      alt((tag("\\lor "), tag("or"), tag("∨"))),
      multispace0,
      alt((or, term)),
    )),
    |t| Logic::Or(Box::new(t.0), Box::new(t.4)),
  )(s)
}

fn to(s: &str) -> IResult<&str, Logic> {
  map(
    tuple((
      alt((and, or, term)),
      multispace0,
      alt((tag("\\to "), tag("to"), tag("→"))),
      multispace0,
      parse,
    )),
    |t| Logic::To(Box::new(t.0), Box::new(t.4)),
  )(s)
}

pub fn parse(s: &str) -> IResult<&str, Logic> {
  alt((to, and, or, term))(s)
}

#[cfg(test)]
mod test {
  use super::*;
  use Logic::*;

  #[test]
  fn test_base() {
    assert_eq!(base("A").unwrap(), ("", Base('A')));
  }

  #[test]
  fn test_paren() {
    assert_eq!(paren("(A)").unwrap(), ("", Base('A')));
  }

  #[test]
  fn test_not() {
    assert_eq!(not("\\lnot A").unwrap(), ("", Not(Box::new(Base('A')))));
  }

  #[test]
  fn test_and() {
    assert_eq!(
      and("A \\land B").unwrap(),
      ("", And(Box::new(Base('A')), Box::new(Base('B'))))
    );
  }

  #[test]
  fn test_or() {
    assert_eq!(
      or("A \\lor B").unwrap(),
      ("", Or(Box::new(Base('A')), Box::new(Base('B'))))
    );
  }

  #[test]
  fn test_to() {
    assert_eq!(
      to("A \\to B").unwrap(),
      ("", To(Box::new(Base('A')), Box::new(Base('B'))))
    );
  }

  #[test]
  fn test_parse() {
    assert_eq!(
      parse("(A \\lor B \\to C) \\to ((A \\to C) \\land (B \\to C))").unwrap(),
      (
        "",
        To(
          Box::new(To(
            Box::new(Or(Box::new(Base('A')), Box::new(Base('B')))),
            Box::new(Base('C'))
          )),
          Box::new(And(
            Box::new(To(Box::new(Base('A')), Box::new(Base('C')))),
            Box::new(To(Box::new(Base('B')), Box::new(Base('C'))))
          ))
        )
      )
    );
    assert_eq!(
      parse("A to not not A").unwrap(),
      (
        "",
        To(
          Box::new(Base('A')),
          Box::new(Not(Box::new(Not(Box::new(Base('A'))))))
        )
      )
    );
  }
}
