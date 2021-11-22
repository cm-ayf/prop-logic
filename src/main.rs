use prop_logic::Args;

#[paw::main]
fn main(args: Args) {
  if let Err(e) = args.exec() {
    eprintln!("{}", e);
  }
}
