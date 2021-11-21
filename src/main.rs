use prop_logic::Expr;

fn main() {
  let expr: Expr = "((A \\to C) \\land (B \\to C)) \\to (A \\lor B \\to C)".parse().unwrap();
  println!("{}", expr);
  println!("{:?}", expr);
  println!("{:?}", expr.check_all());
  println!("{:?}", expr.solve());
}
