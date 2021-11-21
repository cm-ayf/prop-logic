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
    if let Ok(_) = self.solve_common() {
      return Ok(self)
    }

    if let Ok(i) = match self.conc {
      Expr::Base(_) => Err(()),
      Expr::Cont => self.solve_cont(),
      Expr::Not(expr) => self.solve_not(expr),
      Expr::And(left, right) => self.solve_and(left, right),
      Expr::Or(left, right) => self.solve_or(left, right),
      Expr::To(left, right) => self.solve_to(left, right),
    } {
      return Ok(i)
    }

    Err(())
  }

  fn solve_common(&mut self) -> Result<&Self, ()> {
    for axiom in self.axioms.iter() {
      if self.conc.eq(axiom) {
        return Ok(self.infer(Inference::Axiom));
      }
    }

    for axiom in self.axioms.iter() {
      if let Some(conc) = axiom.has(self.conc) {
        match conc {
          Expr::Cont => return Ok(self.infer(Inference::UnaryInf(
            Box::new(InferenceNode {
              conc: &Expr::Cont,
              axioms: self.axioms.clone(),
              inference: Some(Inference::Axiom)
            })
          ))),
          Expr::And(_, _) => {
            let mut i = InferenceNode {
              conc,
              axioms: self.axioms.clone(),
              inference: None
            };
            if let Ok(_) = i.solve() {
              return Ok(self.infer(Inference::UnaryInf(
                Box::new(i)
              )));
            }
          },
          Expr::Or(left, right) => {
            let i0 = InferenceNode {
              conc,
              axioms: self.axioms.clone(),
              inference: Some(Inference::Axiom)
            };
            let mut axioms = self.axioms.clone();
            axioms.insert(left);
            let mut i1 = InferenceNode {
              conc,
              axioms,
              inference: None
            };
            let mut axioms = self.axioms.clone();
            axioms.insert(right);
            let mut i2 = InferenceNode {
              conc,
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
          },
          Expr::To(left, _) => {
            let mut i0 = InferenceNode {
              conc: left,
              axioms: self.axioms.clone(),
              inference: None
            };
            let mut i1 = InferenceNode {
              conc,
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
          },
          Expr::Not(expr) => {
            if self.conc.eq(expr) {
              return Err(())
            }
          },
          _ => return Err(())
        }
      }
    }
    Err(())
  }

  fn solve_cont(&mut self) -> Result<&Self, ()> {
    todo!()
  }

  fn solve_not(&mut self, left: &'a Expr) -> Result<&Self, ()> {
    let mut axioms = self.axioms.clone();
    axioms.insert(left);
    let mut i = InferenceNode {
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
    let mut i0 = InferenceNode {
      conc: left,
      axioms: self.axioms.clone(),
      inference: None
    };
    let mut i1 = InferenceNode {
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
    } else {
      Err(())
    }
  }

  fn solve_or(&mut self, left: &'a Expr, right: &'a Expr) -> Result<&Self, ()> {
    let mut i0 = InferenceNode {
      conc: left,
      axioms: self.axioms.clone(),
      inference: None
    };

    if let Ok(_) = i0.solve() {
      return Ok(self.infer(Inference::UnaryInf(
        Box::new(i0)
      )));
    }

    let mut i1 = InferenceNode {
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
    let mut i = InferenceNode {
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
