# promptoml

promptoml は TOML ファイルでインタラクティブなコマンドラインウィザードを定義できるツールです。

## 特徴

- TOML による簡単な設定
- 複数の入力タイプをサポート
  - テキスト入力
  - パスワード入力
  - 選択肢からの選択
  - 複数選択
  - ファジー検索付き選択
- 条件分岐による柔軟なフロー制御
- 変数の参照と比較演算

## インストール

```bash
# todo
# cargo install promptoml
```

## 使い方

1. ウィザードを TOML ファイルで定義します：

```toml
start = "name"

[state.name]
type = "text"
name = "name"
message = "What is your name?"
to = "age"

[state.age]
type = "text"
name = "age"
message = "What is your age?"
to = "age_condition"

[state.age_condition]
type = "condition"
name = "age_condition"
condition = "$age > 18"
branches = { true = "gender", false = "end" }

[state.gender]
type = "select"
name = "gender"
message = "What is your gender?"
options = ["Male", "Female", "Other"]
to = "end"

[state.end]
type = "done"
```

2. コマンドラインから実行：

```bash
promptoml -c wizard.toml
```

## 式の文法

条件分岐で使用できる式の文法：

```
<Expr> ::= <Value> | <Eval>
<Value> ::= "'" <string> "'" | <number> | <boolean> | <symbol>
<Eval> ::= <Expr> <Op> <Expr> | <SOp> <Expr>
<Op> ::= <Eq> | <Ord> | "."
<Eq> ::= "==" | "!="
<Ord> ::= ">" | ">=" | "<" | "<="
<SOp> ::= "$" | "!"
```

例：
- `$age > 18`
- `$args.name == 'John'`
- `!$is_valid`

## ライセンス

MIT

## 貢献

バグ報告や機能要望は GitHub Issues へお願いします。
プルリクエストも歓迎です。
