use prop_logic::Expr;

fn main() {
  let expr: Expr = "(A \\lor B) \\lor C \\to A \\lor (B \\lor C)".parse().unwrap();
  println!("{}", expr);
  println!("{:?}", expr.check_all());
  println!("{:?}", expr.solve());
}
