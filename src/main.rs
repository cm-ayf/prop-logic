use prop_logic::Expr;

fn main() {
  let expr: Expr = "(A \\to \\lnot B) \\to (B \\to \\lnot A)".parse().unwrap();
  println!("{}", expr);
  println!("{:?}", expr);
  println!("{:?}", expr.check_all());
  println!("{:?}", expr.solve());
}
