# Usable expression

## Eq, Ord

```
# Equal
condition = "value == 1"
condition = "value != 1"
```

```
# Ord
condition = "value > 1"
condition = "value < 1"
condition = "value >= 1"
condition = "value <= 1"
```

## variable

```
# variable
condition = "$state.age > 18"
condition = "$args.name == 'uzimaru'"
condition = "$args.is_hoge"
condition = "$args.users['admin']"
condition = "$args.data[$key_name]"
```

## Def

```
<Expr> ::= <Value> | <Eval> | <Function>
<Value> ::= "'" <string> "'" | <number> | <boolean> | <symbol>
<Eval> ::= <Expr> <Op> <Expr> | <SOp> <Expr>
<Op> ::= <Eq> | <Ord> | "." | "[" <Expr> "]"
<Eq> ::= "==" | "!="
<Ord> ::= ">" | ">=" | "<" | "<="
<SOp> ::= "$" | "!"
<Function> ::= <symbol> "(" <Expr> ")"
```
