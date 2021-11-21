use prop_logic::Logic;

fn main() {
  let logic: Logic = "(A \\to \\lnot B) \\to (B \\to \\lnot A)".parse().unwrap();
  println!("{}", logic);
  println!("{:?}", logic);
  println!("{:?}", logic.check_all());
  println!("{:?}", logic.solve());
}
