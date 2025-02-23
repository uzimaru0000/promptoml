# Syntax

## Basic syntax

基本の構文は以下のようになっています

```toml
start = "start" # スタートするノード

[state.start]
type = "<type>" # Node のタイプ

to = "<node_name>" # 次に実行する Node の名前

[state.done]
type = "done"

```

## Node Type

### Text
テキスト入力をさせるためのNodeです

```toml
type = "text"
name = "name"
message = "Please input your name"
```

### Confirm
y/n で解答をさせるためのNodeです

```toml
type = "confirm"
name = "ready"
message = "Are you ready?"
```

### Password
パスワードのような秘匿情報を入力させるためのNodeです

```toml
type = "password"
name = "password"
message = "Please input your password"
```

### Select 
いくつかの候補から選択させるためのNodeです

```toml
type = "select"
name = "method"
message = "Please select method"
options = ["get", "post", "patch", "delete"]
```

### MultiSelect
いくつかの候補から複数選択させるためのNodeです

```toml
type = "multi_select"
name = "operation"
message = "Please select operations"
options = ["rollback", "deploy", "destroy"]
```

### FuzzySelect 
ユーザーの入力を元に選択肢を検索できるNodeです

```toml
type = "fuzzy_select"
name = "operation"
message = "Please select operations"
options = ["rollback", "deploy", "destroy"]
```

### Condition 
条件式を評価して状態遷移する先を選択できるNodeです

```toml
type = "condition"
name = "check_operation"
condition = "$operation == 'destroy'"
branches = { true = "confirm_destroy", false = "end" }
```

### Set
状態に指定した値を設定するためのNodeです

```toml
type = "set"
name = "const_value"
value = "Hello"
```

### Remove 
状態に設定された値を消すためのNodeです

```toml
type = "remove"
name = "tmp_value"
```

### Done
ステートマシンの終了Nodeです

```toml
type = "done"
```

## Expression

設定ファイル内で利用できる式は以下の通りです

```
<expr>         ::= <binary_expr>

<binary_expr>  ::= <unary_expr> [ <bin_op> <unary_expr> ]

<unary_expr>   ::= <function>
                 | <dollar_expr>
                 | "!" <value>
                 | <value>

<dollar_expr>  ::= "$" <value> { <access> }
<access>       ::= "." <value>
                 | "[" <binary_expr> "]"

<function>     ::= <identifier> "(" <binary_expr> { "," <binary_expr> } ")"

<value>        ::= <string>
                 | <number>
                 | <boolean>
                 | <symbol>
                 | <array>

<array>        ::= "[" [ <binary_expr> { "," <binary_expr> } ] "]"

<string>       ::= "'" { <character> } "'"
<number>       ::= <digit> { <digit> } [ "." { <digit> } ]
<boolean>      ::= "true" | "false"
<symbol>       ::= <letter_or_digit_or_underscore> { <letter_or_digit_or_underscore> }

<bin_op>       ::= "==" | "!=" | ">=" | ">" | "<=" | "<" | "."
```

### 組み込み関数

#### keys

object から key の配列を取得するための関数

#### len

配列の長さを取得するための関数

## 配列の使用例

式の中で配列を使用することができます：

```toml
[state.set_array]
type = "set"
name = "numbers"
value = "[1, 2, 3, 4, 5]"

[state.check_array]
type = "condition"
condition = "len($numbers) > 3"
branches = { true = "large_array", false = "small_array" }

[state.dynamic_array]
type = "set"
name = "combined"
value = "[$value1, $value2, 'static_value']"
```

配列は以下のような場面で使用できます：
- 変数への配列の代入
- 配列の長さの確認
- 動的な配列の生成
- 配列要素へのアクセス（例：`$array[0]`）
