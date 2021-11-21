use prop_logic::parser;
use prop_logic::solver;

fn main() {
  let expr = parser("(A \\lor B \\to C) \\to ((A \\to C) \\land (B \\to C))").unwrap();
  println!("{:?}", solver(&expr));
}
