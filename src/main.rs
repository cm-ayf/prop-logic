use prop_logic::Args;

/// Rust文化圏のお作法に乗っ取り，エントリーポイントとして最小限の処理だけを行なっています．
/// 詳しくは[Args]の説明を参照してください．
#[paw::main]
fn main(args: Args) {
  if let Err(e) = args.exec() {
    eprintln!("{}", e);
  }
}
