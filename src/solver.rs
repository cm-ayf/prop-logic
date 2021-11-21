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
      Expr::Not(e0) => self.solve_not(e0),
      Expr::And(e0, e1) => self.solve_and(e0, e1),
      Expr::Or(e0, e1) => self.solve_or(e0, e1),
      Expr::To(e0, e1) => self.solve_to(e0, e1),
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
          Expr::Not(_) => todo!(),
          _ => return Err(())
        }
      }
    }
    Err(())
  }

  fn solve_cont(&mut self) -> Result<&Self, ()> {
    todo!()
  }

  fn solve_not(&mut self, e0: &'a Expr) -> Result<&Self, ()> {
    let mut axioms = self.axioms.clone();
    axioms.insert(e0);
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

  fn solve_and(&mut self, e0: &'a Expr, e1: &'a Expr) -> Result<&Self, ()> {
    let mut i0 = InferenceNode {
      conc: e0,
      axioms: self.axioms.clone(),
      inference: None
    };
    let mut i1 = InferenceNode {
      conc: e1,
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

  fn solve_or(&mut self, e0: &'a Expr, e1: &'a Expr) -> Result<&Self, ()> {
    let mut i0 = InferenceNode {
      conc: e0,
      axioms: self.axioms.clone(),
      inference: None
    };

    if let Ok(_) = i0.solve() {
      return Ok(self.infer(Inference::UnaryInf(
        Box::new(i0)
      )));
    }

    let mut i1 = InferenceNode {
      conc: e1,
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

  fn solve_to(&mut self, e0: &'a Expr, e1: &'a Expr) -> Result<&Self, ()> {
    let mut axioms = self.axioms.clone();
    axioms.insert(e0);
    let mut i = InferenceNode {
      conc: e1,
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
