use prop_logic::Logic;
use prop_logic::TeX;

fn main() {
  let logic: Logic = "((A to C) and (B to D)) to ((A or B) to (C or D))".parse().unwrap();
  println!("{:?}", logic.check_all());

  match logic.solve() {
    Ok(i) => {
      println!("{}", i);
      println!("{}", i.tex());
    },
    Err(()) => {
      println!("could not solve");
    }
  }
}
