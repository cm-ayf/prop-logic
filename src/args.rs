use std::path::PathBuf;
use structopt::StructOpt;

use crate::exec::*;

#[derive(Debug, StructOpt)]
#[structopt(
  name = "Propositional Logic Solver",
  about = "Parses propositional logic in TeX, outputs in TeX",
  author = "cm-ayf"
)]
pub struct Args {
  input: String,

  /// output in TeX format (bussproof.sty)
  #[structopt(short, long)]
  tex: bool,

  /// output file (if omitted, stdout)
  #[structopt(short, long, parse(from_os_str))]
  out: Option<PathBuf>,
}

impl Args {
  pub fn exec(&self) -> Result<(), ExecError> {
    if let Some(res) = exec(&self.input, self.tex, &self.out)? {
      println!("{}", res);
    }
    Ok(())
  }
}
