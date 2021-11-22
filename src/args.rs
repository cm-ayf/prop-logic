use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
  name = "Propotional Logic Solver",
  about = "Parses propotional logic in TeX, outputs in TeX",
  author = "cm-ayf"
)]
pub struct Args {
  pub input: String,

  #[structopt(short, long)]
  pub tex: bool,

  #[structopt(short, long, parse(from_os_str))]
  pub out: Option<PathBuf>
}