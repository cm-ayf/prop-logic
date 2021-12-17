use prop_logic::Args;

/// 副作用を含む処理をこの関数内に実装し，副作用を含まない処理は全てライブラリクレートに委託しています．
#[paw::main]
fn main(args: Args) -> Result<(), prop_logic::ExecError> {
  if args.interactive {
    loop {
      let input = match input() {
        Ok(input) if input.starts_with("quit") => std::process::exit(0),
        Ok(input) => input,
        Err(e) => {
          eprintln!("input error: {}", e);
          continue;
        }
      };

      let res = prop_logic::exec(&input, args.tex)?;

      match args.out {
        Some(ref path) => {
          std::fs::write(path, res)?;
          break;
        }
        None => println!("{}", res),
      };
    }
  } else {
    if let Some(ref input) = args.input {
      let res = prop_logic::exec(input, args.tex)?;

      match args.out {
        Some(ref path) => std::fs::write(path, res)?,
        None => println!("{}", res),
      };
    }
  }

  Ok(())
}

fn input() -> Result<String, prop_logic::ExecError> {
  println!("input ('quit' to quit):");
  let mut input = String::new();
  std::io::stdin().read_line(&mut input)?;
  Ok(input)
}
