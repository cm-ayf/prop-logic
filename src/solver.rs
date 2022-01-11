//! 論理式を受け取り，推論を行うモジュールです．

use std::cell::RefCell;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;
use std::rc::*;

use super::{logic::*, TeX};

/// 推論を示す構造です．木構造のノードです．仮定以外では証明図の横線と一対一対応します．
#[derive(Debug, Clone)]
pub struct Problem<'a> {
  /// 推論されるべき論理です．
  logic: &'a Logic,

  /// この推論に用いることができる仮定の集合です．key-valueペアのkeyが仮定された論理式で，
  /// valueはその仮定が導出された[Inference](self::Inference)の[self::Inference]です．
  axioms: HashMap<&'a Logic, Rc<RefCell<usize>>>,

  /// 推論を一意に示すためのマーカーです．
  /// 仮定を用いるときに参照番号を付けるために利用します．
  marker: Rc<RefCell<usize>>,

  /// 解こうとしている問題の列です．
  /// ループを検知するために利用します．
  history: Vec<Self>,
}

impl PartialEq for Problem<'_> {
  fn eq(&self, other: &Self) -> bool {
    self.logic == other.logic && self.axioms == other.axioms
  }
}

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

  /// 解こうとしている問題の列です．
  /// ループを検知するために利用します．
  history: Vec<Problem<'a>>,
  
  /// 推論のタイプです．
  /// 詳しくは[InferenceType](InferenceType)の説明を参照してください．
  inference: InferenceType<'a>,
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

impl<'a> Problem<'a> {
  /// 新しい推論すべき問題を生成します．
  pub fn new(logic: &'a Logic) -> Self {
    Self {
      logic,
      axioms: HashMap::new(),
      marker: Rc::new(RefCell::new(0)),
      history: Vec::new(),
    }
  }

  /// 自分の卑属で推論すべき問題を生成します．
  fn problem(&self, logic: &'a Logic, insert: Option<(&'a Logic, Rc<RefCell<usize>>)>) -> Self {
    let mut axioms = self.axioms.clone();
    if let Some((k, v)) = insert {
      axioms.insert(k, v);
    }

    let mut history = self.history.clone();
    history.push(self.clone());

    Self {
      logic,
      axioms,
      marker: Rc::new(RefCell::new(0)),
      history,
    }
  }

  /// 自分の推論が得られたとき，自分を推論にアップグレードします．
  fn infer(self, inference: InferenceType<'a>) -> Inference<'a> {
    let Self {
      logic,
      axioms,
      marker,
      history
    } = self;
    Inference {
      logic,
      axioms,
      marker,
      history,
      inference,
    }
  }

  fn err(&self) -> SolveResult<'a> {
    Err(SolveError::InferError(self.logic.clone()))
  }

  /// 自分の推論を試みます．
  pub fn solve(self) -> SolveResult<'a> {
    if let Ok(i) = self.clone().use_axioms() {
      return Ok(i);
    }

    if let Ok(i) = self.clone().infer_logic() {
      return Ok(i);
    }

    if let Ok(i) = self.clone().use_axioms() {
      return Ok(i);
    }

    self.err()
  }

  /// 自分が使える仮定から自分の推論を試みます．
  fn use_axioms(&self) -> SolveResult<'a> {
    let axioms = self.axioms.clone();

    for (axiom, marker) in axioms {
      let i = self
        .problem(axiom, None)
        .infer(InferenceType::Axiom(Rc::downgrade(&marker)));
      if let Ok(i) = i.use_logic(self.clone()) {
        return Ok(i);
      }
    }

    self.err()
  }

  /// 自分の論理式の木の根の演算子を導入し，推論を試みます．
  fn infer_logic(self) -> SolveResult<'a> {
    match self.logic {
      Logic::Not(logic) => self.infer_not(logic),
      Logic::And(left, right) => self.infer_and(left, right),
      Logic::Or(left, right) => self.infer_or(left, right),
      Logic::To(left, right) => self.infer_to(left, right),
      _ => self.err(),
    }
  }

  /// 論理否定を導入します．否定されていない命題を仮定し，矛盾の推論を試みます．
  fn infer_not(self, logic: &'a Logic) -> SolveResult<'a> {
    let p = self.problem(&Logic::Cont, Some((logic, self.marker.clone())));
    Ok(self.infer(InferenceType::UnaryInf(Box::new(p.solve()?))))
  }

  /// 論理積を導入するため，2つの命題の推論をそれぞれ試みます．
  fn infer_and(self, left: &'a Logic, right: &'a Logic) -> SolveResult<'a> {
    let p0 = self.problem(left, None);
    let p1 = self.problem(right, None);
    Ok(self.infer(InferenceType::BinaryInf(
      Box::new(p0.solve()?),
      Box::new(p1.solve()?),
    )))
  }

  /// 論理和を導入するため，2つの命題の推論をそれぞれ試みます．
  fn infer_or(self, left: &'a Logic, right: &'a Logic) -> SolveResult<'a> {
    for logic in [left, right] {
      let p = self.problem(logic, None);
      if let Ok(i) = p.solve() {
        return Ok(self.infer(InferenceType::UnaryInf(Box::new(i))));
      }
    }

    self.err()
  }

  /// 論理包含を導入するため，左の命題を仮定し，右の命題の推論を試みます．
  fn infer_to(self, left: &'a Logic, right: &'a Logic) -> SolveResult<'a> {
    let p0 = self.problem(right, Some((left, self.marker.clone())));
    Ok(self.infer(InferenceType::UnaryInf(Box::new(p0.solve()?))))
  }
}

impl<'a> Inference<'a> {
  /// 自分の卑属で推論すべき問題を生成します．
  fn problem(
    &self,
    logic: &'a Logic,
    insert: Option<(&'a Logic, Rc<RefCell<usize>>)>,
  ) -> Problem<'a> {
    let mut axioms = self.axioms.clone();
    if let Some((k, v)) = insert {
      axioms.insert(k, v);
    }

    Problem {
      logic,
      axioms,
      marker: Rc::new(RefCell::new(0)),
      history: self.history.clone(),
    }
  }

  /// 自分が解けなかったというエラーを出力します．
  fn err(&self) -> SolveResult<'a> {
    Err(SolveError::InferError(self.logic.clone()))
  }

  /// 得られた推論から目的の問題の推論を試みます．
  fn use_logic(self, target: Problem<'a>) -> SolveResult<'a> {
    if self.logic.eq(target.logic) {
      return Ok(self);
    }

    match self.logic {
      Logic::Cont => self.use_cont(target),
      Logic::Not(logic) => self.use_not(target, logic),
      Logic::And(left, right) => self.use_and(target, left, right),
      Logic::Or(left, right) => self.use_or(target, left, right),
      Logic::To(left, right) => self.use_to(target, left, right),
      _ => self.err(),
    }
  }

  /// 矛盾を除去し，これを利用して目的の問題を推論します．
  fn use_cont(self, target: Problem<'a>) -> SolveResult<'a> {
    Ok(target.infer(InferenceType::UnaryInf(Box::new(self))))
  }

  /// 否定の除去を試み，可能であれば矛盾を推論します．
  fn use_not(self, target: Problem<'a>, logic: &'a Logic) -> SolveResult<'a> {
    let p0 = self.problem(logic, None);
    let p = self.problem(&Logic::Cont, None);

    let i = p.infer(InferenceType::BinaryInf(
      Box::new(p0.solve()?),
      Box::new(self),
    ));
    i.use_logic(target)
  }

  /// 論理積を除去し，これを用いて目的の問題の推論を試みます．
  fn use_and(self, target: Problem<'a>, left: &'a Logic, right: &'a Logic) -> SolveResult<'a> {
    for logic in [left, right] {
      let p = self.problem(logic, None);
      let i = p.infer(InferenceType::UnaryInf(Box::new(self.clone())));
      if let Ok(i) = i.use_logic(target.clone()) {
        return Ok(i);
      }
    }

    self.err()
  }

  /// 論理和の除去を試み，可能であればこれを用いて目的の問題を推論します．
  fn use_or(self, target: Problem<'a>, left: &'a Logic, right: &'a Logic) -> SolveResult<'a> {
    let p1 = self.problem(self.logic, Some((left, self.marker.clone())));
    let p2 = self.problem(self.logic, Some((right, self.marker.clone())));

    Ok(target.infer(InferenceType::TrinaryInf(
      Box::new(self),
      Box::new(p1.solve()?),
      Box::new(p2.solve()?),
    )))
  }

  /// 論理和の除去を試み，可能であればこれを用いて目的の問題の推論を試みます．
  fn use_to(self, target: Problem<'a>, left: &'a Logic, right: &'a Logic) -> SolveResult<'a> {
    let p0 = self.problem(left, None);
    let p = self.problem(right, None);

    let i = p.infer(InferenceType::BinaryInf(
      Box::new(p0.solve()?),
      Box::new(self),
    ));

    i.use_logic(target)
  }

  /// 標準出力用の証明図出力を行う関数です．
  fn print(&self, tree: &mut String, indent: &str, after: &mut usize) {
    let marker = if Rc::weak_count(&self.marker) > 0 {
      *after += 1;
      self.marker.replace(*after);
      format!(" : {}", self.marker.borrow())
    } else {
      if let InferenceType::Axiom(ref marker) = self.inference {
        format!(" from: {}", marker.upgrade().unwrap().borrow())
      } else {
        String::new()
      }
    };

    tree.push_str(&format!("{}{}\n", self.logic, marker));
    match self.inference {
      InferenceType::Axiom(_) => {}
      InferenceType::UnaryInf(ref i0) => {
        tree.push_str(&format!("{}+ ", indent));
        i0.print(tree, &format!("{}  ", indent), after);
      }
      InferenceType::BinaryInf(ref i0, ref i1) => {
        tree.push_str(&format!("{}+ ", indent));
        i0.print(tree, &format!("{}| ", indent), after);
        tree.push_str(&format!("{}+ ", indent));
        i1.print(tree, &format!("{}  ", indent), after);
      }
      InferenceType::TrinaryInf(ref i0, ref i1, ref i2) => {
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
      InferenceType::Axiom(ref marker) => {
        tree.push_str(&format!(
          "{}[{}]_{{{}}}\n",
          indent,
          self.logic.tex(),
          marker.upgrade().unwrap().borrow()
        ));
      }
      InferenceType::UnaryInf(ref i0) => {
        tree.push_str(&format!(
          "{}\\infer{}{{{}}}{{\n",
          indent,
          marker,
          self.logic.tex()
        ));
        i0.print_tex(tree, &format!("{}  ", indent), after);
        tree.push_str(&format!("{}}}\n", indent));
      }
      InferenceType::BinaryInf(ref i0, ref i1) => {
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
      InferenceType::TrinaryInf(ref i0, ref i1, ref i2) => {
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

type SolveResult<'a> = Result<Inference<'a>, SolveError>;

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
