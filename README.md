# prop-logic

## 概要

命題論理ソルバーです．TeX記法等でインラインで証明したい論理式を入力すると，証明図を吐きます．出力には，簡略化した記法とTeX記法のいずれかが選べます．

このブランチにあるものは，wasmを経由してDiscord上のスラッシュコマンドからソルバーを呼び出せるようにしたものです．

## To Be implemented

* 仮定の参照先明示
* 排中律の運用（選択式の予定）

## インストール&起動

* スコープ
  * OAuth2スコープ`application.commands`が必要です．

* 環境変数
  * `BOT_TOKEN`：上記のスコープを持つDiscord Botトークン．
  * `GUILD_ID`：botをインストールするDiscordサーバーのID(18桁)．

* コマンド

```bash
git clone https://github.com/cm-ayf/prop-logic
cd prop-logic
wasm-pack build --release --target nodejs -d bot/pkg

cd bot
npm i
npm run build
npm start
```

## 使い方

* 例

```
/prop-logic input: "((A or B) to C) to (A to C) and (B to C)"

(A ∨ B → C) → (A → C) ∧ (B → C)
  + (A → C) ∧ (B → C)
    + A → C
    | + C
    |   + A ∨ B
    |   | + A
    |   + A ∨ B → C
    + B → C
      + C
        + A ∨ B
        | + B
        + A ∨ B → C
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
