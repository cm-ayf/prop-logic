//! 論理式を受け取り，推論を行うモジュールです．

use std::cell::RefCell;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;
use std::rc::*;

use super::{logic::*, TeX};

/// 推論を示す構造です．木構造のノードです．仮定以外では証明図の横線と一対一対応します．
#[derive(Debug, Clone)]
pub struct Inference<'a> {
  /// 推論されるべき論理です．
  logic: &'a Logic,

  /// この推論に用いることができる仮定の集合です．key-valueペアのkeyが仮定された論理式で，
  /// valueはその仮定が導出された[Inference](self::Inference)の[self::Inference]です．
  axioms: HashMap<&'a Logic, Rc<RefCell<usize>>>,

  /// 推論を一意に示すためのマーカーです．
  /// 仮定を用いるときに参照番号を付けるために利用します．
  marker: Rc<RefCell<usize>>,

  /// 推論のタイプです．[None]はまだ推論されていないことを示します．
  /// 詳しくは[InferenceType](InferenceType)の説明を参照してください．
  inference: Option<InferenceType<'a>>,
}

/// 推論のタイプを示す列挙子です．
#[derive(Debug, Clone)]
enum InferenceType<'a> {
  /// 仮定です．
  Axiom(Weak<RefCell<usize>>),

  /// 1つの命題から推論するタイプです．論理包含の導入などで用いられます．
  UnaryInf(Box<Inference<'a>>),

  /// 2つの命題から推論するタイプです．論理積の導入などで用いられます．
  BinaryInf(Box<Inference<'a>>, Box<Inference<'a>>),

  /// 3つの命題から推論するタイプです．論理和の消去で用いられます．
  TrinaryInf(Box<Inference<'a>>, Box<Inference<'a>>, Box<Inference<'a>>),
}

impl<'a> Inference<'a> {
  /// 新しい推論すべき問題を生成します．
  pub fn new(logic: &'a Logic) -> Self {
    Self {
      logic,
      axioms: HashMap::new(),
      marker: Rc::new(RefCell::new(0)),
      inference: None,
    }
  }

  /// 自分の卑属で推論すべき問題を生成します．
  fn problem(&self, logic: &'a Logic) -> Self {
    Self {
      logic,
      axioms: self.axioms.clone(),
      marker: Rc::new(RefCell::new(0)),
      inference: None,
    }
  }

  /// 自分が解けなかったというエラーを出力します．
  fn err(&self) -> Result<(), SolveError> {
    Err(SolveError::InferError(self.logic.clone()))
  }

  /// 自分の推論を反映します．
  fn infer(&mut self, inference: InferenceType<'a>) {
    self.inference = Some(inference);
  }

  /// 推論全体のエントリーポイントです．
  pub fn solve(&mut self) -> Result<(), SolveError> {
    if let Ok(_) = self.use_axioms() {
      return Ok(());
    }

    if let Ok(_) = self.infer_logic() {
      return Ok(());
    }

    self.err()
  }

  /// それが古典論理上推論可能かを確かめます．
  fn validate(&self) -> Result<(), CheckError> {
    let mut logic = Logic::Not(Box::new(self.logic.clone()));

    for (axiom, _) in self.axioms.clone() {
      logic = Logic::And(Box::new(logic), Box::new(axiom.clone()));
    }

    logic = Logic::Not(Box::new(logic));

    logic.check_all()
  }

  /// 自分が使える仮定を用いて何か示せないか試みます．
  fn use_axioms(&mut self) -> Result<(), SolveError> {
    let mut axioms = self.axioms.clone();
    self.shave_axioms(&mut axioms)?;

    for (axiom, marker) in axioms {
      let mut i = self.problem(axiom);
      i.infer(InferenceType::Axiom(Rc::downgrade(&marker)));
      if let Ok(_) = self.use_logic(i) {
        return Ok(());
      }
    }

    self.err()
  }

  /// 自分が使える仮定のうち，それがなくても古典論理上推論可能な仮定を除きます．
  fn shave_axioms(
    &self,
    axioms: &mut HashMap<&'a Logic, Rc<RefCell<usize>>>,
  ) -> Result<(), CheckError> {
    let mut i = self.clone();
    i.axioms = axioms.clone();
    i.validate()?;

    for (axiom, _) in i.axioms {
      if let Some((axiom, marker)) = axioms.remove_entry(axiom) {
        if let Err(_) = self.shave_axioms(axioms) {
          axioms.insert(axiom, marker);
        }
      }
    }

    Ok(())
  }

  /// 仮定や，仮定から導かれた成立する論理をさらに用いて何か示せないか試みます．
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

  /// 矛盾が導かれた時に，これを利用して自分を推論します．
  fn use_cont(&mut self, i: Self) -> Result<(), SolveError> {
    self.infer(InferenceType::UnaryInf(Box::new(i)));
    Ok(())
  }

  /// ある命題の否定が導かれたときに，否定されていない命題が解ければ，矛盾を推論します．
  fn use_not(&mut self, i1: Self, logic: &'a Logic) -> Result<(), SolveError> {
    let mut i0 = self.problem(logic);
    i0.solve()?;

    let mut i = self.problem(&Logic::Cont);
    i.infer(InferenceType::BinaryInf(Box::new(i0), Box::new(i1)));
    self.use_logic(i)
  }

  /// 2つの命題の論理積が導かれたときに，それら2つを用いて何か示せないか試みます．
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

  /// 2つの命題の論理和が導かれたときに，これを用いて自分を推論できないか試みます．
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

  /// 2つの命題の論理包含が導かれたときに，左の命題が解ければ，右の命題を用いて何か示せないか試みます．
  fn use_to(&mut self, i1: Self, left: &'a Logic, right: &'a Logic) -> Result<(), SolveError> {
    let mut i0 = self.problem(left);
    i0.solve()?;

    let mut i = self.problem(right);
    i.infer(InferenceType::BinaryInf(Box::new(i0), Box::new(i1)));
    self.use_logic(i)
  }

  /// 自分の論理式の木の根の演算子を導入します．
  fn infer_logic(&mut self) -> Result<(), SolveError> {
    match self.logic {
      Logic::Not(logic) => self.infer_not(logic),
      Logic::And(left, right) => self.infer_and(left, right),
      Logic::Or(left, right) => self.infer_or(left, right),
      Logic::To(left, right) => self.infer_to(left, right),
      _ => self.err(),
    }
  }

  /// 論理否定を導入します．否定されていない命題を仮定し，矛盾を推論しようと試みます．
  fn infer_not(&mut self, logic: &'a Logic) -> Result<(), SolveError> {
    let mut i = self.problem(&Logic::Cont);
    if let Some(_) = i.axioms.insert(logic, self.marker.clone()) {
      self.err()?
    }
    i.solve()?;

    self.infer(InferenceType::UnaryInf(Box::new(i)));
    Ok(())
  }

  /// 論理積を導入します．2つの命題をそれぞれ推論しようと試みます．
  fn infer_and(&mut self, left: &'a Logic, right: &'a Logic) -> Result<(), SolveError> {
    let mut i0 = self.problem(left);
    i0.solve()?;

    let mut i1 = self.problem(right);
    i1.solve()?;

    self.infer(InferenceType::BinaryInf(Box::new(i0), Box::new(i1)));
    Ok(())
  }

  /// 論理和を導入します．2つの命題のいずれかを推論しようと試みます．
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

  /// 論理包含を導入します．左の命題を仮定し，右の命題を推論しようと試みます．
  fn infer_to(&mut self, left: &'a Logic, right: &'a Logic) -> Result<(), SolveError> {
    let mut i0 = self.problem(right);
    if let Some(_) = i0.axioms.insert(left, self.marker.clone()) {
      self.err()?
    }
    i0.solve()?;

    self.infer(InferenceType::UnaryInf(Box::new(i0)));
    Ok(())
  }

  /// 標準出力用の証明図出力を行う関数です．
  fn print(&self, tree: &mut String, indent: &str, after: &mut usize) {
    let marker = if Rc::weak_count(&self.marker) > 0 {
      *after += 1;
      self.marker.replace(*after);
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
      None | Some(InferenceType::Axiom(_)) => {}
      Some(InferenceType::UnaryInf(ref i0)) => {
        tree.push_str(&format!("{}+ ", indent));
        i0.print(tree, &format!("{}  ", indent), after);
      }
      Some(InferenceType::BinaryInf(ref i0, ref i1)) => {
        tree.push_str(&format!("{}+ ", indent));
        i0.print(tree, &format!("{}| ", indent), after);
        tree.push_str(&format!("{}+ ", indent));
        i1.print(tree, &format!("{}  ", indent), after);
      }
      Some(InferenceType::TrinaryInf(ref i0, ref i1, ref i2)) => {
        tree.push_str(&format!("{}+ ", indent));
        i0.print(tree, &format!("{}| ", indent), after);
        tree.push_str(&format!("{}+ ", indent));
        i1.print(tree, &format!("{}| ", indent), after);
        tree.push_str(&format!("{}+ ", indent));
        i2.print(tree, &format!("{}  ", indent), after);
      }
    }
  }

  /// TeX記法用の証明図出力を行う関数です．
  fn print_tex(&self, tree: &mut String, indent: &str, after: &mut usize) {
    let marker = if Rc::weak_count(&self.marker) > 0 {
      *after += 1;
      self.marker.replace(*after);
      format!("[{}]", self.marker.borrow())
    } else {
      String::new()
    };

    match self.inference {
      None => {}
      Some(InferenceType::Axiom(ref marker)) => {
        tree.push_str(&format!(
          "{}[{}]_{{{}}}\n",
          indent,
          self.logic.tex(),
          marker.upgrade().unwrap().borrow()
        ));
      }
      Some(InferenceType::UnaryInf(ref i0)) => {
        tree.push_str(&format!(
          "{}\\infer{}{{{}}}{{\n",
          indent,
          marker,
          self.logic.tex()
        ));
        i0.print_tex(tree, &format!("{}  ", indent), after);
        tree.push_str(&format!("{}}}\n", indent));
      }
      Some(InferenceType::BinaryInf(ref i0, ref i1)) => {
        tree.push_str(&format!(
          "{}\\infer{}{{{}}}{{\n",
          indent,
          marker,
          self.logic.tex()
        ));
        i0.print_tex(tree, &format!("{}  ", indent), after);
        tree.push_str(&format!("{}  &\n", indent));
        i1.print_tex(tree, &format!("{}  ", indent), after);
        tree.push_str(&format!("{}}}\n", indent));
      }
      Some(InferenceType::TrinaryInf(ref i0, ref i1, ref i2)) => {
        tree.push_str(&format!(
          "{}\\infer{}{{{}}}{{\n",
          indent,
          marker,
          self.logic.tex()
        ));
        i0.print_tex(tree, &format!("{}  ", indent), after);
        tree.push_str(&format!("{}  &\n", indent));
        i1.print_tex(tree, &format!("{}  ", indent), after);
        tree.push_str(&format!("{}  &\n", indent));
        i2.print_tex(tree, &format!("{}  ", indent), after);
        tree.push_str(&format!("{}}}\n", indent));
      }
    }
  }
}

impl TeX for Inference<'_> {
  fn tex(&self) -> String {
    let mut tree = String::new();
    self.print_tex(&mut tree, "", &mut 0);
    tree
  }
}

impl Display for Inference<'_> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut tree = String::new();
    self.print(&mut tree, "", &mut 0);
    write!(f, "{}", tree)
  }
}

/// 推論時に起きるエラーをまとめた列挙子です．
#[derive(Debug)]
pub enum SolveError {
  /// 古典論理上は証明できるが，証明に失敗した場合のエラーです．
  InferError(Logic),

  /// 古典論理上証明できない場合のエラーです．
  CheckError(CheckError),
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
      Self::CheckError(e) => write!(f, "{}", e),
    }
  }
}

impl Error for SolveError {}
