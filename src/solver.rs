use std::collections::HashSet;

use super::ast::*;

#[derive(Debug)]
pub struct Inference<'a> {
  conc: &'a Logic,
  axioms: HashSet<&'a Logic>,
  inference: Option<InferenceType<'a>>
}

#[derive(Debug)]
enum InferenceType<'a> {
  Axiom,
  UnaryInf(
    Box<Inference<'a>>
  ),
  BinaryInf(
    Box<Inference<'a>>,
    Box<Inference<'a>>
  ),
  TrinaryInf(
    Box<Inference<'a>>,
    Box<Inference<'a>>,
    Box<Inference<'a>>
  )
}

impl<'a> Inference<'a> {
  pub fn new(conc: &'a Logic) -> Self {
    Self {
      conc,
      axioms: HashSet::new(),
      inference: None
    }
  }

  pub fn solve(&mut self) -> Result<&Self, ()> {
    println!("solving {}", self.conc);

    let mut axioms: Vec<_> = self.axioms.iter().cloned().collect();
    axioms.sort();

    for axiom in &axioms {
      if self.conc.eq(axiom) {
        return Ok(self.infer(InferenceType::Axiom));
      }
    }

    println!("no axiom {}", self.conc);

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

    println!("no has's {}", self.conc);

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

    println!("use or   {}", self.conc);

    if let Some(Logic::Or(left, right)) = axioms.first() {
      if let Ok(_) = self.use_or(axioms.first().ok_or(())?, &left, &right) {
        return Ok(self);
      }
    }

    Err(())
  }

  fn use_cont(&mut self) -> Result<&Self, ()> {
    Ok(self.infer(InferenceType::UnaryInf(
      Box::new(Self {
        conc: &Logic::Cont,
        axioms: self.axioms.clone(),
        inference: Some(InferenceType::Axiom)
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

    Ok(self.infer(InferenceType::UnaryInf(
      Box::new(i)
    )))
  }

  fn use_or(&mut self, logic: &'a Logic, left: &'a Logic, right: &'a Logic) -> Result<&Self, ()> {
    let i0 = Self {
      conc: logic,
      axioms: self.axioms.clone(),
      inference: Some(InferenceType::Axiom)
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

    Ok(self.infer(InferenceType::TrinaryInf(
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

    Ok(self.infer(InferenceType::BinaryInf(
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
              inference: Some(InferenceType::Axiom)
            };
            return Ok(self.infer(InferenceType::BinaryInf(
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

    Ok(self.infer(InferenceType::UnaryInf(
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

    Ok(self.infer(InferenceType::BinaryInf(
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
        return Ok(self.infer(InferenceType::UnaryInf(
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

    Ok(self.infer(InferenceType::UnaryInf(
      Box::new(i)
    )))
  }

  fn infer(&mut self, inference: InferenceType<'a>) -> &Self {
    self.inference = Some(inference);
    self
  }
}
