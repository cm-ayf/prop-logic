use std::collections::HashSet;

use super::ast::*;

#[derive(Debug)]
pub struct InferenceNode<'a> {
  conc: &'a Logic,
  axioms: HashSet<&'a Logic>,
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

impl<'a> InferenceNode<'a> {
  pub fn new(conc: &'a Logic) -> Self {
    InferenceNode {
      conc,
      axioms: HashSet::new(),
      inference: None
    }
  }

  pub fn solve(&mut self) -> Result<&Self, ()> {
    let mut axioms: Vec<_> = self.axioms.iter().cloned().collect();
    axioms.sort();

    for axiom in &axioms {
      if self.conc.eq(axiom) {
        return Ok(self.infer(Inference::Axiom));
      }
    }

    for axiom in &axioms {
      for logic in axiom.has(&self.conc) {
        if let Ok(_) = match logic {
          Logic::Cont => self.use_cont(),
          Logic::And(_, _) => self.use_and(logic),
          Logic::Or(left, right) => self.use_or(logic, left, right),
          Logic::To(left, _) => self.use_to(logic, left),
          _ => return Err(())
        } {
          return Ok(self)
        }
      }
    }

    if let Ok(_) = match self.conc {
      Logic::Base(_) => Err(()),
      Logic::Cont => self.solve_cont(),
      Logic::Not(logic) => self.solve_not(logic),
      Logic::And(left, right) => self.solve_and(left, right),
      Logic::Or(left, right) => self.solve_or(left, right),
      Logic::To(left, right) => self.solve_to(left, right),
    } {
      return Ok(self)
    }

    if let Some(Logic::Or(left, right)) = axioms.first() {
      if let Ok(_) = self.use_or(axioms.first().unwrap(), &left, &right) {
        return Ok(self);
      }
    }

    Err(())
  }

  fn use_cont(&mut self) -> Result<&Self, ()> {
    Ok(self.infer(Inference::UnaryInf(
      Box::new(Self {
        conc: &Logic::Cont,
        axioms: self.axioms.clone(),
        inference: Some(Inference::Axiom)
      })
    )))
  }

  fn use_and(&mut self, logic: &'a Logic) -> Result<&Self, ()> {
    let mut i = Self {
      conc: logic,
      axioms: self.axioms.clone(),
      inference: None
    };
    i.solve()?;

    Ok(self.infer(Inference::UnaryInf(
      Box::new(i)
    )))
  }

  fn use_or(&mut self, logic: &'a Logic, left: &'a Logic, right: &'a Logic) -> Result<&Self, ()> {
    let i0 = Self {
      conc: logic,
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
    i1.solve()?;

    let mut axioms = self.axioms.clone();
    axioms.insert(right);
    let mut i2 = Self {
      conc: self.conc,
      axioms,
      inference: None
    };
    i2.solve()?;

    Ok(self.infer(Inference::TrinaryInf(
      Box::new(i0),
      Box::new(i1),
      Box::new(i2)
    )))
  }

  fn use_to(&mut self, logic: &'a Logic, left: &'a Logic) -> Result<&Self, ()> {
    let mut i0 = Self {
      conc: left,
      axioms: self.axioms.clone(),
      inference: None
    };
    i0.solve()?;

    let mut i1 = Self {
      conc: logic,
      axioms: self.axioms.clone(),
      inference: None
    };
    i1.solve()?;

    Ok(self.infer(Inference::BinaryInf(
      Box::new(i0),
      Box::new(i1)
    )))
  }

  fn solve_cont(&mut self) -> Result<&Self, ()> {
    let mut axioms:Vec<_> = self.axioms.iter().collect();
    axioms.sort();

    for axiom in &axioms {
      for child in axiom.children() {
        if let Logic::Not(logic) = child {
          let mut i0 = Self {
            conc: logic,
            axioms: self.axioms.clone(),
            inference: None
          };

          if let Ok(_) = i0.solve() {
            let i1 = Self{
              conc: child,
              axioms: self.axioms.clone(),
              inference: Some(Inference::Axiom)
            };
            return Ok(self.infer(Inference::BinaryInf(
                Box::new(i0),
                Box::new(i1)
              )
            ))
          }
        }
      }
    }

    Err(())
  }

  fn solve_not(&mut self, logic: &'a Logic) -> Result<&Self, ()> {
    let mut axioms = self.axioms.clone();
    axioms.insert(logic);
    let mut i = Self {
      conc: &Logic::Cont,
      axioms,
      inference: None
    };
    i.solve()?;

    Ok(self.infer(Inference::UnaryInf(
      Box::new(i)
    )))
  }

  fn solve_and(&mut self, left: &'a Logic, right: &'a Logic) -> Result<&Self, ()> {
    let mut i0 = Self {
      conc: left,
      axioms: self.axioms.clone(),
      inference: None
    };
    i0.solve()?;

    let mut i1 = Self {
      conc: right,
      axioms: self.axioms.clone(),
      inference: None
    };
    i1.solve()?;

    Ok(self.infer(Inference::BinaryInf(
      Box::new(i0),
      Box::new(i1)
    )))
  }

  fn solve_or(&mut self, left: &'a Logic, right: &'a Logic) -> Result<&Self, ()> {
    for logic in [left, right] {
      let mut i = Self {
        conc: logic,
        axioms: self.axioms.clone(),
        inference: None
      };
  
      if let Ok(_) = i.solve() {
        return Ok(self.infer(Inference::UnaryInf(
          Box::new(i)
        )));
      }
    }

    Err(())
  }

  fn solve_to(&mut self, left: &'a Logic, right: &'a Logic) -> Result<&Self, ()> {
    let mut axioms = self.axioms.clone();
    axioms.insert(left);
    let mut i = Self {
      conc: right,
      axioms,
      inference: None
    };

    i.solve()?;

    Ok(self.infer(Inference::UnaryInf(
      Box::new(i)
    )))
  }

  fn infer(&mut self, inference: Inference<'a>) -> &Self {
    self.inference = Some(inference);
    self
  }
}
