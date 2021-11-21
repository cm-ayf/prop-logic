use std::collections::HashSet;

use super::ast::*;

pub struct Problem<'a> {
  conc: &'a Expr,
  axioms: HashSet<&'a Expr>
}

#[derive(Debug)]
pub struct InferenceNode<'a> {
  conc: &'a Expr,
  axioms: HashSet<&'a Expr>,
  inference: Inference<'a>
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
  let problem =  Problem {
    conc: expr,
    axioms: HashSet::new()
  };
  problem.solve()
}

impl<'a> Problem<'a> {
  fn solve(&self) -> Result<InferenceNode<'a>, ()> {
    if let Ok(i) = self.solve_common() {
      return Ok(i)
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

  fn solve_common(&self) -> Result<InferenceNode<'a>, ()> {
    for axiom in self.axioms.iter() {
      if self.conc.eq(axiom) {
        return Ok(InferenceNode {
          conc: self.conc,
          axioms: self.axioms.clone(),
          inference: Inference::Axiom
        });
      }
    }

    for axiom in self.axioms.iter() {
      if let Some(conc) = axiom.has(self.conc) {
        match conc {
          Expr::Cont => return Ok(self.infer(Inference::UnaryInf(
            Box::new(InferenceNode {
              conc: &Expr::Cont,
              axioms: self.axioms.clone(),
              inference: Inference::Axiom
            })
          ))),
          Expr::And(_, _) => {
            let p = Problem {
              conc,
              axioms: self.axioms.clone()
            };
            match p.solve() {
              Ok(i) => return Ok(self.infer(
                Inference::UnaryInf(Box::new(i))
              )),
              Err(()) => continue
            }
          },
          Expr::Or(left, right) => {
            let p0 = Problem {
              conc,
              axioms: self.axioms.clone()
            };
            let mut axioms = self.axioms.clone();
            axioms.insert(left);
            let p1 = Problem {
              conc,
              axioms
            };
            let mut axioms = self.axioms.clone();
            axioms.insert(right);
            let p2 = Problem {
              conc,
              axioms
            };

            if let (Ok(i0), Ok(i1), Ok(i2))
              = (p0.solve(), p1.solve(), p2.solve()) {
              return Ok(self.infer(Inference::TrinaryInf(
                Box::new(i0),
                Box::new(i1),
                Box::new(i2)
              )));
            }
          },
          Expr::To(left, _) => {
            let p0 = Problem {
              conc: left,
              axioms: self.axioms.clone()
            };
            let p1 = Problem {
              conc,
              axioms: self.axioms.clone()
            };
            if let (Ok(i0), Ok(i1))
              = (p0.solve(), p1.solve()) {
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

  fn solve_cont(&self) -> Result<InferenceNode<'a>, ()> {
    todo!()
  }

  fn solve_not(&self, e0: &'a Expr) -> Result<InferenceNode<'a>, ()> {
    let mut axioms = self.axioms.clone();
    axioms.insert(e0);
    let p = Problem {
      conc: &Expr::Cont,
      axioms
    };

    p.solve().map(|i| self.infer(Inference::UnaryInf(
      Box::new(i)
    )))
  }

  fn solve_and(&self, e0: &'a Expr, e1: &'a Expr) -> Result<InferenceNode<'a>, ()> {
    let p0 = Problem {
      conc: e0,
      axioms: self.axioms.clone()
    };
    let p1 = Problem {
      conc: e1,
      axioms: self.axioms.clone()
    };

    if let (Ok(i0), Ok(i1))
      = (p0.solve(), p1.solve()) {
      return Ok(self.infer(Inference::BinaryInf(
        Box::new(i0),
        Box::new(i1)
      )))
    } else {
      Err(())
    }
  }

  fn solve_or(&self, e0: &'a Expr, e1: &'a Expr) -> Result<InferenceNode<'a>, ()> {
    let p0 = Problem {
      conc: e0,
      axioms: self.axioms.clone()
    };

    if let Ok(i0) = p0.solve() {
      return Ok(self.infer(Inference::UnaryInf(
        Box::new(i0)
      )));
    }

    let p1 = Problem {
      conc: e1,
      axioms: self.axioms.clone()
    };

    if let Ok(i1) = p1.solve() {
      return Ok(self.infer(Inference::UnaryInf(
        Box::new(i1)
      )));
    }

    Err(())
  }

  fn solve_to(&self, e0: &'a Expr, e1: &'a Expr) -> Result<InferenceNode<'a>, ()> {
    let mut axioms = self.axioms.clone();
    axioms.insert(e0);
    let p = Problem {
      conc: e1,
      axioms
    };

    p.solve().map(|i| self.infer(Inference::UnaryInf(
      Box::new(i)
    )))
  }

  fn infer(&self, inference: Inference<'a>) -> InferenceNode<'a> {
    InferenceNode {
      conc: self.conc,
      axioms: self.axioms.clone(),
      inference
    }
  }
}
