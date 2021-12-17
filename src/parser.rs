//! ```bnf
//! <base>  := A-Z
//! <cont>  := '\perp '
//! <paren> := '(' ws0 <parse> ws0 ')'
//! <term>  := <base> | <cont> | <paren> | <not>
//! <not>   := '\lnot ' ws0 ( <term> )
//! <and>   := <term> ws0 '\land ' ws0 ( <and> | <term> )
//! <or>    := <term> ws0 '\land ' ws0 ( <or> | <term> )
//! <to>    := ( <and> | <or> | <term> ) ws0 '\land ' ws0 <parse>
//! <parse> := <to> | <and> | <or> | <term>
//! ```

use nom::{
  branch::*, bytes::complete::*, character::complete::*, combinator::*, error::Error, sequence::*,
  Err, IResult,
};

use super::logic::*;

pub type ParseLogicError = Err<Error<String>>;

/// `<base> := A-Z`
fn base(s: &str) -> IResult<&str, Logic> {
  map(one_of("ABCDEFGHIJKLMNOPQRSTUVWXYZ"), |c| Logic::Base(c))(s)
}

/// `<cont> := '\perp '`
fn cont(s: &str) -> IResult<&str, Logic> {
  value(Logic::Cont, alt((tag("\\perp "), tag("cont"), tag("⊥"))))(s)
}

/// `<paren> := '(' ws0 <parse> ws0 ')'`
fn paren(s: &str) -> IResult<&str, Logic> {
  delimited(
    char('('),
    delimited(multispace0, parse, multispace0),
    char(')'),
  )(s)
}

/// `<term> := <base> | <cont> | <paren> | <not>`
fn term(s: &str) -> IResult<&str, Logic> {
  alt((base, cont, paren, not))(s)
}

/// `<not> := '\lnot ' ws0 ( <term> )`
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

/// `<and> := <term> ws0 '\land ' ws0 ( <and> | <term> )`
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

/// `<or> := <term> ws0 '\land ' ws0 ( <or> | <term> )`
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

/// `<to> := ( <and> | <or> | <term> ) ws0 '\land ' ws0 <parse>`
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

/// `<parse> := <to> | <and> | <or> | <term>`; entry point
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
