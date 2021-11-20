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
  <term>    := <factor> | <and> | <or>
  <and>     := <factor> [ ws ] '\land' ws ( <factor> | <and> )
  <or>      := <factor> [ ws ] '\lor' ws ( <factor> | <or> )
  <factor>  := [ '\lnot' ws ] ( <base> | <paren> )
  <paren>   := '(' [ ws ] <expr> [ ws ] ')'
  <base>    := A-Z
*/

const BASE: &'static str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";

fn base(s: &str) -> IResult<&str, Expr> {
  map(
    one_of(BASE),
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
        Some(_) => Expr::UnaryOp{
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
      and
    )),
    |t| Expr::BinaryOp{
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
      or
    )),
    |t| Expr::BinaryOp{
      op: BinaryOpKind::Or,
      left: Box::new(t.0),
      right: Box::new(t.4)
    }
  )(s)
}

fn term(s: &str) -> IResult<&str, Expr> {
  alt((factor, and, or))(s)
}


pub fn expr(s: &str) -> IResult<&str, Expr> {
  map(
    tuple((
      term,
      opt(tuple((
        multispace0,
        tuple((char('\\'), char('t'), char('o'))),
        multispace1,
        term
      )))
    )),
    |(e, opt)| {
      match opt {
        Some(t) => Expr::BinaryOp{
          op: BinaryOpKind::To,
          left: Box::new(e),
          right: Box::new(t.3)
        },
        None => e
      }
    }
  )(s)
}

