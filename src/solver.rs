use std::cell::RefCell;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;
use std::rc::*;

use super::{logic::*, TeX};

#[derive(Debug, Clone)]
pub struct Inference<'a> {
  logic: &'a Logic,
  axioms: HashMap<&'a Logic, Rc<RefCell<usize>>>,
  marker: Rc<RefCell<usize>>,
  inference: Option<InferenceType<'a>>,
}

#[derive(Debug, Clone)]
enum InferenceType<'a> {
  Axiom(Weak<RefCell<usize>>),
  UnaryInf(Box<Inference<'a>>),
  BinaryInf(Box<Inference<'a>>, Box<Inference<'a>>),
  TrinaryInf(Box<Inference<'a>>, Box<Inference<'a>>, Box<Inference<'a>>),
}

impl<'a> Inference<'a> {
  pub fn new(logic: &'a Logic) -> Self {
    Self {
      logic,
      axioms: HashMap::new(),
      marker: Rc::new(RefCell::new(1)),
      inference: None,
    }
  }

  fn problem(&self, logic: &'a Logic) -> Self {
    Self {
      logic,
      axioms: self.axioms.clone(),
      marker: Rc::new(RefCell::new(0)),
      inference: None,
    }
  }

  fn err(&self) -> Result<(), SolveError> {
    Err(SolveError::InferError(self.logic.clone()))
  }

  fn infer(&mut self, inference: InferenceType<'a>) {
    self.inference = Some(inference);
  }

  pub fn solve(&mut self) -> Result<(), SolveError> {
    self.validate()?;

    if let Ok(_) = self.use_axioms() {
      return Ok(());
    }

    if let Ok(_) = self.infer_logic() {
      return Ok(());
    }

    self.err()
  }

  fn validate(&self) -> Result<(), SolveError> {
    let mut logic = Logic::Not(Box::new(self.logic.clone()));

    for (axiom, _) in self.axioms.clone() {
      logic = Logic::And(Box::new(logic), Box::new(axiom.clone()));
    }

    logic = Logic::Not(Box::new(logic));

    match logic.check_all() {
      Ok(_) => Ok(()),
      Err(_) => self.err()
    }
  }

  fn use_axioms(&mut self) -> Result<(), SolveError> {
    let axioms = self.axioms.clone();
    let mut axioms: Vec<_> = axioms.iter().collect();
    axioms.sort();
    
    for (axiom, marker) in axioms {
      let mut i = self.problem(axiom);
      i.infer(InferenceType::Axiom(Rc::downgrade(marker)));
      if let Ok(_) = self.use_logic(i) {
        return Ok(());
      }
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
      Logic::Not(logic) if self.logic.ne(logic) => self.use_not(i, logic),
      Logic::And(left, right) => self.use_and(i, left, right),
      Logic::Or(left, right) => self.use_or(i, left, right),
      Logic::To(left, right) if self.logic.ne(left) => self.use_to(i, left, right),
      _ => self.err(),
    }
  }

  fn use_cont(&mut self, i: Self) -> Result<(), SolveError> {
    self.infer(InferenceType::UnaryInf(Box::new(i)));
    Ok(())
  }

  fn use_not(&mut self, i1: Self, logic: &'a Logic) -> Result<(), SolveError> {
    let mut i0 = self.problem(logic);
    i0.solve()?;

    let mut i = self.problem(&Logic::Cont);
    i.infer(InferenceType::BinaryInf(Box::new(i0), Box::new(i1)));
    self.use_cont(i)
  }

  fn use_and(&mut self, i0: Self, left: &'a Logic, right: &'a Logic) -> Result<(), SolveError> {
    for logic in [left, right] {
      let mut i = self.problem(logic);
      i.infer(InferenceType::UnaryInf(Box::new(i0.clone())));

      if let Ok(_) = self.use_logic(i) {
        return Ok(());
      }
    }

    self.err()
  }

  fn use_or(&mut self, i0: Self, left: &'a Logic, right: &'a Logic) -> Result<(), SolveError> {
    let mut i1 = self.problem(self.logic);
    if let Some(_) = i1.axioms.insert(left, self.marker.clone()) {
      self.err()?
    }
    i1.solve()?;

    let mut i2 = self.problem(self.logic);
    if let Some(_) = i2.axioms.insert(right, self.marker.clone()) {
      self.err()?
    }
    i2.solve()?;

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
    if let Some(_) = i.axioms.insert(logic, self.marker.clone()) {
      self.err()?
    }
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
    let mut i0 = self.problem(right);
    if let Some(_) = i0.axioms.insert(left, self.marker.clone()) {
      self.err()?
    }
    i0.solve()?;

    self.infer(InferenceType::UnaryInf(Box::new(i0)));
    Ok(())
  }

  fn print(&self, tree: &mut String, indent: &str, mut after: usize) -> usize {
    let marker = if Rc::weak_count(&self.marker) > 0 {
      after += 1;
      self.marker.replace(after);
      format!(" : {}", self.marker.borrow())
    } else {
      if let Some(InferenceType::Axiom(ref marker)) = self.inference {
        format!(" from: {}", marker.upgrade().unwrap().borrow())
      } else {
        String::new()
      }
    };

    tree.push_str(&format!("{}{}\n", self.logic, marker));

    match self.inference {
      None | Some(InferenceType::Axiom(_))=> {}
      Some(InferenceType::UnaryInf(ref i0)) => {
        tree.push_str(&format!("{}+ ", indent));
        after = i0.print(tree, &format!("{}  ", indent), after);
      }
      Some(InferenceType::BinaryInf(ref i0, ref i1)) => {
        tree.push_str(&format!("{}+ ", indent));
        after = i0.print(tree, &format!("{}| ", indent), after);
        tree.push_str(&format!("{}+ ", indent));
        after = i1.print(tree, &format!("{}  ", indent), after);
      }
      Some(InferenceType::TrinaryInf(ref i0, ref i1, ref i2)) => {
        tree.push_str(&format!("{}+ ", indent));
        after = i0.print(tree, &format!("{}| ", indent), after);
        tree.push_str(&format!("{}+ ", indent));
        after = i1.print(tree, &format!("{}| ", indent), after);
        tree.push_str(&format!("{}+ ", indent));
        after = i2.print(tree, &format!("{}  ", indent), after);
      }
    }

    after
  }

  fn print_tex(&self, mut after: usize) -> (String, usize) {
    let marker = if Rc::weak_count(&self.marker) > 0 {
      after += 1;
      self.marker.replace(after);
      format!("\\RightLabel{{\\scriptsize {}}}\n", self.marker.borrow())
    } else {
      String::new()
    };

      match self.inference {
      None => (format!("{}", self.logic.tex()), after),
      Some(InferenceType::Axiom(ref marker)) => (format!("\\AxiomC{{$[{}]_{{{}}}$}}", self.logic.tex(), marker.upgrade().unwrap().borrow()), after),
      Some(InferenceType::UnaryInf(ref i0)) => {
        let (s0, after) = i0.print_tex(after);
        (format!("{}\n{}\\UnaryInfC{{${}$}}", s0, marker, self.logic.tex()), after)
      }
      Some(InferenceType::BinaryInf(ref i0, ref i1)) => {
        let (s0, after) = i0.print_tex(after);
        let (s1, after) = i1.print_tex(after);
        (format!("{}\n{}\n{}\\BinaryInfC{{${}$}}", s0, s1, marker, self.logic.tex()), after)
      }
      Some(InferenceType::TrinaryInf(ref i0, ref i1, ref i2)) => {
        let (s0, after) = i0.print_tex(after);
        let (s1, after) = i1.print_tex(after);
        let (s2, after) = i2.print_tex(after);
        (format!("{}\n{}\n{}\n{}\\TrinaryInfC{{${}$}}", s0, s1, s2, marker, self.logic.tex()), after)
      }
    }
  }
}

impl TeX for Inference<'_> {
  fn tex(&self) -> String {
    self.print_tex(0).0
  }
}

impl Display for Inference<'_> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut tree = String::new();
    self.print(&mut tree, "", 0);
    write!(f, "{}", tree)
  }
}

#[derive(Debug)]
pub enum SolveError {
  InferError(Logic),
  CheckError(CheckError)
}

impl From<CheckError> for SolveError {
  fn from(e: CheckError) -> Self {
    Self::CheckError(e)
  }
}

impl Display for SolveError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::InferError(logic) => write!(f, "could not infer: {}", logic),
      Self::CheckError(e) =>  write!(f, "{}", e),
    }
  }
}

impl Error for SolveError {}
