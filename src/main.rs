use prop_logic::parser;
use prop_logic::solver;

fn main() {
  let expr = parser("((A \\to C) \\land (B \\to C)) \\to (A \\lor B \\to C)").unwrap();
  println!("{:?}", solver(&expr));
}
