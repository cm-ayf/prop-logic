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
  #[structopt(short, long)]
  interactive: bool,

  input: Option<String>,

  /// output in TeX format (bussproof.sty)
  #[structopt(short, long)]
  tex: bool,

  /// output file (if omitted, stdout)
  #[structopt(short, long, parse(from_os_str))]
  out: Option<PathBuf>,
}

impl Args {
  pub fn exec(&self) -> Result<(), ExecError> {
    if self.interactive {
      loop {
        println!("input ('quit' to quit):");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        if input.starts_with("quit") {
          return Ok(());
        }

        if let Some(res) = exec(&input, self.tex, &self.out)? {
          println!("{}", res);
        }
      }
    } else {
      if let Some(ref input) = &self.input {
        if let Some(res) = exec(input, self.tex, &self.out)? {
          println!("{}", res);
        }
      }
      Ok(())
    }
  }
}

