use std::fmt::Display;
use std::{collections::HashSet, error::Error};

use super::{logic::*, TeX};

#[derive(Debug, Clone)]
pub struct Inference<'a> {
  logic: &'a Logic,
  axioms: HashSet<&'a Logic>,
  inference: Option<InferenceType<'a>>,
}

#[derive(Debug, Clone)]
enum InferenceType<'a> {
  Axiom,
  UnaryInf(Box<Inference<'a>>),
  BinaryInf(Box<Inference<'a>>, Box<Inference<'a>>),
  TrinaryInf(Box<Inference<'a>>, Box<Inference<'a>>, Box<Inference<'a>>),
}

impl<'a> Inference<'a> {
  pub fn new(logic: &'a Logic) -> Self {
    Self {
      logic,
      axioms: HashSet::new(),
      inference: None,
    }
  }

  fn problem(&self, logic: &'a Logic) -> Self {
    Self {
      logic,
      axioms: self.axioms.clone(),
      inference: None,
    }
  }

  fn err(&self) -> Result<(), SolveError> {
    Err(SolveError {
      logic: self.logic.clone(),
    })
  }

  fn infer(&mut self, inference: InferenceType<'a>) {
    self.inference = Some(inference);
  }

  pub fn solve(&mut self) -> Result<(), SolveError> {
    let mut axioms: Vec<_> = self.axioms.iter().cloned().collect();
    axioms.sort();

    for axiom in &axioms {
      let mut i = self.problem(axiom);
      i.infer(InferenceType::Axiom);
      if let Ok(_) = self.use_logic(i) {
        return Ok(());
      }
    }

    if let Ok(_) = self.infer_logic() {
      return Ok(());
    }

    self.err()
  }

  fn use_logic(&mut self, i: Self) -> Result<(), SolveError> {
    if self.logic.eq(i.logic) {
      *self = i;
      return Ok(());
    }

    match i.logic {
      Logic::Cont => self.use_cont(i),
      Logic::Not(logic) => self.use_not(i, logic),
      Logic::And(left, right) => self.use_and(i, left, right),
      Logic::Or(left, right) => self.use_or(i, left, right),
      Logic::To(left, right) => self.use_to(i, left, right),
      _ => self.err(),
    }
  }

  fn use_cont(&mut self, i: Self) -> Result<(), SolveError> {
    let mut i0 = self.problem(i.logic);
    i0.infer(InferenceType::Axiom);

    self.infer(InferenceType::UnaryInf(Box::new(i0)));
    Ok(())
  }

  fn use_not(&mut self, i1: Self, logic: &'a Logic) -> Result<(), SolveError> {
    let mut i0 = self.problem(logic);
    i0.solve()?;

    let mut i = self.problem(&Logic::Cont);
    i.infer(InferenceType::BinaryInf(Box::new(i0), Box::new(i1)));
    self.use_cont(i)
  }

  fn use_and(&mut self, i: Self, left: &'a Logic, right: &'a Logic) -> Result<(), SolveError> {
    for logic in [left, right] {
      let mut i0 = self.problem(logic);
      i0.infer(InferenceType::UnaryInf(Box::new(i.clone())));

      if let Ok(_) = self.use_logic(i0) {
        return Ok(());
      }
    }

    self.err()
  }

  fn use_or(&mut self, i0: Self, left: &'a Logic, right: &'a Logic) -> Result<(), SolveError> {
    let mut i1 = self.problem(self.logic);
    i1.axioms.insert(left);
    i1.infer_logic()?;

    let mut i2 = self.problem(self.logic);
    i2.axioms.insert(right);
    i2.infer_logic()?;

    self.infer(InferenceType::TrinaryInf(
      Box::new(i0),
      Box::new(i1),
      Box::new(i2),
    ));
    Ok(())
  }

  fn use_to(&mut self, i1: Self, left: &'a Logic, right: &'a Logic) -> Result<(), SolveError> {
    let mut i0 = self.problem(left);
    i0.solve()?;

    let mut i = self.problem(right);
    i.infer(InferenceType::BinaryInf(Box::new(i0), Box::new(i1)));

    self.use_logic(i)
  }

  fn infer_logic(&mut self) -> Result<(), SolveError> {
    match self.logic {
      Logic::Not(logic) => self.infer_not(logic),
      Logic::And(left, right) => self.infer_and(left, right),
      Logic::Or(left, right) => self.infer_or(left, right),
      Logic::To(left, right) => self.infer_to(left, right),
      _ => self.err(),
    }
  }

  fn infer_not(&mut self, logic: &'a Logic) -> Result<(), SolveError> {
    let mut i = self.problem(&Logic::Cont);
    i.axioms.insert(logic);
    i.solve()?;

    self.infer(InferenceType::UnaryInf(Box::new(i)));
    Ok(())
  }

  fn infer_and(&mut self, left: &'a Logic, right: &'a Logic) -> Result<(), SolveError> {
    let mut i0 = self.problem(left);
    i0.solve()?;

    let mut i1 = self.problem(right);
    i1.solve()?;

    self.infer(InferenceType::BinaryInf(Box::new(i0), Box::new(i1)));
    Ok(())
  }

  fn infer_or(&mut self, left: &'a Logic, right: &'a Logic) -> Result<(), SolveError> {
    for logic in [left, right] {
      let mut i = self.problem(logic);

      if let Ok(_) = i.solve() {
        self.infer(InferenceType::UnaryInf(Box::new(i)));
        return Ok(());
      }
    }

    self.err()
  }

  fn infer_to(&mut self, left: &'a Logic, right: &'a Logic) -> Result<(), SolveError> {
    let mut i = self.problem(right);
    i.axioms.insert(left);
    i.solve()?;

    self.infer(InferenceType::UnaryInf(Box::new(i)));
    Ok(())
  }

  fn print(&self, tree: &mut String, indent: &str) {
    tree.push_str(&format!("{}\n", self.logic));
    match self.inference {
      None | Some(InferenceType::Axiom) => {}
      Some(InferenceType::UnaryInf(ref i0)) => {
        tree.push_str(&format!("{}+ ", indent));
        i0.print(tree, &format!("{}  ", indent));
      }
      Some(InferenceType::BinaryInf(ref i0, ref i1)) => {
        tree.push_str(&format!("{}+ ", indent));
        i0.print(tree, &format!("{}| ", indent));
        tree.push_str(&format!("{}+ ", indent));
        i1.print(tree, &format!("{}  ", indent));
      }
      Some(InferenceType::TrinaryInf(ref i0, ref i1, ref i2)) => {
        tree.push_str(&format!("{}+ ", indent));
        i0.print(tree, &format!("{}| ", indent));
        tree.push_str(&format!("{}+ ", indent));
        i1.print(tree, &format!("{}| ", indent));
        tree.push_str(&format!("{}+ ", indent));
        i2.print(tree, &format!("{}  ", indent));
      }
    }
  }
}

impl TeX for Inference<'_> {
  fn tex(&self) -> String {
    match self.inference {
      None => format!("{}", self.logic.tex()),
      Some(InferenceType::Axiom) => format!("\\AxiomC{{${}$}}", self.logic.tex()),
      Some(InferenceType::UnaryInf(ref i0)) => {
        format!("{}\n\\UnaryInfC{{${}$}}", i0.tex(), self.logic.tex())
      }
      Some(InferenceType::BinaryInf(ref i0, ref i1)) => format!(
        "{}\n{}\n\\BinaryInfC{{${}$}}",
        i0.tex(),
        i1.tex(),
        self.logic.tex()
      ),
      Some(InferenceType::TrinaryInf(ref i0, ref i1, ref i2)) => format!(
        "{}\n{}\n{}\n\\TrinaryInfC{{${}$}}",
        i0.tex(),
        i1.tex(),
        i2.tex(),
        self.logic.tex()
      ),
    }
  }
}

impl Display for Inference<'_> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut tree = String::new();
    self.print(&mut tree, "");
    write!(f, "{}", tree)
  }
}

#[derive(Debug)]
pub struct SolveError {
  logic: Logic,
}

impl Display for SolveError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "could not solve: {}", self.logic)
  }
}

impl Error for SolveError {}
