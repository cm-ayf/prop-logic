//! 文字列を解析し，論理式の木を出力する具体的な実装を行うモジュールです．[nom]パッケージを利用しています．
//! 詳しくは[公式ドキュメント](https://docs.rs/nom/7.1.0/nom/)を参照してください．

use nom::{branch::*, character::complete::*, combinator::*, Err, error::Error, sequence::*, IResult};

use super::logic::*;

pub type ParseLogicError = Err<Error<String>>;

/*
  <expr>    := <term> [ [ ws ] '\to' ws <term>]
  <term>    := <factor> [ [ ws ] '\land'|'\lor' ws <factor> ]
  <factor>  := [ '\lnot' ws ] ( <base> | <paren> )
  <paren>   := '(' [ ws ] <expr> [ ws ] ')'
  <base>    := A-Z
*/

/// 原子式をパースします．
fn base(s: &str) -> IResult<&str, Logic> {
  map(one_of("ABCDEFGHIJKLMNOPQRSTUVWXYZ"), |c| Logic::Base(c))(s)
}

/// かっこを含む式をパースします．
fn paren(s: &str) -> IResult<&str, Logic> {
  delimited(
    char('('),
    delimited(multispace0, expr, multispace0),
    char(')'),
  )(s)
}

/// 否定を含む式をパースします．
fn factor(s: &str) -> IResult<&str, Logic> {
  map(
    tuple((
      opt(tuple((
        tuple((
          opt(tuple((char('\\'), char('l')))),
          char('n'),
          char('o'),
          char('t'),
        )),
        multispace1,
      ))),
      alt((base, paren, factor)),
    )),
    |(opt, e)| match opt {
      Some(_) => Logic::Not(Box::new(e)),
      None => e,
    },
  )(s)
}

/// 論理積・論理和を含む式をパースします．
fn term(s: &str) -> IResult<&str, Logic> {
  map(
    tuple((
      factor,
      opt(tuple((
        multispace0,
        opt(tuple((char('\\'), char('l')))),
        alt((
          map(tuple((char('a'), char('n'), char('d'))), |_| true),
          map(tuple((char('o'), char('r'))), |_| false),
        )),
        multispace1,
        factor,
      ))),
    )),
    |(f0, opt)| match opt {
      Some((_, _, t, _, f1)) => match t {
        true => Logic::And(Box::new(f0), Box::new(f1)),
        false => Logic::Or(Box::new(f0), Box::new(f1)),
      },
      None => f0,
    },
  )(s)
}

/// 論理包含を含む式をパースします．
pub fn expr(s: &str) -> IResult<&str, Logic> {
  map(
    tuple((
      opt(tuple((
        term,
        multispace0,
        tuple((opt(char('\\')), char('t'), char('o'))),
        multispace1,
      ))),
      term,
    )),
    |(opt, e)| match opt {
      Some(t) => Logic::To(Box::new(t.0), Box::new(e)),
      None => e,
    },
  )(s)
}

#[cfg(test)]
mod test {
  //! テストを行うサブモジュールです．

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
  fn test_factor() {
    assert_eq!(factor("A").unwrap(), ("", Base('A')));
    assert_eq!(factor("\\lnot A").unwrap(), ("", Not(Box::new(Base('A')))));
  }

  #[test]
  fn test_term() {
    assert_eq!(term("\\lnot A").unwrap(), ("", Not(Box::new(Base('A')))));
    assert_eq!(
      term("A \\land B").unwrap(),
      ("", And(Box::new(Base('A')), Box::new(Base('B'))))
    );
    assert_eq!(
      term("A \\lor B").unwrap(),
      ("", Or(Box::new(Base('A')), Box::new(Base('B'))))
    );
  }

  #[test]
  fn test_expr() {
    assert_eq!(expr("\\lnot A").unwrap(), ("", Not(Box::new(Base('A')))));
    assert_eq!(
      expr("(A \\lor B \\to C) \\to ((A \\to C) \\land (B \\to C))").unwrap(),
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
    )
  }
}
