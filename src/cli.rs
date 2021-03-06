//! 引数解析ライブラリ`structopt`の設定と，コマンドラインとしての実行内容を実装するモジュールです．
//! 詳しくは[公式ドキュメント](https://docs.rs/structopt/0.3.25/structopt/)を参照してください．
//! 
//! main関数に`#[paw::main]`マクロを適応することで，コマンドライン引数を自動で[Args]にパースします．
//! `exec`メソッドによって，解析された引数に則って実行できます．
//! #Examples
//! ```no_run
//! #[paw::main]
//! fn main(args: Args) {
//!   if let Err(e) = args.exec() {
//!     eprintln!("{}", e);
//!   }
//! }
//! ```
//! 
//! 

use std::path::PathBuf;
use structopt::StructOpt;

use crate::exec::*;

/// Parses propositional logic in TeX, outputs in TeX
#[derive(Debug, StructOpt)]
#[structopt(
  name = "Propositional Logic Solver",
  about = "Parses propositional logic in TeX, outputs in TeX",
  author = "cm-ayf"
)]
pub struct Args {
  /// text input (if omitted, starts in interactive mode)
  input: Option<String>,

  /// output in TeX format (bussproof.sty)
  #[structopt(short, long)]
  tex: bool,

  /// output file (if omitted, stdout)
  #[structopt(short, long, parse(from_os_str))]
  out: Option<PathBuf>,
}

impl Args {
  /// 解析されたコマンドラインの命令を実行します．詳しくは[このモジュールの説明](self)を参照してください．
  pub fn exec(&self) -> Result<(), ExecError> {
    if let Some(ref input) = self.input {
      let res = exec(input, self.tex)?;

      match self.out {
        Some(ref path) => std::fs::write(path, res)?,
        None => println!("{}", res),
      };

      Ok(())
    } else {
      loop {
        println!("input ('quit' to quit):");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        if input.starts_with("quit") {
          return Ok(());
        }

        let res = exec(&input, self.tex)?;

        match self.out {
          Some(ref path) => std::fs::write(path, res)?,
          None => println!("{}", res),
        };
      }
    }
  }
}
