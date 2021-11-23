# prop-logic

## 概要

命題論理ソルバーです．TeX記法等でインラインで証明したい論理式を入力すると，証明図を吐きます．出力には，簡略化した記法とTeX記法のいずれかが選べます．

仮定の参照先が明示されるようになりました．

## To Be implemented

* ~~仮定の参照先明示~~
* 排中律の運用（選択式の予定）

## インストール
```bash
git clone https://github.com/cm-ayf/prop-logic
cd prop-logic
cargo install --path .
```

またはインストールなしで`cargo run [--release]`してください．

## 使い方

* 例

```bash
$ prop-logic "((A or B) to C) to (A to C) and (B to C)"
(A ∨ B → C) → (A → C) ∧ (B → C) : 1
+ (A → C) ∧ (B → C)
  + A → C : 2
  | + C
  |   + A ∨ B
  |   | + A from: 2
  |   + A ∨ B → C from: 1
  + B → C : 3
    + C
      + A ∨ B
      | + B from: 3
      + A ∨ B → C from: 1
```

* 記法
  * `not | \lnot`：否定（…でない）です．
  * `and | \land`：論理積（かつ）です．
  * `or | \lor`：論理和（または）です．
  * `to | \to `：論理包含（ならば）です．

* かっこ`()`について
  * 優先順位を指定します．
  * 同じ二項演算（`and, or, to`）を繰り返す場合はかっこが必要です．
    * 例：`(A and B) and C to A and (B and C)`
  * 記法の項で上にあるものほど優先して計算されます：
    * 例：`not A and B to (A \to B)`は`((not A) and B) to (A to B)`に同じです．

* 引数一覧

```bash
$ prop-logic -h
USAGE:
    prop-logic [FLAGS] [OPTIONS] <input>

FLAGS:
    -h, --help       Prints help information
    -t, --tex        output in TeX format (bussproof.sty)
    -V, --version    Prints version information

OPTIONS:
    -o, --out <out>    output file (if omitted, stdout)

ARGS:
    <input>  
```

