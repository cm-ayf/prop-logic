use prop_logic::parser;
use prop_logic::solver;

fn main() {
  let expr = parser("(A \\lor B) \\lor C \\to A \\lor (B \\lor C)").unwrap();
  println!("{:?}", solver(&expr));
}
