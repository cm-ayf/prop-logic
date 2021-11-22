use prop_logic::Logic;
use prop_logic::TeX;

mod args;

#[paw::main]
fn main(args: args::Args) {
  let logic: Logic = match args.input.parse() {
    Ok(s) => s,
    Err(e) => {
      eprintln!("error parsing input: {}", e);
      return;
    }
  };

  if let Err(e) = logic.check_all() {
    eprintln!("turns out to be false if: {}", e);
    return;
  }

  let inference = match logic.solve() {
    Ok(i) => i,
    Err(()) => {
      eprintln!("could not solve");
      return;
    }
  };

  let res = if args.tex {
    inference.tex()
  } else {
    inference.to_string()
  };

  match args.out {
    Some(path) => if let Err(e) = std::fs::write(path, res) {
      eprintln!("could not write file: {}", e);
    },
    None => println!("{}", res)
  };
}
