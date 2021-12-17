//! 引数解析ライブラリ`structopt`の設定を行うモジュールです．
//! main関数に`#[paw::main]`マクロを適応することで，コマンドライン引数を自動で[Args]にパースします．
//! 詳しくは[公式ドキュメント](https://docs.rs/structopt/0.3.25/structopt/)を参照してください．

use std::path::PathBuf;
use structopt::StructOpt;

/// Parses propositional logic in TeX, outputs in TeX
#[derive(Debug, StructOpt)]
#[structopt(
  name = "Propositional Logic Solver",
  about = "Parses propositional logic in TeX, outputs in TeX",
  author = "cm-ayf"
)]
pub struct Args {
  /// execute in interactive mode
  #[structopt(short, long)]
  pub interactive: bool,

  /// text input
  pub input: Option<String>,

  /// output in TeX format (bussproof.sty)
  #[structopt(short, long)]
  pub tex: bool,

  /// output file (if omitted, stdout)
  #[structopt(short, long, parse(from_os_str))]
  pub out: Option<PathBuf>,
}
