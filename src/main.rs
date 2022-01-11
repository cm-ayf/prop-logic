use prop_logic::Args;

/// 副作用を含む処理をこの関数内に実装し，副作用を含まない処理は全てライブラリクレートに委託しています．
#[paw::main]
fn main(args: Args) {
  if let Err(e) = args.exec() {
    eprintln!("{}", e);
  }
}
