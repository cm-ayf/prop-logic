use std::collections::HashSet;

use super::ast::*;

#[derive(Debug)]
pub struct InferenceNode<'a> {
  conc: &'a Expr,
  axioms: HashSet<&'a Expr>,
  inference: Option<Inference<'a>>
}

#[derive(Debug)]
enum Inference<'a> {
  Axiom,
  UnaryInf(
    Box<InferenceNode<'a>>
  ),
  BinaryInf(
    Box<InferenceNode<'a>>,
    Box<InferenceNode<'a>>
  ),
  TrinaryInf(
    Box<InferenceNode<'a>>,
    Box<InferenceNode<'a>>,
    Box<InferenceNode<'a>>
  )
}

pub fn solver<'a>(expr: &'a Expr) -> Result<InferenceNode<'a>, ()> {
  let mut i = InferenceNode {
    conc: expr,
    axioms: HashSet::new(),
    inference: None
  };
  i.solve()?;
  Ok(i)
}

impl<'a> InferenceNode<'a> {
  fn solve(&mut self) -> Result<&Self, ()> {
    let mut axioms: Vec<_> = self.axioms.iter().cloned().collect();
    axioms.sort();

    for axiom in &axioms {
      if self.conc.eq(axiom) {
        return Ok(self.infer(Inference::Axiom));
      }
    }

    for axiom in &axioms {
      if let Some(expr) = axiom.has(&self.conc) {
        if let Ok(_) = match expr {
          Expr::Cont => self.use_cont(),
          Expr::And(_, _) => self.use_and(expr),
          Expr::Or(left, right) => self.use_or(expr, left, right),
          Expr::To(left, _) => self.use_to(expr, left),
          _ => return Err(())
        } {
          return Ok(self)
        }
      }
    }

    if let Ok(_) = match self.conc {
      Expr::Base(_) => Err(()),
      Expr::Cont => self.solve_cont(),
      Expr::Not(expr) => self.solve_not(expr),
      Expr::And(left, right) => self.solve_and(left, right),
      Expr::Or(left, right) => self.solve_or(left, right),
      Expr::To(left, right) => self.solve_to(left, right),
    } {
      return Ok(self)
    }

    if let Some(Expr::Or(left, right)) = axioms.first() {
      if let Ok(_) = self.use_or(axioms.first().unwrap(), &left, &right) {
        return Ok(self);
      }
    }

    Err(())
  }

  fn use_cont(&mut self) -> Result<&Self, ()> {
    Ok(self.infer(Inference::UnaryInf(
      Box::new(Self {
        conc: &Expr::Cont,
        axioms: self.axioms.clone(),
        inference: Some(Inference::Axiom)
      })
    )))
  }

  fn use_and(&mut self, expr: &'a Expr) -> Result<&Self, ()> {
    let mut i = Self {
      conc: expr,
      axioms: self.axioms.clone(),
      inference: None
    };
    if let Ok(_) = i.solve() {
      return Ok(self.infer(Inference::UnaryInf(
        Box::new(i)
      )));
    }

    Err(())
  }

  fn use_or(&mut self, expr: &'a Expr, left: &'a Expr, right: &'a Expr) -> Result<&Self, ()> {
    let i0 = Self {
      conc: expr,
      axioms: self.axioms.clone(),
      inference: Some(Inference::Axiom)
    };
    let mut axioms = self.axioms.clone();
    axioms.insert(left);
    let mut i1 = Self {
      conc: self.conc,
      axioms,
      inference: None
    };
    let mut axioms = self.axioms.clone();
    axioms.insert(right);
    let mut i2 = Self {
      conc: self.conc,
      axioms,
      inference: None
    };

    if let (Ok(_), Ok(_))
      = (i1.solve(), i2.solve()) {
      return Ok(self.infer(Inference::TrinaryInf(
        Box::new(i0),
        Box::new(i1),
        Box::new(i2)
      )));
    }

    Err(())
  }

  fn use_to(&mut self, expr: &'a Expr, left: &'a Expr) -> Result<&Self, ()> {
    let mut i0 = Self {
      conc: left,
      axioms: self.axioms.clone(),
      inference: None
    };
    let mut i1 = Self {
      conc: expr,
      axioms: self.axioms.clone(),
      inference: None
    };
    if let (Ok(_), Ok(_))
      = (i0.solve(), i1.solve()) {
      return Ok(self.infer(Inference::BinaryInf(
        Box::new(i0),
        Box::new(i1)
      )))
    }

    Err(())
  }

  fn solve_cont(&mut self) -> Result<&Self, ()> {
    todo!()
  }

  fn solve_not(&mut self, left: &'a Expr) -> Result<&Self, ()> {
    let mut axioms = self.axioms.clone();
    axioms.insert(left);
    let mut i = Self {
      conc: &Expr::Cont,
      axioms,
      inference: None
    };

    if let Ok(_) = i.solve() {
      return Ok(self.infer(Inference::UnaryInf(
        Box::new(i)
      )));
    }

    Err(())
  }

  fn solve_and(&mut self, left: &'a Expr, right: &'a Expr) -> Result<&Self, ()> {
    let mut i0 = Self {
      conc: left,
      axioms: self.axioms.clone(),
      inference: None
    };
    let mut i1 = Self {
      conc: right,
      axioms: self.axioms.clone(),
      inference: None
    };

    if let (Ok(_), Ok(_))
      = (i0.solve(), i1.solve()) {
      return Ok(self.infer(Inference::BinaryInf(
        Box::new(i0),
        Box::new(i1)
      )))
    }

    Err(())
  }

  fn solve_or(&mut self, left: &'a Expr, right: &'a Expr) -> Result<&Self, ()> {
    let mut i0 = Self {
      conc: left,
      axioms: self.axioms.clone(),
      inference: None
    };

    if let Ok(_) = i0.solve() {
      return Ok(self.infer(Inference::UnaryInf(
        Box::new(i0)
      )));
    }

    let mut i1 = Self {
      conc: right,
      axioms: self.axioms.clone(),
      inference: None
    };

    if let Ok(_) = i1.solve() {
      return Ok(self.infer(Inference::UnaryInf(
        Box::new(i1)
      )));
    }

    Err(())
  }

  fn solve_to(&mut self, left: &'a Expr, right: &'a Expr) -> Result<&Self, ()> {
    let mut axioms = self.axioms.clone();
    axioms.insert(left);
    let mut i = Self {
      conc: right,
      axioms,
      inference: None
    };

    if let Ok(_) = i.solve() {
      return Ok(self.infer(Inference::UnaryInf(
        Box::new(i)
      )));
    }

    Err(())
  }

  fn infer(&mut self, inference: Inference<'a>) -> &Self {
    self.inference = Some(inference);
    self
  }
}
